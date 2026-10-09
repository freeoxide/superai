//! Toride backend adoption (PKG-05..08): a posture-forcing runner plus the
//! catalog-method adapter for the six package-manager kinds.

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use toride_apps::backend::{
    Backend, BackendId, InstallRequest, UninstallRequest, UpdateRequest, Version as BackendVersion,
};
use toride_apps::backends::HomebrewBackend;
use toride_apps::runner::CommandRunner;
use toride_apps::{
    CargoBackend, InstallOptions, NpmBackend, Operation, PipxBackend, Target, UninstallOptions,
    UpdatePlan, UvBackend, plan_install, plan_uninstall,
};
use toride_registry::{App, Arch, Availability, InstallMethod as TorideMethod, Os, TorideId};
use toride_runner::policy::{ArgvPolicy, EnvPrecedence, OutputCap, PathResolution};
use toride_runner::{
    CommandOutput, CommandSpec, DuctRunner, Error as RunnerError, OutputMode, Runner,
};

use crate::error::CoreError;
use crate::install_catalog::{
    CommandTokens, InstallCatalogEntry, InstallMethod, InstallMethodKind,
};
use crate::install_execute::{EXEC_TIMEOUT, OUTPUT_LIMIT, minimal_env_vars};
use crate::process::{ProcessOutput, contains_shell_metachars, display_command, scrub_stderr};

/// `(spec, output)` pairs retained per bridge so verb callers can recover
/// the child's real output; backends swallow it inside their sync verbs.
const OUTPUT_LOG_CAP: usize = 8;

/// Runner rewriting every spec to superai's posture before delegation:
/// validated argv, the exact env allowlist, `ChildEnvNoCwd`, timeout, cap.
pub struct SecRunner {
    inner: Arc<dyn Runner>,
    log: Mutex<Vec<(CommandSpec, CommandOutput)>>,
}

impl fmt::Debug for SecRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecRunner").finish_non_exhaustive()
    }
}

impl SecRunner {
    /// Wrap `inner`; every dispatched spec is forced before it runs.
    pub fn new(inner: Arc<dyn Runner>) -> Self {
        Self {
            inner,
            log: Mutex::new(Vec::new()),
        }
    }

    fn record(&self, spec: &CommandSpec, output: &CommandOutput) {
        let mut log = self.log.lock().unwrap_or_else(PoisonError::into_inner);
        if log.len() >= OUTPUT_LOG_CAP {
            log.remove(0);
        }
        log.push((spec.clone(), output.clone()));
    }

    /// Output of the most recent run of exactly `program` + `args`.
    fn last_output(&self, program: &str, args: &[String]) -> Option<CommandOutput> {
        let log = self.log.lock().unwrap_or_else(PoisonError::into_inner);
        log.iter()
            .rev()
            .find(|(spec, _)| spec.program == program && spec.args == args)
            .map(|(_, output)| output.clone())
    }
}

impl Runner for SecRunner {
    fn run(&self, spec: &CommandSpec) -> toride_runner::Result<CommandOutput> {
        reject_unsafe_argv(spec)?;
        let forced = force_posture(spec);
        let output = self.inner.run(&forced)?;
        self.record(&forced, &output);
        Ok(output)
    }
}

/// Superai's argv gate: the full `SHELL_METACHARS` list plus NUL and shell
/// pairs; offenders are identified by position, never echoed.
fn reject_unsafe_argv(spec: &CommandSpec) -> toride_runner::Result<()> {
    let reject = |detail: String| RunnerError::ArgvRejected {
        program: spec.program.clone(),
        detail,
    };
    if spec.program.is_empty() {
        return Err(reject("program must not be empty".to_owned()));
    }
    if spec.program.contains('\0') {
        return Err(reject("program must not contain NUL".to_owned()));
    }
    if (spec.program == "sh" || spec.program == "bash") && spec.args.iter().any(|a| a == "-c") {
        return Err(reject(
            "must not invoke a shell via `sh -c` / `bash -c`".to_owned(),
        ));
    }
    if contains_shell_metachars(&spec.program) {
        return Err(reject("program contains a shell metacharacter".to_owned()));
    }
    for (index, arg) in spec.args.iter().enumerate() {
        if arg.contains('\0') {
            return Err(reject(format!(
                "argument at index {index} must not contain NUL"
            )));
        }
        if contains_shell_metachars(arg) {
            return Err(reject(format!(
                "argument at index {index} contains a shell metacharacter"
            )));
        }
    }
    Ok(())
}

/// Rewrite `spec` with superai's posture; the child env is exactly the
/// allowlist, so caller env entries and removals are dropped.
fn force_posture(spec: &CommandSpec) -> CommandSpec {
    CommandSpec::new(spec.program.clone())
        .args(spec.args.iter().cloned())
        .stdin_null(true)
        .clear_env(true)
        .envs(minimal_env_vars())
        .env_precedence(EnvPrecedence::RemoveWins)
        .path_resolution(PathResolution::ChildEnvNoCwd)
        .argv_policy(ArgvPolicy::RejectShellMetachars)
        .timeout(EXEC_TIMEOUT)
        .output_cap(OutputCap::Always(OUTPUT_LIMIT))
        .output_mode(OutputMode::Capture)
        .redact(true)
}

/// The six kinds superai routes through toride backends.
const ROUTED_KINDS: [InstallMethodKind; 6] = [
    InstallMethodKind::Npm,
    InstallMethodKind::Homebrew,
    InstallMethodKind::HomebrewCask,
    InstallMethodKind::Cargo,
    InstallMethodKind::Pipx,
    InstallMethodKind::Uv,
];

fn typed_refusal(entry: &InstallCatalogEntry, method: &InstallMethod) -> CoreError {
    CoreError::ExternalInstallRequired {
        harness: entry.harness.clone(),
        instructions: format!(
            "method `{}` has no package-manager backend on this path; see {}",
            method.kind, entry.docs
        ),
    }
}

fn catalog_method<'a>(
    entry: &'a InstallCatalogEntry,
    kind: &InstallMethodKind,
) -> Result<&'a InstallMethod, CoreError> {
    entry
        .methods
        .iter()
        .find(|m| m.kind == *kind)
        .ok_or_else(|| CoreError::Validation {
            field: "method".to_owned(),
            reason: format!(
                "install method `{kind}` not supported for `{}`; supported: {:?}",
                entry.harness,
                entry
                    .methods
                    .iter()
                    .map(|m| m.kind.to_string())
                    .collect::<Vec<_>>()
            ),
        })
}

