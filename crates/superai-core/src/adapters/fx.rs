//! fx (Vercel Labs) adapter: `fx` binary, fixed `~/.fx` JSON root (no
//! relocation env documented), global `mcp.json` under the `mcp` key.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, Arch, ConfigScope, ConfigSurface, DetectionResult, DocumentKind, Os,
    PathResolver, Platform, ProductStatus, RestartBehavior, SurfaceOwnership, VersionResolution,
    WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "fx";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "fx (Vercel Labs)";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "fx";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/fx.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside `~/.fx/settings.json`.
pub const SETTINGS_OWNED_SELECTORS: &[&str] = &[
    "provider",
    "providers",
    "models",
    "permission_mode",
    "effort",
    "auto_compact_percent",
    "review_model",
    "fast_mode",
    "ultrafast_mode",
    "auto_upgrade",
    "update_channel",
    "theme",
    "slash_menu_categories",
    "notifications",
    "statusLine",
    "workspaces",
];

/// Restricted key set of `<workspace>/.fx.json`.
pub const PROJECT_OWNED_SELECTORS: &[&str] = &[
    "max_agent_steps",
    "max_tool_result_bytes",
    "context",
    "provider_order",
    "provider_strict",
];

/// Concrete adapter for fx.
#[derive(Debug, Clone)]
pub struct FxAdapter {
    id: HarnessId,
}

impl FxAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        let home = super::home_dir()?;
        Some(home.join(".fx"))
    }

    fn settings_path_for_root(root: &Path) -> PathBuf {
        root.join("settings.json")
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "detection branches are explicit for evidence"
    )]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        match Self::default_config_root() {
            Some(root) => {
                if root.exists() {
                    evidence.push(format!("config root exists at {}", root.display()));
                    let cfg = Self::settings_path_for_root(&root);
                    if cfg.exists() {
                        evidence.push(format!("settings.json found at {}", cfg.display()));
                    } else {
                        evidence.push(format!("settings.json missing at {}", cfg.display()));
                    }
                    let mcp = root.join("mcp.json");
                    if mcp.exists() {
                        evidence.push(format!("mcp.json found at {}", mcp.display()));
                    }
                    for dir in ["skills", "sessions", "recordings", "themes"] {
                        let path = root.join(dir);
                        if path.exists() {
                            evidence.push(format!("{dir} dir present at {}", path.display()));
                        }
                    }
                } else {
                    evidence.push(format!("config root missing at {}", root.display()));
                }
            }
            None => {
                evidence.push("could not resolve default config root (no HOME)".to_owned());
            }
        }
        evidence.push(
            "no config-dir override documented; global root fixed at ~/.fx (fx.md section 3)"
                .to_owned(),
        );
        if Path::new(".fx.json").exists() {
            evidence.push(".fx.json present in cwd (project)".to_owned());
        }
        if Path::new(".mcp.json").exists() {
            evidence.push(".mcp.json present in cwd (project)".to_owned());
        }
    }
}

impl Default for FxAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for FxAdapter {
    fn id(&self) -> HarnessId {
        self.id.clone()
    }

    fn display_name(&self) -> &str {
        DISPLAY_NAME
    }

