//! Sculptor adapter: orchestrator with global/per-repo env files, harness
//! settings, and worktree/container isolation; `Constrained`.

use std::path::{Path, PathBuf};

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, RestartBehavior, SurfaceOwnership, VersionResolution,
    WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "sculptor";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Sculptor";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "sculptor";

/// Older binary name the detection lookup also accepts.
pub const EXECUTABLE_ALT: &str = "sculptor-app";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/orchestrators.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: workspace/container.
pub const CONSTRAINED_NOTE: &str = "env + harness settings, workspace/container: `~/.sculptor/.env` (global) + `.sculptor/.env` (per-repo, .gitignore) injected into agent env (override-shell toggle), harness settings claude-code managed/custom binary + pi API-key env/provider connections, workspaces git worktree default `~/.sculptor/workspaces/<id>/code/` (Clone/In-place modes), container backend via `run-backend.py` / devcontainer spec, macOS keychain not forwarded to containers (re-auth needed)";

/// Owned selectors for env/harness mutation.
pub const OWNED_SELECTORS: &[&str] = &[
    "claude_code.model",
    "claude_code.api_key",
    "pi.api_key",
    "pi.provider",
    "environment_variables",
    "setup_command",
    "harness.settings",
    "dependencies.claude_code_binary",
];

/// Concrete adapter for Sculptor (`Constrained`, `os_bound`): worktrees plus an optional Docker backend.
#[derive(Debug, Clone)]
pub struct SculptorAdapter {
    id: HarnessId,
}

impl SculptorAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    /// Constraint recorded wherever full support would overstate.
    pub fn constrained_note(&self) -> &str {
        CONSTRAINED_NOTE
    }

    fn global_env_path() -> Option<PathBuf> {
        let home = super::home_dir()?;
        Some(home.join(".sculptor").join(".env"))
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "detection branches are explicit for evidence"
    )]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        evidence.push("harnesses: claude-code (integrated, streaming JSON, control protocol, auto-approved) + pi (RPC, version-pinned extensions) + any terminal agent generically".to_owned());
        match Self::global_env_path() {
            Some(path) => {
                if path.exists() {
                    evidence.push(format!("global env exists at {}", path.display()));
                    match std::fs::read_to_string(&path) {
                        Ok(text)
                            if text.contains("ANTHROPIC_BASE_URL")
                                || text.contains("ANTHROPIC_API_KEY") =>
                        {
                            evidence.push("global env contains ANTHROPIC_*".to_owned());
                        }
                        Ok(_) => {}
                        Err(err) => evidence.push(format!(
                            "global env unreadable at {}: {err}",
                            path.display()
                        )),
                    }
                } else {
                    evidence.push(format!("global env missing at {}", path.display()));
                }
            }
            None => evidence.push("could not resolve global env path (no HOME)".to_owned()),
        }
        let project_env = Path::new(".sculptor").join(".env");
        if project_env.exists() {
            evidence.push(format!(
                "project .sculptor/.env found at {}",
                project_env.display()
            ));
        } else {
            evidence.push(format!(
                "project .sculptor/.env missing at {}",
                project_env.display()
            ));
        }
        let workspaces = super::home_dir().map(|h| h.join(".sculptor").join("workspaces"));
        if let Some(ws) = workspaces {
            if ws.exists() {
                evidence.push(format!("workspaces root exists at {}", ws.display()));
                if let Ok(entries) = std::fs::read_dir(&ws) {
                    let count = entries.count();
                    evidence.push(format!("workspaces root contains {count} entries"));
                }
            } else {
                evidence.push(format!("workspaces root missing at {}", ws.display()));
            }
        }
        let container_backend = Path::new("container").join("recipes").join("docker");
        if container_backend.exists() {
            evidence.push(format!(
                "container recipe present at {}",
                container_backend.display()
            ));
        }
        evidence.push("dependencies: git + GitHub CLI + Claude CLI managed/custom binary, container backend experimental (run-backend.py)".to_owned());
        evidence.push("credential caveat: macOS keychain not forwarded to container, re-auth inside container required".to_owned());
        for var in [
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_API_KEY",
            "CONTAINER_BACKEND",
        ] {
            if let Ok(val) = std::env::var(var)
                && !val.trim().is_empty()
            {
                let preview = if var.contains("KEY") || var.contains("TOKEN") {
                    "[REDACTED]".to_owned()
                } else {
                    val
                };
                evidence.push(format!("{var} set to {preview}"));
            } else {
                evidence.push(format!("{var} not set"));
            }
        }
    }
}