fn toride_install_method(
    entry: &InstallCatalogEntry,
    method: &InstallMethod,
) -> Result<TorideMethod, CoreError> {
    let package = method.package_name.clone();
    match method.kind {
        InstallMethodKind::Npm => Ok(TorideMethod::Npm {
            package,
            version: None,
        }),
        InstallMethodKind::Homebrew => Ok(TorideMethod::Homebrew {
            cask: false,
            token: package,
        }),
        InstallMethodKind::HomebrewCask => Ok(TorideMethod::Homebrew {
            cask: true,
            token: package,
        }),
        InstallMethodKind::Cargo => Ok(TorideMethod::Cargo {
            crate_: package,
            version: None,
        }),
        InstallMethodKind::Pipx => Ok(TorideMethod::Pipx { package }),
        InstallMethodKind::Uv => Ok(TorideMethod::Uv {
            package,
            version: None,
        }),
        InstallMethodKind::Mise | InstallMethodKind::Direct | InstallMethodKind::External => {
            Err(typed_refusal(entry, method))
        }
    }
}

fn toride_app(entry: &InstallCatalogEntry, install: TorideMethod) -> App {
    App {
        id: TorideId::slugify(&entry.harness),
        name: entry.harness.clone(),
        aliases: Vec::new(),
        summary: None,
        description: None,
        homepage: None,
        license: None,
        developer: None,
        binaries: entry.executables.clone(),
        latest: None,
        // superai's platform constraints gate upstream of the adapter; the
        // toride planner only enforces its own cask/OS rules.
        platforms: Vec::new(),
        artifacts: Vec::new(),
        install,
        sources: Vec::new(),
        availability: Availability::Available,
    }
}

/// Host platform pair the adapter routes for, in superai's vocabulary
/// (`linux`/`macos`/`windows`; `x86_64`/`aarch64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostPlatform<'a> {
    /// Operating system name.
    pub os: &'a str,
    /// Architecture name.
    pub arch: &'a str,
}

impl HostPlatform<'_> {
    /// The platform superai is running on.
    pub fn current() -> HostPlatform<'static> {
        HostPlatform {
            os: if cfg!(target_os = "macos") {
                "macos"
            } else if cfg!(target_os = "windows") {
                "windows"
            } else {
                "linux"
            },
            arch: if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            },
        }
    }
}

fn target_for(platform: HostPlatform<'_>) -> Target {
    let os = match platform.os {
        "macos" => Os::MacOs,
        "windows" => Os::Windows,
        "linux" => Os::Linux,
        _ => {
            if cfg!(target_os = "macos") {
                Os::MacOs
            } else if cfg!(target_os = "windows") {
                Os::Windows
            } else {
                Os::Linux
            }
        }
    };
    let arch = match platform.arch {
        "x86_64" => Arch::X86_64,
        "aarch64" => Arch::Aarch64,
        _ => {
            if cfg!(target_arch = "aarch64") {
                Arch::Aarch64
            } else {
                Arch::X86_64
            }
        }
    };
    Target::new(os, arch)
}

fn split_argv(argv: &[String]) -> Result<(&str, &[String]), CoreError> {
    let (program, rest) = argv.split_first().ok_or_else(|| CoreError::Validation {
        field: "operation".to_owned(),
        reason: "toride operation rendered an empty argv".to_owned(),
    })?;
    Ok((program, rest))
}

fn tokens_from_operation(operation: &Operation) -> Result<CommandTokens, CoreError> {
    let argv = operation.argv();
    let (program, rest) = split_argv(&argv)?;
    Ok(CommandTokens {
        executable: program.to_owned(),
        args: rest.to_vec(),
    })
}

fn install_options(version: Option<&str>) -> InstallOptions {
    let options = InstallOptions::new();
    match version {
        Some(v) => options.version(Some(BackendVersion::new(v))),
        None => options,
    }
}

fn update_operation(
    entry: &InstallCatalogEntry,
    method: &InstallMethod,
) -> Result<Operation, CoreError> {
    let package = method.package_name.clone();
    match method.kind {
        InstallMethodKind::Npm => Ok(Operation::NpmUpdate {
            package,
            global: true,
        }),
        InstallMethodKind::Cargo => Ok(Operation::CargoUpdate { crate_: package }),
        InstallMethodKind::Pipx => Ok(Operation::PipxUpdate { package }),
        InstallMethodKind::Uv => Ok(Operation::UvUpdate { package }),
        InstallMethodKind::Homebrew => Ok(Operation::BrewUpgrade {
            cask: false,
            token: package,
        }),
        InstallMethodKind::HomebrewCask => Ok(Operation::BrewUpgrade {
            cask: true,
            token: package,
        }),
        InstallMethodKind::Mise | InstallMethodKind::Direct | InstallMethodKind::External => {
            Err(typed_refusal(entry, method))
        }
    }
}

fn backend_id_for(method: &InstallMethod) -> Result<BackendId, CoreError> {
    match method.kind {
        InstallMethodKind::Npm => Ok(BackendId::Npm),
        InstallMethodKind::Cargo => Ok(BackendId::Cargo),
        InstallMethodKind::Pipx => Ok(BackendId::Pipx),
        InstallMethodKind::Uv => Ok(BackendId::Uv),
        InstallMethodKind::Homebrew | InstallMethodKind::HomebrewCask => Ok(BackendId::Homebrew),
        InstallMethodKind::Mise | InstallMethodKind::Direct | InstallMethodKind::External => {
            Err(typed_refusal_for_kind(&method.kind))
        }
    }
}

fn typed_refusal_for_kind(kind: &InstallMethodKind) -> CoreError {
    CoreError::Validation {
        field: "method".to_owned(),
        reason: format!("method `{kind}` has no package-manager backend on this path"),
    }
}

/// Map toride plan-stage refusals onto `CoreError::Validation`; the
/// refusal text is carried verbatim, never replaced with a hand-built argv.
fn map_plan_error(err: toride_apps::Error, verb: &str) -> CoreError {
    match err {
        toride_apps::Error::VersionNotSelectable {
            app,
            method,
            version,
        } => CoreError::Validation {
            field: "version".to_owned(),
            reason: format!(
                "cannot {verb} `{app}` at version {version} via {method}: \
                 the method takes no version operand"
            ),
        },
        toride_apps::Error::InvalidVersion {
            app,
            version,
            reason,
        } => CoreError::Validation {
            field: "version".to_owned(),
            reason: format!("cannot {verb} `{app}` at version {version}: {reason}"),
        },
        toride_apps::Error::UnsupportedMethod {
            app,
            method,
            target,
            reason,
        } => CoreError::Validation {
            field: "method".to_owned(),
            reason: format!("no backend can {verb} `{app}` via {method} on {target}: {reason}"),
        },
        toride_apps::Error::AppDisabled { app } => CoreError::Validation {
            field: "method".to_owned(),
            reason: format!("`{app}` is disabled by its source and cannot be installed"),
        },
        toride_apps::Error::PlatformMismatch {
            app,
            target,
            claims,
        } => CoreError::Validation {
            field: "method".to_owned(),
            reason: format!(
                "`{app}` does not claim support for target {target}: claims {claims:?}"
            ),
        },
        toride_apps::Error::ElevationRequired { backend, operation } => CoreError::Validation {
            field: "elevation".to_owned(),
            reason: format!("`{backend}` requires elevation for {operation}"),
        },
        toride_apps::Error::DryRun { app } => CoreError::Validation {
            field: "dry_run".to_owned(),
            reason: format!("plan for `{app}` is marked dry-run; refusing to execute"),
        },
        other => CoreError::Validation {
            field: "toride_backend".to_owned(),
            reason: other.to_string(),
        },
    }
}