    fn product_status(&self) -> ProductStatus {
        ProductStatus::Preview
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
            Platform::new(Os::Macos, Arch::X86_64),
            Platform::new(Os::Macos, Arch::Aarch64),
        ]
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
                match super::probe_version_with_fallback(
                    &path,
                    &["-v"],
                    &["--version"],
                    Duration::from_secs(2),
                ) {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `{EXECUTABLE} -v`"));
                        version = Some(v);
                    }
                    None => {
                        evidence.push(format!(
                            "version probe failed for `{EXECUTABLE} -v` (timeout or non-zero)"
                        ));
                    }
                }
                binary_path = Some(path);
            }
            None => {
                evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            }
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), version.is_some());

        let confidence =
            super::detection_confidence(binary_path.is_some(), version.is_some(), false);
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, "fx", SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let settings_resolver = PathResolver::fallback_only("~/.fx/settings.json");
        let mut settings = ConfigSurface::new(
            "settings.json",
            settings_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        settings.precedence = 10;
        settings.owned_selectors = SETTINGS_OWNED_SELECTORS
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        settings.backup_required = true;
        settings.restart_behavior = RestartBehavior::Restart;
        surfaces.push(settings);

        let mcp_resolver = PathResolver::fallback_only("~/.fx/mcp.json");
        let mut mcp = ConfigSurface::new(
            "mcp.json",
            mcp_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        mcp.precedence = 11;
        mcp.owned_selectors = vec!["mcp".to_owned()];
        mcp.backup_required = true;
        mcp.restart_behavior = RestartBehavior::Reload;
        surfaces.push(mcp);

        let project_resolver = PathResolver::fallback_only(".fx.json (workspace)");
        let mut project = ConfigSurface::new(
            "project .fx.json",
            project_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project.precedence = 14;
        project.owned_selectors = PROJECT_OWNED_SELECTORS
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        project.backup_required = false;
        surfaces.push(project);

        let project_mcp_resolver = PathResolver::fallback_only(".mcp.json (workspace)");
        let mut project_mcp = ConfigSurface::new(
            "project .mcp.json",
            project_mcp_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project_mcp.precedence = 15;
        project_mcp.owned_selectors = vec!["mcpServers".to_owned()];
        project_mcp.backup_required = false;
        surfaces.push(project_mcp);

        for (id, hint) in [
            ("skills (~/.fx)", "~/.fx/skills/<name>/SKILL.md"),
            ("skills (.fx)", ".fx/skills/<name>/SKILL.md (project)"),
        ] {
            let mut surface = ConfigSurface::new(
                id,
                PathResolver::fallback_only(hint),
                DocumentKind::TextFragment,
                ConfigScope::User,
                SurfaceOwnership::UserEditable,
            );
            surface.precedence = 7;
            surface.backup_required = false;
            surfaces.push(surface);
        }

        for id in ["sessions", "recordings", "themes"] {
            let mut surface = ConfigSurface::new(
                id,
                PathResolver::fallback_only(&format!("~/.fx/{id}/")),
                DocumentKind::Opaque,
                ConfigScope::Internal,
                SurfaceOwnership::HarnessManaged,
            );
            surface.precedence = 0;
            surface.backup_required = false;
            surfaces.push(surface);
        }

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::all_operations_full()
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "sessions/*".to_owned(),
            "recordings/*".to_owned(),
            "cache/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("fixed root, per-process env overrides");
        plan.description = format!(
            " fx reads a fixed ~/.fx root (no config-dir override documented); \
             config_root {} is informative only; per-process FX_* overrides are not \
             written back (fx.md sections 2-3)",
            instance.config_root
        );
        plan.shared_state_warnings = vec![
            "Vercel/codex/grok login state lives behind fx login and the OS keychain, shared across profiles"
                .to_owned(),
            "workspace settings under `workspaces` key absolute paths, so they pin to real directories"
                .to_owned(),
        ];
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.fx".to_owned(),
            "~/.fx/skills".to_owned(),
            ".fx.json".to_owned(),
            ".mcp.json".to_owned(),
            ".fx/skills".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::FixedPathSingle | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("fx requires isolation fixed_path_single, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "mcp.json",
            "mcp",
            DocumentKind::Json,
            ConfigScope::User,
            RestartBehavior::Reload,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no plugin mechanism documented (fx.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{FxAdapter, HARNESS_ID_STR, PROJECT_OWNED_SELECTORS, SETTINGS_OWNED_SELECTORS};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> FxAdapter {
        FxAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-fx-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::FixedPathSingle,
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
    fn supported_platforms_match_upstream_matrix() {
        let a = adapter();
        let platforms = a.supported_platforms();
        assert_eq!(platforms.len(), 4, "linux+macos, x86_64+aarch64 only");
        let os_set: HashSet<String> = platforms.iter().map(|p| p.os.to_string()).collect();
        assert!(os_set.contains("linux"));
        assert!(os_set.contains("macos"));
        assert!(
            !os_set.contains("windows"),
            "fx.md documents no Windows builds"
        );
    }

    #[test]
    fn detection_returns_evidence_and_confidence() {
        let a = adapter();
        let result = a.detection();
        assert!(!result.evidence.is_empty());
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("no config-dir override"))
        );
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
    fn config_surfaces_include_settings_mcp_and_project_files() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 8);
        let settings = surfaces
            .iter()
            .find(|s| s.id == "settings.json")
            .expect("settings.json surface must exist");
        assert_eq!(settings.kind, DocumentKind::Json);
        assert_eq!(settings.scope, ConfigScope::User);
        assert_eq!(settings.ownership, SurfaceOwnership::UserEditable);
        for sel in SETTINGS_OWNED_SELECTORS {
            assert!(settings.owned_selectors.contains(&(*sel).to_owned()));
        }

        let mcp = surfaces
            .iter()
            .find(|s| s.id == "mcp.json")
            .expect("mcp.json surface");
        assert_eq!(mcp.owned_selectors, vec!["mcp".to_owned()]);

        let project = surfaces
            .iter()
            .find(|s| s.id == "project .fx.json")
            .expect("project .fx.json");
        for sel in PROJECT_OWNED_SELECTORS {
            assert!(project.owned_selectors.contains(&(*sel).to_owned()));
        }

        let project_mcp = surfaces
            .iter()
            .find(|s| s.id == "project .mcp.json")
            .expect("project .mcp.json");
        assert_eq!(project_mcp.owned_selectors, vec!["mcpServers".to_owned()]);
    }

    #[test]
    fn supported_operations_cover_full() {
        let a = adapter();
        let ops = a.supported_operations();
        for (_, support) in &ops {
            assert_eq!(*support, AdapterSupport::Full);
        }
        let names: HashSet<String> = ops.iter().map(|(n, _)| n.clone()).collect();
        for required in [
            "detect",
            "read_config",
            "write_config",
            "manage_mcp",
            "plan_wrapper",
        ] {
            assert!(names.contains(required), "missing op {required}");
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_sessions_and_recordings() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        let is_excluded = |file: &str| crate::adapters::exclusion_matches(&exclusions, file);
        assert!(!is_excluded("settings.json"));
        assert!(!is_excluded("mcp.json"));
        assert!(is_excluded("sessions/abc.json"));
        assert!(is_excluded("recordings/run.bin"));
    }

    #[test]
    fn plan_wrapper_declares_no_relocation_env_vars() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".fx-work"));
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no relocation env vars may be fabricated: {:?}",
            plan.env_vars
        );
        assert!(!plan.description.is_empty());
        assert!(plan.description.contains("~/.fx"));
        assert!(
            plan.shared_state_warnings
                .iter()
                .any(|w| w.contains("keychain"))
        );
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".fx-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_fixed_root() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(candidates.iter().any(|c| c == "~/.fx"));
        assert!(candidates.iter().any(|c| c == ".fx.json"));
    }

    #[test]
    fn validate_instance_accepts_fixed_and_relocated() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".fx-work"));
        a.validate_instance(&inst).unwrap();
        let mut relocated = sample_instance_with_root(&crate::test_util::tmp_abs_str(".fx-work"));
        relocated.isolation = Isolation::RelocatedRoot;
        a.validate_instance(&relocated).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".fx-work"));
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
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".fx-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/fx")
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
        let path = fixture_path("settings.minimal.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::json::load(&path).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn fixture_populated_carries_documented_keys() {
        let path = fixture_path("settings.populated.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::json::load(&path).unwrap();
        assert!(map.contains_key("provider"));
        assert!(map.contains_key("permission_mode"));
        assert!(map.contains_key("workspaces"));
    }

    #[test]
    fn fixture_foreign_preserves_unknown_keys_on_edit() {
        let path = fixture_path("settings.foreign.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::json::load(&path).unwrap();
        assert!(
            original.contains_key("fxUnknownKey") || original.contains_key("foreignKey"),
            "foreign fixture must carry a foreign key"
        );
        let dir = crate::test_util::temp_dir_unique("fx");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("settings.foreign.copy.json");
        std::fs::copy(&path, &tmp).unwrap();
        superai_config::json::edit(&tmp, |map| {
            map.insert(
                "theme".to_owned(),
                serde_json::Value::String("dark".to_owned()),
            );
        })
        .unwrap();
        let after = superai_config::json::load(&tmp).unwrap();
        assert!(
            after.contains_key("fxUnknownKey") || after.contains_key("foreignKey"),
            "foreign keys must be preserved, got {after:?}"
        );
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("settings.malformed.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::json::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn mcp_decl_targets_global_mcp_json_under_mcp_key() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "mcp.json");
        assert_eq!(decl.dest_key, "mcp");
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