impl Default for SculptorAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for SculptorAdapter {
    fn id(&self) -> HarnessId {
        self.id.clone()
    }

    fn display_name(&self) -> &str {
        DISPLAY_NAME
    }

    fn product_status(&self) -> ProductStatus {
        ProductStatus::Active
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        super::desktop_platforms()
    }

    fn adapter_revision(&self) -> &str {
        ADAPTER_REVISION
    }

    fn research_doc_link(&self) -> &str {
        RESEARCH_DOC
    }

    fn last_verified_date(&self) -> &str {
        LAST_VERIFIED
    }

    fn detection(&self) -> DetectionResult {
        let mut evidence = Vec::new();
        let mut version: Option<String> = None;
        let mut binary_path: Option<PathBuf> = None;

        if let Some(path) = super::find_in_path(&[EXECUTABLE, EXECUTABLE_ALT]) {
            evidence.push(format!(
                "found binary `{}` at {}",
                EXECUTABLE,
                path.display()
            ));
            match super::probe_version(&path) {
                Some(v) => {
                    evidence.push(format!("version `{v}` via `{EXECUTABLE} --version`"));
                    version = Some(v);
                }
                None => {
                    evidence.push(format!(
                        "version probe failed for `{EXECUTABLE} --version` (timeout or non-zero)"
                    ));
                }
            }
            binary_path = Some(path);
        } else {
            evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            evidence.push(
                "Sculptor is desktop app (imbue-ai/sculptor, MIT), detection via ~/.sculptor/.env and workspaces"
                    .to_owned(),
            );
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), version.is_some());

        let confidence = super::detection_confidence(
            binary_path.is_some(),
            version.is_some(),
            evidence.iter().any(|e| e.contains("global env exists")),
        );

        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution(&self) -> VersionResolution {
        self.version_resolution_from(&self.detection())
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        if let Some(v) = detection.version.clone() {
            let mut notes = Vec::new();
            notes.push(format!("detected sculptor version {v}"));
            notes.push(format!("mapped to schema version {SCHEMA_VERSION_STR}"));
            notes.push(format!("constrained: {CONSTRAINED_NOTE}"));
            let mut res =
                VersionResolution::new(Some(v), Some(SCHEMA_VERSION_STR.to_owned()), true);
            res.notes = notes;
            res
        } else {
            let mut res = VersionResolution::unknown();
            res.notes.clone_from(&detection.evidence);
            res
        }
    }