/// Map a runner failure from a backend verb. Displays are rebuilt from the
/// planned argv so toride's `***` placeholder never surfaces.
fn map_runner_error(
    err: RunnerError,
    verb: &str,
    program: &str,
    args: &[String],
    redact: bool,
) -> CoreError {
    let display = display_command(program, args, redact);
    match err {
        RunnerError::CommandFailed {
            program: failed_program,
            exit_code,
            stderr,
            ..
        } => CoreError::Verification {
            path: PathBuf::from(failed_program),
            kind: format!("{verb}_exit"),
            reason: format!(
                "{verb} command `{display}` failed with {exit_code:?}: {}",
                if redact {
                    "[REDACTED]".to_owned()
                } else {
                    stderr.trim().to_owned()
                }
            ),
        },
        RunnerError::CommandTimeout { timeout, .. } => CoreError::BinaryDetection {
            binary: program.to_owned(),
            reason: format!(
                "command timed out after {}s: `{display}`",
                timeout.as_secs()
            ),
        },
        RunnerError::OutputLimitExceeded {
            limit, observed, ..
        } => CoreError::Verification {
            path: PathBuf::from(program),
            kind: "output_limit".to_owned(),
            reason: format!(
                "command output limit exceeded: `{display}` (limit: {limit} bytes, observed: {observed} bytes)"
            ),
        },
        RunnerError::ArgvRejected {
            program: rejected,
            detail,
        } => CoreError::Validation {
            field: "command.argv".to_owned(),
            reason: format!("`{rejected}` rejected: {detail}"),
        },
        RunnerError::ProgramRejected {
            program: rejected,
            detail,
        } => CoreError::BinaryDetection {
            binary: rejected,
            reason: detail,
        },
        RunnerError::SpawnFailed {
            program: failed,
            detail,
        } => CoreError::BinaryDetection {
            binary: failed,
            reason: format!("failed to spawn `{display}`: {detail}"),
        },
        RunnerError::BinaryNotFound(binary) => CoreError::BinaryDetection {
            binary,
            reason: "not found on PATH".to_owned(),
        },
        RunnerError::WaitFailed {
            program: failed,
            detail,
        }
        | RunnerError::StdinFailed {
            program: failed,
            detail,
        } => CoreError::BinaryDetection {
            binary: failed,
            reason: detail,
        },
        RunnerError::Io(detail) | RunnerError::OutputParse(detail) | RunnerError::Other(detail) => {
            CoreError::BinaryDetection {
                binary: program.to_owned(),
                reason: detail,
            }
        }
        other => CoreError::BinaryDetection {
            binary: program.to_owned(),
            reason: other.to_string(),
        },
    }
}

fn map_run_error(
    err: toride_apps::Error,
    verb: &str,
    program: &str,
    args: &[String],
    redact: bool,
) -> CoreError {
    match err {
        toride_apps::Error::Command(runner_err) => {
            map_runner_error(runner_err, verb, program, args, redact)
        }
        other => map_plan_error(other, verb),
    }
}

fn to_process_output(output: &CommandOutput, redact: bool) -> ProcessOutput {
    ProcessOutput::new(
        output.stdout.clone(),
        scrub_stderr(&output.stderr, redact),
        output.exit_code,
    )
}

/// The toride execution surface for superai: plans through toride's pure
/// planner, executes through the six backends, all under [`SecRunner`].
#[derive(Debug)]
pub struct TorideBridge {
    sec: Arc<SecRunner>,
}

impl Default for TorideBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl TorideBridge {
    /// Production bridge over toride's duct runner.
    pub fn new() -> Self {
        Self {
            sec: Arc::new(SecRunner::new(Arc::new(DuctRunner))),
        }
    }

    /// Bridge over an injected runner (tests; no real child spawns).
    pub fn with_runner(inner: Arc<dyn Runner>) -> Self {
        Self {
            sec: Arc::new(SecRunner::new(inner)),
        }
    }

