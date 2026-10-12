//! Zulu CLI adapter: `zulu` binary (npm `@comate/zulu`), project-scope
//! `.comate` files plus a global `~/.comate/mcp.json`; SaaS-license auth.

use std::path::{Path, PathBuf};
use std::time::Duration;

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
pub const HARNESS_ID_STR: &str = "zulu-cli";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Zulu CLI (Baidu Comate)";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "zulu";

/// npm package that installs the `zulu` binary.
pub const NPM_PACKAGE: &str = "@comate/zulu";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/zulu-cli.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors for `mcp.json` mutation (global and project shapes match).
pub const OWNED_SELECTORS: &[&str] = &["mcpServers"];

/// Constrained note: license-key `SaaS` with no local auth or relocation lever.
pub const CONSTRAINED_NOTE: &str = "headless license-key SaaS (zulu run --license, required); no config-dir override env var; Turbo mode executes model instructions with an empty blacklist, so run output is untrusted";

/// Concrete adapter for Zulu CLI.
#[derive(Debug, Clone)]
pub struct ZuluCliAdapter {
    id: HarnessId,
}

impl ZuluCliAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn run_with_timeout(binary: &Path, args: &[&str]) -> Option<String> {
        super::run_capturing(binary, args, Duration::from_secs(2))
    }

    /// `zulu --version` is undocumented; npm metadata is the version source.
    fn probe_npm_version() -> Option<String> {
        let path_var = std::env::var_os("PATH")?;
        Self::probe_npm_version_from(&path_var)
    }

    fn probe_npm_version_from(path_var: &std::ffi::OsStr) -> Option<String> {
        let npm = super::find_in_path_within(path_var, &["npm"])?;
        let output = Self::run_with_timeout(&npm, &["ls", "-g"])?;
        Self::harvest_npm_version(&output)
    }

    fn harvest_npm_version(output: &str) -> Option<String> {
        let needle = format!("{NPM_PACKAGE}@");
        let line = output.lines().find(|l| l.contains(&needle))?;
        let at = line.find(&needle)?;
        let rest = line.get(at + needle.len()..)?;
        let version: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
            .collect();
        if version.contains('.') {
            Some(version)
        } else {
            None
        }
    }

    fn global_mcp_path() -> Option<PathBuf> {
        let home = super::home_dir()?;
        Some(home.join(".comate").join("mcp.json"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        match Self::global_mcp_path() {
            Some(mcp) => {
                if mcp.exists() {
                    evidence.push(format!("global mcp.json found at {}", mcp.display()));
                } else {
                    evidence.push(format!("global mcp.json missing at {}", mcp.display()));
                }
            }
            None => {
                evidence.push("could not resolve global mcp.json path (no HOME)".to_owned());
            }
        }
        let engine = super::home_dir().map(|home| home.join(".comate-engine"));
        match engine {
            Some(dir) if dir.exists() => {
                evidence.push(format!("state dir exists at {}", dir.display()));
                let store = dir.join("store");
                if store.exists() {
                    evidence.push(format!("session store present at {}", store.display()));
                }
            }
            Some(dir) => {
                evidence.push(format!("state dir missing at {}", dir.display()));
            }
            None => {
                evidence.push("could not resolve state dir (no HOME)".to_owned());
            }
        }
        for rel in [
            ".comate/mcp.json",
            ".comate/mcp.local.json",
            ".comateignore",
        ] {
            if Path::new(rel).exists() {
                evidence.push(format!("{rel} present in cwd"));
            }
        }
    }
}

impl Default for ZuluCliAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ZuluCliAdapter {
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

        match super::find_in_path(&[EXECUTABLE]) {
            Some(path) => {
                evidence.push(format!(
                    "found binary `{}` at {}",
                    EXECUTABLE,
                    path.display()
                ));
                match Self::run_with_timeout(&path, &["--version"])
                    .and_then(|out| super::parse_version_output(&out))
                {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `{EXECUTABLE} --version`"));
                        version = Some(v);
                    }
                    None => {
                        evidence.push(format!(
                            "version probe failed for `{EXECUTABLE} --version` (undocumented; zulu-cli.md section 7)"
                        ));
                    }
                }
                binary_path = Some(path);
            }
            None => {
                evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            }
        }

        if version.is_none() {
            match Self::probe_npm_version() {
                Some(v) => {
                    evidence.push(format!("version `{v}` via npm metadata ({NPM_PACKAGE})"));
                    version = Some(v);
                }
                None => {
                    evidence.push(format!(
                        "npm metadata probe failed for {NPM_PACKAGE} (zulu --version is undocumented)"
                    ));
                }
            }
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), version.is_some());

        let confidence =
            super::detection_confidence(binary_path.is_some(), version.is_some(), false);
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, "zulu-cli", SCHEMA_VERSION_STR)
    }

    #[expect(clippy::too_many_lines, reason = "surfaces are declarative")]
    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let global_mcp_resolver = PathResolver::fallback_only("~/.comate/mcp.json");
        let mut global_mcp = ConfigSurface::new(
            "mcp.json",
            global_mcp_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        global_mcp.precedence = 10;
        global_mcp.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        global_mcp.backup_required = true;
        global_mcp.restart_behavior = RestartBehavior::Reload;
        surfaces.push(global_mcp);

        let project_mcp_resolver = PathResolver::fallback_only(".comate/mcp.json (project)");
        let mut project_mcp = ConfigSurface::new(
            "project .comate/mcp.json",
            project_mcp_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project_mcp.precedence = 14;
        project_mcp.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        project_mcp.backup_required = false;
        surfaces.push(project_mcp);

        let local_mcp_resolver = PathResolver::fallback_only(".comate/mcp.local.json (project)");
        let mut local_mcp = ConfigSurface::new(
            "mcp.local.json",
            local_mcp_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        local_mcp.precedence = 15;
        local_mcp.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        local_mcp.backup_required = false;
        surfaces.push(local_mcp);

        let hooks_resolver = PathResolver::fallback_only(".comate/hooks.json (project)");
        let mut hooks = ConfigSurface::new(
            "hooks.json",
            hooks_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        hooks.precedence = 12;
        hooks.owned_selectors = vec!["hooks".to_owned()];
        hooks.backup_required = false;
        surfaces.push(hooks);

        let memory_resolver = PathResolver::fallback_only(".comate/memory.json (project)");
        let mut memory = ConfigSurface::new(
            "memory.json",
            memory_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        memory.precedence = 11;
        memory.owned_selectors = vec!["memory".to_owned()];
        memory.backup_required = false;
        surfaces.push(memory);

        let sandbox_resolver = PathResolver::fallback_only(".comate/sandbox.json (project)");
        let mut sandbox = ConfigSurface::new(
            "sandbox.json",
            sandbox_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        sandbox.precedence = 11;
        sandbox.owned_selectors = vec!["sandbox".to_owned()];
        sandbox.backup_required = false;
        surfaces.push(sandbox);

        let ignore_resolver = PathResolver::fallback_only(".comateignore (project)");
        let mut ignore = ConfigSurface::new(
            "comateignore",
            ignore_resolver,
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        ignore.precedence = 9;
        ignore.backup_required = false;
        surfaces.push(ignore);

        for name in ["agents", "rules", "commands"] {
            let resolver = PathResolver::fallback_only(&format!(".comate/{name}/<file> (project)"));
            let surface_id = format!("{name} (.comate)");
            let mut surface = ConfigSurface::new(
                &surface_id,
                resolver,
                DocumentKind::TextFragment,
                ConfigScope::ProjectWorkspace,
                SurfaceOwnership::UserEditable,
            );
            surface.precedence = 8;
            surface.backup_required = false;
            surfaces.push(surface);
        }

        let state_resolver = PathResolver::fallback_only("~/.comate-engine (logs + session store)");
        let mut state = ConfigSurface::new(
            "comate-engine",
            state_resolver,
            DocumentKind::Opaque,
            ConfigScope::Internal,
            SurfaceOwnership::HarnessManaged,
        );
        state.precedence = 0;
        state.backup_required = false;
        surfaces.push(state);

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
            "log/*".to_owned(),
            "store/*".to_owned(),
            "logs/*".to_owned(),
            "cache/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("project-scope config; no relocation env var");
        plan.description = format!(
            " zulu has no config-dir override: project .comate files travel with the \
             checkout; config_root {} is informative only; auth is a per-run \
             --license string superai never stores (zulu-cli.md sections 3-4)",
            instance.config_root
        );
        plan.shared_state_warnings = vec![
            "comate SaaS license and session history are shared across profiles".to_owned(),
            "zulu runs in Turbo mode with an empty blacklist: treat its output as untrusted"
                .to_owned(),
        ];
        plan.auth_prerequisites = vec![crate::adapter::AuthPrerequisite::harness_login(
            "zulu run requires --license <key> on every invocation",
        )];
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.comate".to_owned(),
            "~/.comate-engine".to_owned(),
            ".comate/mcp.json".to_owned(),
            ".comate/mcp.local.json".to_owned(),
            ".comateignore".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ProjectScope | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("zulu-cli requires isolation project_scope, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "mcp.json",
            "mcpServers",
            DocumentKind::Json,
            ConfigScope::User,
            RestartBehavior::Reload,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no plugin mechanism documented (zulu-cli.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{HARNESS_ID_STR, NPM_PACKAGE, OWNED_SELECTORS, ZuluCliAdapter};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ZuluCliAdapter {
        ZuluCliAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-zulu-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::ProjectScope,
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
        let entry = crate::harness_catalog::find_by_id(HARNESS_ID_STR).unwrap();
        assert_eq!(a.id().as_str(), HARNESS_ID_STR);
        assert_eq!(a.display_name(), entry.display_name);
        assert_eq!(a.product_status(), entry.product_status);
        assert_eq!(a.research_doc_link(), entry.research_doc);
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn supported_platforms_covers_all() {
        let a = adapter();
        let os_set: HashSet<String> = a
            .supported_platforms()
            .iter()
            .map(|p| p.os.to_string())
            .collect();
        assert!(os_set.contains("linux"));
        assert!(os_set.contains("windows"));
    }

    #[test]
    fn detection_returns_evidence_and_confidence() {
        let a = adapter();
        let result = a.detection();
        assert!(!result.evidence.is_empty());
        assert!(result.evidence.iter().any(|e| e.contains("constrained")));
        match result.present {
            InstallPresence::Absent => {
                assert!(result.version.is_none());
            }
            InstallPresence::Present => {
                assert!(result.version.is_some());
            }
            InstallPresence::UnknownVersion => {
                assert!(result.evidence.iter().any(|e| e.contains("found binary")));
            }
            InstallPresence::Broken => {
                assert!(!result.evidence.is_empty());
            }
        }
    }

    #[test]
    fn npm_version_harvest_finds_the_package_line() {
        let output = format!("/usr/lib\n├── some-pkg@1.0.0\n└── {NPM_PACKAGE}@1.7.21\n");
        assert_eq!(
            ZuluCliAdapter::harvest_npm_version(&output).as_deref(),
            Some("1.7.21")
        );
        assert_eq!(ZuluCliAdapter::harvest_npm_version("no packages"), None);
        assert_eq!(ZuluCliAdapter::harvest_npm_version(""), None);
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
        } else {
            assert!(!res.compatible);
            assert!(res.schema_version.is_none());
        }
        assert!(!res.notes.is_empty());
    }

    #[test]
    fn config_surfaces_split_global_project_and_state() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 10);
        let global_mcp = surfaces
            .iter()
            .find(|s| s.id == "mcp.json")
            .expect("global mcp.json surface must exist");
        assert_eq!(global_mcp.kind, DocumentKind::Json);
        assert_eq!(global_mcp.scope, ConfigScope::User);
        assert_eq!(global_mcp.ownership, SurfaceOwnership::UserEditable);
        assert!(global_mcp.backup_required);
        for sel in OWNED_SELECTORS {
            assert!(global_mcp.owned_selectors.contains(&(*sel).to_owned()));
        }

        let state = surfaces
            .iter()
            .find(|s| s.id == "comate-engine")
            .expect("comate-engine");
        assert_eq!(state.kind, DocumentKind::Opaque);
        assert_eq!(state.ownership, SurfaceOwnership::HarnessManaged);

        for other in [
            "project .comate/mcp.json",
            "mcp.local.json",
            "hooks.json",
            "memory.json",
            "sandbox.json",
            "comateignore",
        ] {
            assert!(surfaces.iter().any(|s| s.id == other), "missing {other}");
        }
    }

    #[test]
    fn supported_operations_are_constrained() {
        let a = adapter();
        let ops = a.supported_operations();
        for (name, support) in &ops {
            assert_eq!(
                *support,
                AdapterSupport::Constrained,
                "operation {name} should be Constrained"
            );
        }
        let names: HashSet<String> = ops.iter().map(|(n, _)| n.clone()).collect();
        for required in ["detect", "read_config", "manage_mcp", "plan_wrapper"] {
            assert!(names.contains(required), "missing op {required}");
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_logs_and_store() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        let is_excluded = |file: &str| crate::adapters::exclusion_matches(&exclusions, file);
        assert!(!is_excluded("mcp.json"));
        assert!(!is_excluded(".comate/hooks.json") || is_excluded("hooks.json"));
        assert!(is_excluded("log/zulurun-2026-10-12.log"));
        assert!(is_excluded("store/chat_session_abc"));
    }

    #[test]
    fn plan_wrapper_declares_no_relocation_env_vars() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".zulu-work"));
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no relocation env vars may be fabricated: {:?}",
            plan.env_vars
        );
        assert!(!plan.description.is_empty());
        assert!(plan.description.contains("no config-dir override"));
        assert!(
            plan.shared_state_warnings
                .iter()
                .any(|w| w.contains("untrusted"))
        );
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".zulu-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_comate_roots() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(candidates.iter().any(|c| c.contains(".comate")));
        assert!(candidates.iter().any(|c| c.contains(".comate-engine")));
    }

    #[test]
    fn validate_instance_accepts_project_scope() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".zulu-work"));
        a.validate_instance(&inst).unwrap();
        let mut relocated = sample_instance_with_root(&crate::test_util::tmp_abs_str(".zulu-work"));
        relocated.isolation = Isolation::RelocatedRoot;
        a.validate_instance(&relocated).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".zulu-work"));
        inst.isolation = Isolation::EnvOnly;
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "isolation"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn validate_instance_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".zulu-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/zulu_cli")
    }

    fn fixture_path(name: &str) -> PathBuf {
        fixtures_root().join(name)
    }

    #[test]
    fn fixture_missing_file_loads_as_empty() {
        let path = fixture_path("nonexistent.json");
        let map = superai_config::json::load(&path).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn fixture_minimal_parses() {
        let path = fixture_path("mcp.minimal.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::json::load(&path).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn fixture_populated_carries_mcp_servers() {
        let path = fixture_path("mcp.populated.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::json::load(&path).unwrap();
        assert!(map.contains_key("mcpServers"));
    }

    #[test]
    fn fixture_foreign_preserves_unknown_keys_on_edit() {
        let path = fixture_path("mcp.foreign.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::json::load(&path).unwrap();
        assert!(
            original.contains_key("extraSection") || original.contains_key("foreignKey"),
            "foreign fixture must carry a foreign key"
        );
        let dir = crate::test_util::temp_dir_unique("zulu");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("mcp.foreign.copy.json");
        std::fs::copy(&path, &tmp).unwrap();
        superai_config::json::edit(&tmp, |map| {
            map.insert(
                "mcpServers".to_owned(),
                serde_json::json!({"search": {"command": "mcp-search"}}),
            );
        })
        .unwrap();
        let after = superai_config::json::load(&tmp).unwrap();
        assert!(
            after.contains_key("extraSection") || after.contains_key("foreignKey"),
            "foreign keys must be preserved, got {after:?}"
        );
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("mcp.malformed.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::json::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn mcp_decl_targets_global_comate_mcp_json() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "mcp.json");
        assert_eq!(decl.dest_key, "mcpServers");
        assert!(decl.read_only.is_none());
        assert!(a.mcp_absence_reason().is_none());
    }

    #[test]
    fn plugin_absence_names_the_gap() {
        let a = adapter();
        assert!(a.plugin_absence_reason().is_some());
        assert!(a.plugin_decl().is_none());
    }

    #[test]
    fn adapter_is_object_safe() {
        let a = adapter();
        let boxed: Box<dyn Adapter> = Box::new(a);
        assert_eq!(boxed.id().as_str(), HARNESS_ID_STR);
        assert!(!boxed.config_surfaces().is_empty());
        assert!(!boxed.plan_mirror_exclusions().is_empty());
    }
}