    #[expect(clippy::too_many_lines, reason = "surfaces are declarative")]
    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let global_env_resolver = PathResolver::new(
            Some("~/.sculptor/.env (global env, loaded into agent, override-shell toggle)"),
            Some("~/.sculptor/.env (macOS global env)"),
            Some("%USERPROFILE%\\.sculptor\\.env"),
            "~/.sculptor/.env (global, ANTHROPIC_BASE_URL injection, toggle override host vars)",
        );
        let mut global_env = ConfigSurface::new(
            ".env (global)",
            global_env_resolver,
            DocumentKind::Env,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        global_env.precedence = 10;
        global_env.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        global_env.backup_required = true;
        surfaces.push(global_env);

        let project_env_resolver = PathResolver::fallback_only(
            ".sculptor/.env (per-repo, .gitignore, override-shell toggle, per-repo setup)",
        );
        let mut project_env = ConfigSurface::new(
            ".env (project)",
            project_env_resolver,
            DocumentKind::Env,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project_env.precedence = 12;
        project_env.owned_selectors = vec![
            "ANTHROPIC_BASE_URL".to_owned(),
            "ANTHROPIC_API_KEY".to_owned(),
            "environment_variables".to_owned(),
        ];
        project_env.backup_required = true;
        surfaces.push(project_env);

        let harnesses_resolver = PathResolver::new(
            Some("~/.sculptor/harnesses.json or Settings → Harnesses (claude-code/pi settings)"),
            Some("~/.sculptor/harnesses.json (macOS, managed vs custom binary)"),
            Some("%USERPROFILE%\\.sculptor\\harnesses.json"),
            "~/.sculptor/harnesses.json (Harnesses: Claude model/fast/effort, Pi API-key/provider connections)",
        );
        let mut harnesses = ConfigSurface::new(
            "harnesses (Claude/Pi settings)",
            harnesses_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        harnesses.precedence = 11;
        harnesses.owned_selectors = vec![
            "claude.model".to_owned(),
            "claude.effort".to_owned(),
            "pi.provider".to_owned(),
            "dependencies.claude_code_binary".to_owned(),
        ];
        harnesses.backup_required = true;
        surfaces.push(harnesses);

        let workspaces_resolver = PathResolver::new(
            Some("~/.sculptor/workspaces/<id>/code/ (git worktree default, branch <user>/<slug>)"),
            Some("~/.sculptor/workspaces/<id>/code/ (macOS worktree)"),
            Some("~/.sculptor/workspaces/<id>/code/"),
            "~/.sculptor/workspaces/<id>/code/ (worktree, also Clone/In-place modes, setup command)",
        );
        let mut workspaces = ConfigSurface::new(
            "workspaces (worktree/container)",
            workspaces_resolver,
            DocumentKind::Opaque,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::HarnessManaged,
        );
        workspaces.precedence = 9;
        workspaces.backup_required = false;
        surfaces.push(workspaces);

        let container_resolver = PathResolver::fallback_only(
            "container/recipes/docker/ (experimental container backend, git+claude CLI, backend URL, volumes/ports via env, pairing mode bidirectional sync)",
        );
        let mut container = ConfigSurface::new(
            "container backend (experimental)",
            container_resolver,
            DocumentKind::Opaque,
            ConfigScope::Internal,
            SurfaceOwnership::HarnessManaged,
        );
        container.precedence = 4;
        container.backup_required = false;
        container.restart_behavior = RestartBehavior::Restart;
        surfaces.push(container);

        let deps_resolver = PathResolver::fallback_only(
            "Settings → Dependencies (git, gh CLI, Claude CLI managed/auto-download backend binaries ~100MB cached, signal forwarding)",
        );
        let mut deps = ConfigSurface::new(
            "dependencies (git/gh/Claude CLI)",
            deps_resolver,
            DocumentKind::Opaque,
            ConfigScope::User,
            SurfaceOwnership::HarnessManaged,
        );
        deps.precedence = 6;
        deps.backup_required = false;
        surfaces.push(deps);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        vec![
            ("detect".to_owned(), AdapterSupport::Constrained),
            ("read_config".to_owned(), AdapterSupport::Constrained),
            ("write_config".to_owned(), AdapterSupport::Constrained),
            ("manage_skills".to_owned(), AdapterSupport::Constrained),
            ("manage_mcp".to_owned(), AdapterSupport::Constrained),
            ("manage_plugins".to_owned(), AdapterSupport::Constrained),
            ("configure_provider".to_owned(), AdapterSupport::Constrained),
            ("plan_mirror".to_owned(), AdapterSupport::Constrained),
            ("plan_wrapper".to_owned(), AdapterSupport::Constrained),
            ("scan_candidates".to_owned(), AdapterSupport::Constrained),
            ("validate_instance".to_owned(), AdapterSupport::Constrained),
        ]
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "~/.sculptor/workspaces/*".to_owned(),
            ".sculptor/workspaces/*".to_owned(),
            "cache/*".to_owned(),
            "logs/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "tmp/*".to_owned(),
            "*.lock".to_owned(),
            "pairing/*".to_owned(),
            "DerivedData/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new(
            "os_bound via worktree/container + .env injection (constrained, container pairing)",
        );
        plan.env_vars.push((
            "SCULPTOR_WORKSPACE_PATH".to_owned(),
            format!("{}/code", instance.config_root),
        ));
        plan.env_vars.push((
            "SCULPTOR_ENV_FILE".to_owned(),
            format!("{}/.env", instance.config_root),
        ));
        plan.env_vars.push((
            "SCULPTOR_GLOBAL_ENV".to_owned(),
            instance.config_root.to_string(),
        ));
        plan.description = format!(
            " Wrapper sets SCULPTOR_WORKSPACE_PATH={}/code SCULPTOR_ENV_FILE={}/.env (global ~/.sculptor/.env + .sculptor/.env per-repo, {}), container backend via run-backend.py/devcontainer if enabled",
            instance.config_root, instance.config_root, CONSTRAINED_NOTE
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.sculptor/.env".to_owned(),
            ".sculptor/.env (per-repo)".to_owned(),
            "~/.sculptor/workspaces (worktrees)".to_owned(),
            "~/.sculptor/harnesses.json (Claude/Pi)".to_owned(),
            "container/recipes/docker (container backend)".to_owned(),
            "Settings → Dependencies (git/gh/Claude CLI)".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::OsBound
            | Isolation::Unknown
            | Isolation::ProjectScope
            | Isolation::RelocatedRoot => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!(
                    "sculptor requires isolation os_bound (workspace/container) or project_scope, got {other}: {CONSTRAINED_NOTE}"
                ),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "orchestrator: MCP servers are carried into managed harnesses via Claude settings sync; sculptor has no own MCP dest (orchestrators.md)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "orchestrator: bundled plugins are injected into Claude sessions by sculptor itself; no user plugin mechanism (orchestrators.md)",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{HARNESS_ID_STR, OWNED_SELECTORS, SculptorAdapter};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> SculptorAdapter {
        SculptorAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-sculptor-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::OsBound,
            origin: InstanceOrigin::Created,
            ownership: Ownership::SuperaiCreated,
            template: None,
            created_at: "2026-08-26T00:00:00Z".to_owned(),
            adapter_revision: crate::adapter::ADAPTER_REVISION.to_owned(),
        }
    }

    #[test]
    fn adapter_identity() {
        let a = adapter();
        // Constructor wiring plus catalog registration: an id the catalog
        // does not know can never reconcile with detection or instances.
        let entry = crate::harness_catalog::find_by_id(HARNESS_ID_STR).unwrap();
        assert_eq!(a.id().as_str(), HARNESS_ID_STR);
        assert_eq!(a.product_status(), entry.product_status);
    }

    #[test]
    fn supported_platforms_covers_all() {
        let a = adapter();
        let platforms = a.supported_platforms();
        assert!(platforms.len() >= 3);
        let os_set: HashSet<String> = platforms.iter().map(|p| p.os.to_string()).collect();
        assert!(os_set.contains("linux"));
        assert!(os_set.contains("macos"));
        assert!(os_set.contains("windows"));
    }

    #[test]
    fn detection_returns_evidence_and_confidence() {
        let a = adapter();
        let result = a.detection();
        assert!(!result.evidence.is_empty());
        assert!(result.evidence.iter().any(|e| e.contains("constrained")));
        match result.present {
            InstallPresence::Absent => assert!(result.version.is_none()),
            InstallPresence::Present => assert!(result.version.is_some()),
            InstallPresence::UnknownVersion => {
                assert!(result.evidence.iter().any(|e| e.contains("found binary")));
            }
            InstallPresence::Broken => assert!(!result.evidence.is_empty()),
        }
    }

    #[test]
    fn version_resolution_maps_detected() {
        let a = adapter();
        let res = a.version_resolution();
        if res.detected_version.is_some() {
            assert_eq!(
                res.schema_version.as_deref(),
                Some(super::SCHEMA_VERSION_STR)
            );
            assert!(res.compatible);
            assert!(res.notes.iter().any(|n| n.contains("sculptor")));
        } else {
            assert!(!res.compatible);
            assert!(res.schema_version.is_none());
        }
        assert!(!res.notes.is_empty());
    }

    #[test]
    fn config_surfaces_include_env_and_workspaces() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 5);
        let global_env = surfaces
            .iter()
            .find(|s| s.id == ".env (global)")
            .expect("global env must exist");
        assert_eq!(global_env.kind, DocumentKind::Env);
        assert_eq!(global_env.scope, ConfigScope::User);
        assert_eq!(global_env.ownership, SurfaceOwnership::UserEditable);
        for sel in OWNED_SELECTORS {
            assert!(global_env.owned_selectors.contains(&(*sel).to_owned()));
        }
        let project_env = surfaces
            .iter()
            .find(|s| s.id == ".env (project)")
            .expect("project env must exist");
        assert_eq!(project_env.scope, ConfigScope::ProjectWorkspace);
        let workspaces = surfaces
            .iter()
            .find(|s| s.id == "workspaces (worktree/container)")
            .expect("workspaces must exist");
        assert_eq!(workspaces.kind, DocumentKind::Opaque);
    }

    #[test]
    fn supported_operations_are_constrained() {
        let a = adapter();
        let ops = a.supported_operations();
        assert!(!ops.is_empty());
        for (_, support) in &ops {
            assert_eq!(*support, AdapterSupport::Constrained);
        }
        let names: HashSet<String> = ops.iter().map(|(n, _)| n.clone()).collect();
        for required in [
            "detect",
            "read_config",
            "write_config",
            "plan_wrapper",
            "validate_instance",
        ] {
            assert!(names.contains(required), "missing op {required}");
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_workspaces_and_logs() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        assert!(!exclusions.is_empty());
        for pat in ["~/.sculptor/workspaces/*", "cache/*", "*.lock"] {
            assert!(
                exclusions.contains(&pat.to_owned()),
                "exclusions must contain {pat}"
            );
        }
        assert!(!exclusions.contains(&".env (global)".to_owned()));
    }

    #[test]
    fn plan_wrapper_sets_workspace_and_env() {
        let root = crate::test_util::tmp_abs_str(".sculptor-work");
        let a = adapter();
        let inst = sample_instance_with_root(&root);
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars.iter().any(
                |(k, v)| k == "SCULPTOR_WORKSPACE_PATH" && v == format!("{root}/code").as_str()
            )
        );
        assert!(
            plan.env_vars
                .iter()
                .any(|(k, v)| k == "SCULPTOR_ENV_FILE" && v == format!("{root}/.env").as_str())
        );
        assert!(!plan.description.is_empty());
        assert!(plan.description.contains("SCULPTOR_WORKSPACE_PATH"));
        assert!(plan.description.contains("container"));
    }

    #[test]
    fn plan_wrapper_quoting_with_spaces() {
        let root = crate::test_util::tmp_abs_str("my sculptor work");
        let a = adapter();
        let inst = sample_instance_with_root(&root);
        let plan = a.plan_wrapper(&inst).unwrap();
        let env_val = plan
            .env_vars
            .iter()
            .find(|(k, _)| k == "SCULPTOR_WORKSPACE_PATH")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert_eq!(env_val, format!("{root}/code"));
        assert!(env_val.contains(' '));
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".sculptor-work"));
        inst.harness = HarnessId::new("claude-code").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_env_and_workspaces_and_container() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.contains(".sculptor/.env")));
        assert!(candidates.iter().any(|c| c.contains("workspaces")));
        assert!(candidates.iter().any(|c| c.contains("container")));
    }

    #[test]
    fn validate_instance_accepts_os_bound() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".sculptor-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".sculptor-work"));
        inst.isolation = Isolation::EnvOnly;
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "isolation"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn supported_skill_modes_matches_catalog_constrained() {
        let a = adapter();
        let modes = a.supported_skill_modes();
        assert_eq!(modes.len(), 3);
        let s: HashSet<String> = modes.iter().map(ToString::to_string).collect();
        assert!(s.contains("link_all"));
        assert!(s.contains("link_selected"));
        assert!(s.contains("copy_selected"));
    }

    #[test]
    fn fixture_corpus_validates_secret_free_and_flags_malformed() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sculptor");
        let report = crate::verification::fixture_report(&dir);
        assert!(!report.outcomes.is_empty(), "sculptor corpus must load");
        assert!(
            report.validity_pass,
            "validity: {:?}",
            report
                .outcomes
                .iter()
                .filter(|o| o.exists && o.is_valid != o.expected_valid)
                .map(|o| o.path.display().to_string())
                .collect::<Vec<_>>()
        );
        assert!(
            report.secret_free_pass,
            "sculptor fixtures must be secret-free"
        );
        assert!(report.malformed_count >= 1);
    }

    #[test]
    fn fixture_env_and_harnesses_load() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sculptor");
        let env = superai_config::env_file::load(&dir.join("global.populated.env")).unwrap();
        assert!(!env.is_empty());
        let harnesses = superai_config::json::load(&dir.join("harnesses.populated.json")).unwrap();
        assert!(harnesses.contains_key("claude_code") || !harnesses.is_empty());
        assert!(
            superai_config::json::load(&dir.join("harnesses.malformed.json")).is_err(),
            "malformed harnesses must fail to parse"
        );
    }
}