    fn backend_for(&self, id: BackendId) -> Result<Box<dyn Backend>, CoreError> {
        #[expect(
            clippy::clone_on_ref_ptr,
            reason = "widening Arc<SecRunner> to Arc<dyn Runner> needs clone; Arc::clone will not unsize"
        )]
        let runner: Arc<dyn Runner> = self.sec.clone();
        let runner = CommandRunner::new(runner);
        Ok(match id {
            BackendId::Npm => Box::new(NpmBackend::new(runner)),
            BackendId::Cargo => Box::new(CargoBackend::new(runner)),
            BackendId::Pipx => Box::new(PipxBackend::new(runner)),
            BackendId::Uv => Box::new(UvBackend::new(runner)),
            BackendId::Homebrew => Box::new(HomebrewBackend::new(runner)),
            other => {
                return Err(CoreError::Validation {
                    field: "backend".to_owned(),
                    reason: format!(
                        "toride routed the plan to `{other}`, a backend superai does not attach"
                    ),
                });
            }
        })
    }

    fn captured(
        &self,
        program: &str,
        args: &[String],
        redact: bool,
    ) -> Result<ProcessOutput, CoreError> {
        self.sec.last_output(program, args).map_or_else(
            || {
                Err(CoreError::Verification {
                    path: PathBuf::from(program),
                    kind: "bridge_capture".to_owned(),
                    reason: format!(
                        "backend reported success without dispatching `{}`",
                        display_command(program, args, redact)
                    ),
                })
            },
            |output| Ok(to_process_output(&output, redact)),
        )
    }

    /// Plan and execute an install; version refusals fail at plan time as
    /// `CoreError::Validation`, never a hand-built fallback argv.
    pub fn install(
        &self,
        entry: &InstallCatalogEntry,
        kind: &InstallMethodKind,
        version: Option<&str>,
        redact: bool,
        platform: HostPlatform<'_>,
    ) -> Result<ProcessOutput, CoreError> {
        let method = catalog_method(entry, kind)?;
        let app = toride_app(entry, toride_install_method(entry, method)?);
        let target = target_for(platform);
        let plan = plan_install(&app, &target, &install_options(version))
            .map_err(|e| map_plan_error(e, "install"))?;
        let install_argv = plan.operation.argv();
        let (program, args) = split_argv(&install_argv)?;
        let backend = self.backend_for(plan.backend)?;
        backend
            .install_sync(InstallRequest::new(&plan, &target))
            .map_err(|e| map_run_error(e, "install", program, args, redact))?;
        self.captured(program, args, redact)
    }

    /// Plan and execute an update through the mapped backend. Updates never
    /// target a version; the manager's current is the only destination.
    pub fn update(
        &self,
        entry: &InstallCatalogEntry,
        kind: &InstallMethodKind,
        redact: bool,
        platform: HostPlatform<'_>,
    ) -> Result<ProcessOutput, CoreError> {
        let method = catalog_method(entry, kind)?;
        let operation = update_operation(entry, method)?;
        let update_argv = operation.argv();
        let (program, args) = split_argv(&update_argv)?;
        let target = target_for(platform);
        let plan = UpdatePlan {
            app: TorideId::slugify(&entry.harness),
            backend: backend_id_for(method)?,
            operation,
            dry_run: false,
            requires_elevation: false,
        };
        let backend = self.backend_for(plan.backend)?;
        backend
            .update_sync(UpdateRequest::new(&plan, &target))
            .map_err(|e| map_run_error(e, "update", program, args, redact))?;
        self.captured(program, args, redact)
    }

    /// Execute a six-kind update recognized from its toride-generated argv
    /// (the `UpdateCommandRunner` seam hands over plain tokens). `None` when
    /// the argv is not a routed update shape; the caller runs it natively.
    pub fn update_by_argv(
        &self,
        executable: &str,
        args: &[String],
        redact: bool,
    ) -> Option<Result<ProcessOutput, CoreError>> {
        let (backend, operation) = update_operation_from_argv(executable, args)?;
        let update_argv = operation.argv();
        let (program, rest) = split_argv(&update_argv).ok()?;
        let target = target_for(HostPlatform::current());
        let plan = UpdatePlan {
            app: TorideId::slugify(program),
            backend,
            operation,
            dry_run: false,
            requires_elevation: false,
        };
        let backend = self.backend_for(plan.backend).ok()?;
        Some(
            backend
                .update_sync(UpdateRequest::new(&plan, &target))
                .map_err(|e| map_run_error(e, "update", program, rest, redact))
                .and_then(|()| self.captured(program, rest, redact)),
        )
    }

    /// Plan and execute an uninstall through the mapped backend; zap is
    /// hard-coded false so cask removal never touches collateral files.
    pub fn uninstall(
        &self,
        entry: &InstallCatalogEntry,
        kind: &InstallMethodKind,
        redact: bool,
        platform: HostPlatform<'_>,
    ) -> Result<ProcessOutput, CoreError> {
        let method = catalog_method(entry, kind)?;
        let app = toride_app(entry, toride_install_method(entry, method)?);
        let target = target_for(platform);
        let plan = plan_uninstall(&app, &target, &UninstallOptions { zap: false })
            .map_err(|e| map_plan_error(e, "uninstall"))?;
        let uninstall_argv = plan.operation.argv();
        let (program, args) = split_argv(&uninstall_argv)?;
        let backend = self.backend_for(plan.backend)?;
        backend
            .uninstall_sync(UninstallRequest::new(&plan, &target))
            .map_err(|e| map_run_error(e, "uninstall", program, args, redact))?;
        self.captured(program, args, redact)
    }
}

/// Preview argv for an install without executing: toride's pure planner
/// over the mapped method, rendered back to `CommandTokens`.
pub fn preview_install(
    entry: &InstallCatalogEntry,
    kind: &InstallMethodKind,
    version: Option<&str>,
    platform: HostPlatform<'_>,
) -> Result<CommandTokens, CoreError> {
    let method = catalog_method(entry, kind)?;
    let app = toride_app(entry, toride_install_method(entry, method)?);
    let target = target_for(platform);
    let plan = plan_install(&app, &target, &install_options(version))
        .map_err(|e| map_plan_error(e, "install"))?;
    tokens_from_operation(&plan.operation)
}

/// Preview argv for an uninstall without executing; zap is always false.
pub fn preview_uninstall(
    entry: &InstallCatalogEntry,
    kind: &InstallMethodKind,
    platform: HostPlatform<'_>,
) -> Result<CommandTokens, CoreError> {
    let method = catalog_method(entry, kind)?;
    let app = toride_app(entry, toride_install_method(entry, method)?);
    let target = target_for(platform);
    let plan = plan_uninstall(&app, &target, &UninstallOptions { zap: false })
        .map_err(|e| map_plan_error(e, "uninstall"))?;
    tokens_from_operation(&plan.operation)
}

/// Preview argv for an update without executing. Mirrors the operation the
/// update verb executes; toride publishes no public update planner.
pub fn preview_update(
    entry: &InstallCatalogEntry,
    kind: &InstallMethodKind,
) -> Result<CommandTokens, CoreError> {
    let method = catalog_method(entry, kind)?;
    tokens_from_operation(&update_operation(entry, method)?)
}

/// Whether `kind` routes through the toride backends.
pub fn is_routed_kind(kind: &InstallMethodKind) -> bool {
    ROUTED_KINDS.contains(kind)
}

/// The version operand a routed install may carry: a channel names no
/// manager operand, so channels install at the manager's default.
pub fn routed_install_version(requested: Option<&str>) -> Option<&str> {
    requested.filter(|v| !crate::install_execute::is_channel(v))
}

/// Recognize the argv of a six-kind update verb toride's planner emits and
/// rebuild the `(backend, operation)` pair it came from. `None` for anything
/// else (mise, a foreign command) — the caller then runs natively.
fn update_operation_from_argv(executable: &str, args: &[String]) -> Option<(BackendId, Operation)> {
    let mut parts = vec![executable.to_owned()];
    parts.extend(args.iter().cloned());
    let joined = parts;
    let matches = |shape: &[&str]| -> bool {
        joined.len() == shape.len() && joined.iter().zip(shape).all(|(got, want)| got == want)
    };
    let last = args.last().map(String::as_str)?;
    let operation = match (executable, args.first().map(String::as_str)) {
        ("npm", Some("update")) if matches(&["npm", "update", "-g", last]) => {
            Operation::NpmUpdate {
                package: last.to_owned(),
                global: true,
            }
        }
        ("cargo", Some("install")) if matches(&["cargo", "install", "--force", last]) => {
            Operation::CargoUpdate {
                crate_: last.to_owned(),
            }
        }
        ("pipx", Some("upgrade")) if matches(&["pipx", "upgrade", last]) => Operation::PipxUpdate {
            package: last.to_owned(),
        },
        ("uv", Some("tool")) if matches(&["uv", "tool", "upgrade", last]) => Operation::UvUpdate {
            package: last.to_owned(),
        },
        ("brew", Some("upgrade")) => {
            if matches(&["brew", "upgrade", "--cask", last]) {
                Operation::BrewUpgrade {
                    cask: true,
                    token: last.to_owned(),
                }
            } else if matches(&["brew", "upgrade", last]) {
                Operation::BrewUpgrade {
                    cask: false,
                    token: last.to_owned(),
                }
            } else {
                return None;
            }
        }
        _ => return None,
    };
    // Round-trip: route only argv toride itself would render, so a foreign
    // command can never be reinterpreted as a different toride operation.
    if operation.argv() != joined {
        return None;
    }
    Some((
        match &operation {
            Operation::NpmUpdate { .. } => BackendId::Npm,
            Operation::CargoUpdate { .. } => BackendId::Cargo,
            Operation::PipxUpdate { .. } => BackendId::Pipx,
            Operation::UvUpdate { .. } => BackendId::Uv,
            Operation::BrewUpgrade { .. } => BackendId::Homebrew,
            _ => return None,
        },
        operation,
    ))
}

/// Shared hermetic runner for crate tests: records every dispatched spec and
/// replays queued outputs (an empty stdout when the queue runs dry).
#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Arc, Mutex, PoisonError};

    use toride_runner::{CommandOutput, CommandSpec, Runner};

    #[derive(Debug, Clone, Default)]
    pub(crate) struct SpyRunner {
        specs: Arc<Mutex<Vec<CommandSpec>>>,
        outputs: Arc<Mutex<Vec<CommandOutput>>>,
    }

    impl SpyRunner {
        pub(crate) fn enqueue(&self, output: CommandOutput) {
            self.outputs
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(output);
        }

        pub(crate) fn recorded(&self) -> Vec<CommandSpec> {
            self.specs
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        }
    }

    impl Runner for SpyRunner {
        fn run(&self, spec: &CommandSpec) -> toride_runner::Result<CommandOutput> {
            self.specs
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(spec.clone());
            let mut outputs = self.outputs.lock().unwrap_or_else(PoisonError::into_inner);
            Ok(if outputs.is_empty() {
                CommandOutput::from_stdout("")
            } else {
                outputs.remove(0)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::SpyRunner;
    use super::*;
    use crate::install_catalog::{DetectHints, PlatformConstraints};
    use std::time::Duration;

    fn entry(harness: &str, kind: InstallMethodKind, package: &str) -> InstallCatalogEntry {
        InstallCatalogEntry {
            harness: harness.to_owned(),
            executables: vec!["tool".to_owned()],
            bundle_ids: Vec::new(),
            apps: Vec::new(),
            methods: vec![InstallMethod {
                kind,
                package_name: package.to_owned(),
                tap: None,
                repo: None,
                registry: None,
            }],
            version_source: String::new(),
            constraints: PlatformConstraints::default(),
            detect: DetectHints::default(),
            requires_admin: false,
            checksum: None,
            conflicts: Vec::new(),
            docs: "https://example.com/docs".to_owned(),
            last_verified: "2026-01-01".to_owned(),
        }
    }

    fn sec_over(spy: &SpyRunner) -> SecRunner {
        SecRunner::new(Arc::new(spy.clone()))
    }

    fn platform(os: &'static str, arch: &'static str) -> HostPlatform<'static> {
        HostPlatform { os, arch }
    }

    #[test]
    fn forced_spec_pins_the_process_posture() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stdout("ok"));
        let permissive = CommandSpec::new("npm")
            .arg("install")
            .env("SUPERAI_REQUEST_VAR", "1")
            .cwd("/tmp")
            .timeout(Duration::from_secs(1));
        sec_over(&spy).run(&permissive).unwrap();
        let forced = &spy.recorded()[0];
        assert_eq!(forced.program, "npm");
        assert_eq!(forced.args, ["install"]);
        assert!(forced.clear_env, "child env must start empty");
        assert_eq!(forced.env_precedence, EnvPrecedence::RemoveWins);
        assert_eq!(forced.path_resolution, PathResolution::ChildEnvNoCwd);
        assert_eq!(forced.argv_policy, ArgvPolicy::RejectShellMetachars);
        assert_eq!(forced.output_cap, OutputCap::Always(OUTPUT_LIMIT));
        assert_eq!(forced.timeout, Some(EXEC_TIMEOUT));
        assert!(forced.stdin_null, "child stdin must be null");
        assert!(forced.stdin.is_none());
        assert!(forced.cwd.is_none(), "backends never pick a cwd");
        assert!(forced.redact);
        assert_eq!(forced.output_mode, OutputMode::Capture);
        assert_eq!(forced.env, minimal_env_vars());
        assert!(
            forced.env_remove.is_empty(),
            "the allowlist is forced whole; caller removals are dropped"
        );
        assert!(
            !forced
                .env
                .iter()
                .any(|(key, _)| key == "SUPERAI_REQUEST_VAR"),
            "a caller env addition must not reach the child"
        );
    }

    #[test]
    fn argv_gate_rejects_the_full_metachar_list_before_delegation() {
        // Every one of these passes CommandTokens::validate's shorter list
        // today (install_catalog.rs); the bridge must still refuse it.
        for bad in [
            "a&b", "a!b", "a\"b", "a'b", "a\nb", "a\rb", "a|b", "a>b", "a<b", "a;b",
        ] {
            let spy = SpyRunner::default();
            let err = sec_over(&spy)
                .run(&CommandSpec::new("npm").arg(bad))
                .unwrap_err();
            assert!(
                matches!(err, RunnerError::ArgvRejected { .. }),
                "`{bad}` must be rejected: {err:?}"
            );
            assert!(
                spy.recorded().is_empty(),
                "`{bad}` must be refused before delegation"
            );
        }
    }

    #[test]
    fn argv_gate_rejects_nul_shell_pairs_and_empty_program() {
        for spec in [
            CommandSpec::new("np\u{0}m"),
            CommandSpec::new("npm").arg("inst\u{0}all"),
            CommandSpec::new("sh").arg("-c"),
            CommandSpec::new("bash").args(["-c", "true"]),
            CommandSpec::new(""),
        ] {
            let spy = SpyRunner::default();
            let err = sec_over(&spy).run(&spec).unwrap_err();
            assert!(
                matches!(err, RunnerError::ArgvRejected { .. }),
                "spec {:?} must be rejected: {err:?}",
                spec.program
            );
            assert!(spy.recorded().is_empty());
        }
    }

    #[test]
    fn last_output_returns_the_latest_run_for_repeated_argv() {
        let spy = SpyRunner::default();
        let spec = CommandSpec::new("npm").arg("install");
        let enforcer = sec_over(&spy);
        spy.enqueue(CommandOutput::from_stdout("first"));
        enforcer.run(&spec).unwrap();
        spy.enqueue(CommandOutput::from_stdout("second"));
        enforcer.run(&spec).unwrap();
        assert_eq!(
            enforcer
                .last_output("npm", &["install".to_owned()])
                .unwrap()
                .stdout,
            "second"
        );
    }

    #[test]
    fn preview_install_pins_argv_for_the_six_kinds() {
        let cases = [
            (
                InstallMethodKind::Npm,
                "@openai/codex",
                Some("1.2.3"),
                "npm",
                "linux",
                vec!["install", "-g", "@openai/codex@1.2.3"],
            ),
            (
                InstallMethodKind::Npm,
                "@openai/codex",
                None,
                "npm",
                "linux",
                vec!["install", "-g", "@openai/codex"],
            ),
            (
                InstallMethodKind::Homebrew,
                "ripgrep",
                Some("14.1.0"),
                "brew",
                "linux",
                vec!["install", "ripgrep@14.1.0"],
            ),
            (
                InstallMethodKind::Homebrew,
                "ripgrep",
                None,
                "brew",
                "linux",
                vec!["install", "ripgrep"],
            ),
            (
                InstallMethodKind::HomebrewCask,
                "firefox",
                None,
                "brew",
                // casks exist only on macOS; the cask flag is the whole point.
                "macos",
                vec!["install", "--cask", "firefox"],
            ),
            (
                InstallMethodKind::Cargo,
                "ripgrep",
                Some("14.1.0"),
                "cargo",
                "linux",
                // cargo addresses versions with --version, not pkg@ver.
                vec!["install", "--version", "14.1.0", "ripgrep"],
            ),
            (
                InstallMethodKind::Cargo,
                "ripgrep",
                None,
                "cargo",
                "linux",
                vec!["install", "ripgrep"],
            ),
            (
                InstallMethodKind::Pipx,
                "black",
                None,
                "pipx",
                "linux",
                vec!["install", "black"],
            ),
            (
                InstallMethodKind::Uv,
                "ruff",
                Some("0.6.0"),
                "uv",
                "linux",
                // uv pins with ==ver, not @ver.
                vec!["tool", "install", "ruff==0.6.0"],
            ),
            (
                InstallMethodKind::Uv,
                "ruff",
                None,
                "uv",
                "linux",
                vec!["tool", "install", "ruff"],
            ),
        ];
        for (kind, package, version, executable, os, args) in cases {
            let harness = entry("codex-cli", kind.clone(), package);
            let tokens = preview_install(&harness, &kind, version, platform(os, "x86_64")).unwrap();
            assert_eq!(tokens.executable, executable, "executable for {kind}");
            assert_eq!(tokens.args, args, "argv for {kind}");
        }
    }

    #[test]
    fn preview_install_cask_refuses_non_macos_targets() {
        let cask = entry("warp", InstallMethodKind::HomebrewCask, "warp");
        let err = preview_install(
            &cask,
            &InstallMethodKind::HomebrewCask,
            None,
            platform("linux", "x86_64"),
        )
        .unwrap_err();
        assert!(
            matches!(err, CoreError::Validation { .. }),
            "cask off macOS must be a validation refusal: {err}"
        );
    }

    #[test]
    fn preview_install_pipx_refuses_a_version_at_plan_time() {
        let pipx = entry("aider", InstallMethodKind::Pipx, "aider");
        let err = preview_install(
            &pipx,
            &InstallMethodKind::Pipx,
            Some("0.1.0"),
            platform("linux", "x86_64"),
        )
        .unwrap_err();
        match err {
            CoreError::Validation { field, reason } => {
                assert_eq!(field, "version");
                assert!(
                    reason.contains("no version operand"),
                    "refusal must be carried verbatim: {reason}"
                );
            }
            other => panic!("expected a typed validation refusal, got {other}"),
        }
    }

    #[test]
    fn preview_uninstall_never_spells_zap() {
        let cask = entry("warp", InstallMethodKind::HomebrewCask, "warp");
        let tokens = preview_uninstall(
            &cask,
            &InstallMethodKind::HomebrewCask,
            platform("macos", "aarch64"),
        )
        .unwrap();
        assert_eq!(tokens.executable, "brew");
        assert_eq!(tokens.args, ["uninstall", "--cask", "warp"]);
        let npm = entry("codex-cli", InstallMethodKind::Npm, "@openai/codex");
        let tokens =
            preview_uninstall(&npm, &InstallMethodKind::Npm, platform("linux", "x86_64")).unwrap();
        assert_eq!(tokens.executable, "npm");
        assert_eq!(tokens.args, ["uninstall", "-g", "@openai/codex"]);
    }

    #[test]
    fn preview_update_pins_argv_for_the_six_kinds() {
        let cases = [
            (
                InstallMethodKind::Npm,
                "@openai/codex",
                "npm",
                vec!["update", "-g", "@openai/codex"],
            ),
            (
                InstallMethodKind::Homebrew,
                "ripgrep",
                "brew",
                vec!["upgrade", "ripgrep"],
            ),
            (
                InstallMethodKind::HomebrewCask,
                "firefox",
                "brew",
                vec!["upgrade", "--cask", "firefox"],
            ),
            // cargo's upgrade story is a forced re-install at latest.
            (
                InstallMethodKind::Cargo,
                "ripgrep",
                "cargo",
                vec!["install", "--force", "ripgrep"],
            ),
            (
                InstallMethodKind::Pipx,
                "black",
                "pipx",
                vec!["upgrade", "black"],
            ),
            // uv's verb is `upgrade`, not the `update` superai used to build.
            (
                InstallMethodKind::Uv,
                "ruff",
                "uv",
                vec!["tool", "upgrade", "ruff"],
            ),
        ];
        for (kind, package, executable, args) in cases {
            let harness = entry("codex-cli", kind.clone(), package);
            let tokens = preview_update(&harness, &kind).unwrap();
            assert_eq!(tokens.executable, executable, "executable for {kind}");
            assert_eq!(tokens.args, args, "update argv for {kind}");
        }
    }

    fn assert_typed_external(result: Result<CommandTokens, CoreError>, kind: &InstallMethodKind) {
        match result {
            Err(CoreError::ExternalInstallRequired {
                harness,
                instructions,
            }) => {
                assert_eq!(harness, "cline", "for {kind}");
                assert!(instructions.contains("docs"), "for {kind}: {instructions}");
            }
            other => panic!("`{kind}` must refuse typed, got {other:?}"),
        }
    }

    #[test]
    fn non_routable_kinds_refuse_typed_on_every_verb() {
        for kind in [
            InstallMethodKind::Mise,
            InstallMethodKind::Direct,
            InstallMethodKind::External,
        ] {
            let harness = entry("cline", kind.clone(), "cline");
            assert_typed_external(
                preview_install(&harness, &kind, None, platform("linux", "x86_64")),
                &kind,
            );
            assert_typed_external(
                preview_uninstall(&harness, &kind, platform("linux", "x86_64")),
                &kind,
            );
            assert_typed_external(preview_update(&harness, &kind), &kind);
        }
        assert!(!is_routed_kind(&InstallMethodKind::Mise));
        assert!(is_routed_kind(&InstallMethodKind::Uv));
    }

    #[test]
    fn unknown_kind_for_entry_is_a_validation_error() {
        let npm_only = entry("codex-cli", InstallMethodKind::Npm, "@openai/codex");
        let err = preview_install(
            &npm_only,
            &InstallMethodKind::Cargo,
            None,
            platform("linux", "x86_64"),
        )
        .unwrap_err();
        assert!(matches!(err, CoreError::Validation { .. }), "{err}");
    }

    #[test]
    fn bridge_install_runs_mapped_argv_under_the_forced_posture() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stdout("installed"));
        let bridge = TorideBridge::with_runner(Arc::new(spy.clone()));
        let npm = entry("codex-cli", InstallMethodKind::Npm, "@openai/codex");
        let out = bridge
            .install(
                &npm,
                &InstallMethodKind::Npm,
                Some("1.2.3"),
                false,
                platform("linux", "x86_64"),
            )
            .unwrap();
        assert!(out.success);
        assert_eq!(out.stdout, "installed");
        let recorded = spy.recorded();
        assert_eq!(recorded.len(), 1, "npm install is one command");
        assert_eq!(recorded[0].program, "npm");
        assert_eq!(recorded[0].args, ["install", "-g", "@openai/codex@1.2.3"]);
        assert!(recorded[0].clear_env);
        assert_eq!(recorded[0].output_cap, OutputCap::Always(OUTPUT_LIMIT));
        assert_eq!(recorded[0].timeout, Some(EXEC_TIMEOUT));
    }

    #[test]
    fn bridge_install_failure_maps_to_install_exit_with_redacted_stderr() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stderr("registry unreachable", 1));
        let bridge = TorideBridge::with_runner(Arc::new(spy));
        let npm = entry("codex-cli", InstallMethodKind::Npm, "@openai/codex");
        let err = bridge
            .install(
                &npm,
                &InstallMethodKind::Npm,
                None,
                true,
                platform("linux", "x86_64"),
            )
            .unwrap_err();
        match err {
            CoreError::Verification { path, kind, reason } => {
                assert_eq!(path, PathBuf::from("npm"));
                assert_eq!(kind, "install_exit");
                assert!(reason.contains("npm install -g @openai/codex"), "{reason}");
                assert!(reason.contains("[REDACTED]"), "{reason}");
                assert!(!reason.contains("registry unreachable"), "{reason}");
            }
            other => panic!("expected install_exit verification, got {other}"),
        }

        let spy_plain = SpyRunner::default();
        spy_plain.enqueue(CommandOutput::from_stderr("registry unreachable", 1));
        let bridge = TorideBridge::with_runner(Arc::new(spy_plain));
        let err = bridge
            .install(
                &npm,
                &InstallMethodKind::Npm,
                None,
                false,
                platform("linux", "x86_64"),
            )
            .unwrap_err();
        let CoreError::Verification { kind, reason, .. } = err else {
            panic!("expected install_exit verification");
        };
        assert_eq!(kind, "install_exit");
        assert!(reason.contains("registry unreachable"), "{reason}");
    }

    #[test]
    fn bridge_homebrew_install_returns_the_install_output_not_the_probe() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stdout("install-ran"));
        spy.enqueue(CommandOutput::from_stdout("137.0"));
        let bridge = TorideBridge::with_runner(Arc::new(spy.clone()));
        let brew = entry("ripgrep", InstallMethodKind::Homebrew, "ripgrep");
        let out = bridge
            .install(
                &brew,
                &InstallMethodKind::Homebrew,
                None,
                false,
                platform("linux", "x86_64"),
            )
            .unwrap();
        assert_eq!(out.stdout, "install-ran");
        assert!(
            spy.recorded().len() >= 2,
            "brew probes the version after installing"
        );
    }

    #[test]
    fn bridge_update_and_uninstall_run_the_previewed_argv() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stdout("updated"));
        let bridge = TorideBridge::with_runner(Arc::new(spy.clone()));
        let uv = entry("ruff", InstallMethodKind::Uv, "ruff");
        let out = bridge
            .update(
                &uv,
                &InstallMethodKind::Uv,
                false,
                platform("linux", "x86_64"),
            )
            .unwrap();
        assert_eq!(out.stdout, "updated");
        assert_eq!(spy.recorded()[0].args, ["tool", "upgrade", "ruff"]);

        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stdout("removed"));
        let bridge = TorideBridge::with_runner(Arc::new(spy.clone()));
        let out = bridge
            .uninstall(
                &uv,
                &InstallMethodKind::Uv,
                false,
                platform("linux", "x86_64"),
            )
            .unwrap();
        assert_eq!(out.stdout, "removed");
        assert_eq!(spy.recorded()[0].args, ["tool", "uninstall", "ruff"]);
    }

    #[test]
    fn bridge_refuses_mise_before_any_command_runs() {
        let spy = SpyRunner::default();
        let bridge = TorideBridge::with_runner(Arc::new(spy.clone()));
        let mise = entry("opencode", InstallMethodKind::Mise, "opencode");
        let err = bridge
            .update(
                &mise,
                &InstallMethodKind::Mise,
                false,
                platform("linux", "x86_64"),
            )
            .unwrap_err();
        assert!(
            matches!(err, CoreError::ExternalInstallRequired { .. }),
            "{err}"
        );
        assert!(spy.recorded().is_empty());
    }

    #[test]
    fn runner_errors_map_onto_the_existing_core_variants() {
        let cases: Vec<(RunnerError, &'static str)> = vec![
            (
                RunnerError::CommandTimeout {
                    program: "npm".to_owned(),
                    args: vec!["install".to_owned()],
                    timeout: EXEC_TIMEOUT,
                },
                "timed out",
            ),
            (
                RunnerError::OutputLimitExceeded {
                    program: "npm".to_owned(),
                    args: String::new(),
                    limit: OUTPUT_LIMIT,
                    observed: OUTPUT_LIMIT + 1,
                },
                "output limit exceeded",
            ),
            (
                RunnerError::ArgvRejected {
                    program: "npm".to_owned(),
                    detail: "argument at index 0 contains a shell metacharacter".to_owned(),
                },
                "rejected",
            ),
            (
                RunnerError::ProgramRejected {
                    program: "npm".to_owned(),
                    detail: "not found on PATH".to_owned(),
                },
                "not found on PATH",
            ),
            (
                RunnerError::SpawnFailed {
                    program: "npm".to_owned(),
                    detail: "permission denied".to_owned(),
                },
                "failed to spawn",
            ),
        ];
        for (err, needle) in cases {
            let mapped = map_runner_error(err, "install", "npm", &["install".to_owned()], false);
            let text = format!("{mapped}");
            assert!(text.contains(needle), "mapping lost `{needle}`: {text}");
        }
        let timeout = RunnerError::CommandTimeout {
            program: "npm".to_owned(),
            args: vec![],
            timeout: EXEC_TIMEOUT,
        };
        assert!(matches!(
            map_runner_error(timeout, "update", "npm", &[], false),
            CoreError::BinaryDetection { .. }
        ));
        let limit = RunnerError::OutputLimitExceeded {
            program: "npm".to_owned(),
            args: String::new(),
            limit: OUTPUT_LIMIT,
            observed: OUTPUT_LIMIT + 1,
        };
        match map_runner_error(limit, "uninstall", "npm", &[], false) {
            CoreError::Verification { kind, reason, .. } => {
                assert_eq!(kind, "output_limit");
                assert!(reason.contains("output limit exceeded"));
            }
            other => panic!("expected output_limit verification, got {other}"),
        }
    }

    #[test]
    fn caller_env_never_overrides_the_allowlist() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stdout(""));
        sec_over(&spy)
            .run(
                &CommandSpec::new("cargo")
                    .env("PATH", "/hostile/bin")
                    .env("HOME", "/hostile")
                    .env_remove("HOME"),
            )
            .unwrap();
        let forced = &spy.recorded()[0];
        // PATH drives ChildEnvNoCwd program resolution, so an override here
        // would redirect both the child env and the spawned binary.
        assert_eq!(
            forced.env,
            minimal_env_vars(),
            "the forced env must be exactly the allowlist"
        );
        assert!(forced.env_remove.is_empty());
        assert!(
            !forced
                .env
                .iter()
                .any(|(_, value)| value.contains("/hostile")),
            "no caller-supplied value may survive the rewrite"
        );
    }

    #[test]
    fn update_by_argv_routes_every_six_kind_update_verb() {
        let cases = [
            (
                "npm",
                vec!["update", "-g", "@openai/codex"],
                CommandOutput::from_stdout("npm-updated"),
            ),
            (
                "brew",
                vec!["upgrade", "ripgrep"],
                CommandOutput::from_stdout("brew-updated"),
            ),
            (
                "brew",
                vec!["upgrade", "--cask", "firefox"],
                CommandOutput::from_stdout("cask-updated"),
            ),
            (
                "cargo",
                vec!["install", "--force", "ripgrep"],
                CommandOutput::from_stdout("cargo-updated"),
            ),
            (
                "pipx",
                vec!["upgrade", "black"],
                CommandOutput::from_stdout("pipx-updated"),
            ),
            (
                "uv",
                vec!["tool", "upgrade", "ruff"],
                CommandOutput::from_stdout("uv-updated"),
            ),
        ];
        for (program, args, output) in cases {
            let spy = SpyRunner::default();
            spy.enqueue(output.clone());
            let bridge = TorideBridge::with_runner(Arc::new(spy.clone()));
            let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
            let out = bridge
                .update_by_argv(program, &args, false)
                .unwrap_or_else(|| panic!("`{program} {args:?}` must route"))
                .expect("routed update must succeed");
            assert_eq!(out.stdout, output.stdout, "argv `{program} {args:?}`");
            assert_eq!(spy.recorded()[0].program, program, "{args:?}");
            assert_eq!(spy.recorded()[0].args, args, "{program}");
            assert!(spy.recorded()[0].clear_env, "{program}");
        }
    }

    #[test]
    fn update_by_argv_refuses_shapes_toride_would_not_render() {
        let bridge = TorideBridge::with_runner(Arc::new(SpyRunner::default()));
        // npm without -g, mise, a foreign program, and truncated brew argv
        // are all native-path commands, never a toride operation.
        let native = [
            ("npm", vec!["update".to_owned(), "pkg".to_owned()]),
            ("mise", vec!["upgrade".to_owned(), "pkg".to_owned()]),
            ("curl", vec!["https://example.com".to_owned()]),
            ("brew", vec!["upgrade".to_owned()]),
            ("cargo", vec!["install".to_owned(), "ripgrep".to_owned()]),
        ];
        for (program, args) in native {
            assert!(
                bridge.update_by_argv(program, &args, false).is_none(),
                "`{program} {args:?}` must not be reinterpreted as a toride update"
            );
        }
    }

    #[test]
    fn update_by_argv_maps_a_failed_update_to_update_exit() {
        let spy = SpyRunner::default();
        spy.enqueue(CommandOutput::from_stderr("network unreachable", 1));
        let bridge = TorideBridge::with_runner(Arc::new(spy));
        let err = bridge
            .update_by_argv(
                "npm",
                &[
                    "update".to_owned(),
                    "-g".to_owned(),
                    "@openai/codex".to_owned(),
                ],
                true,
            )
            .expect("argv must route")
            .unwrap_err();
        match err {
            CoreError::Verification { kind, reason, .. } => {
                assert_eq!(kind, "update_exit");
                assert!(reason.contains("npm update -g @openai/codex"), "{reason}");
                assert!(reason.contains("[REDACTED]"), "{reason}");
            }
            other => panic!("expected update_exit verification, got {other}"),
        }
    }

    #[test]
    fn routed_install_version_drops_channels() {
        assert_eq!(routed_install_version(Some("1.2.3")), Some("1.2.3"));
        assert_eq!(routed_install_version(Some("latest")), None);
        assert_eq!(routed_install_version(Some("stable")), None);
        assert_eq!(routed_install_version(None), None);
    }
}
