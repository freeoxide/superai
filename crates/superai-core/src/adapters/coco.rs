//! `CoCo` (Snowflake Cortex Code) adapter: `cortex` binary, root
//! `~/.snowflake` or `$SNOWFLAKE_HOME`, JSON tree under `cortex/` with a
//! writable `mcp.json`; paid-Snowflake-account-tied auth.

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
pub const HARNESS_ID_STR: &str = "coco";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Cortex Code (CoCo)";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "cortex";

/// Environment variable that relocates the `~/.snowflake` root.
pub const CONFIG_ENV_VAR: &str = "SNOWFLAKE_HOME";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/coco.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside `cortex/settings.json`.
pub const SETTINGS_OWNED_SELECTORS: &[&str] = &["compactMode", "autoUpdate", "theme"];

/// Owned selectors inside `cortex/permissions.json`.
pub const PERMISSIONS_OWNED_SELECTORS: &[&str] =
    &["onlyAllow", "defaultMode", "dangerouslyAllowAll"];

/// Constrained note: no BYO endpoint; auth is a paid Snowflake account.
pub const CONSTRAINED_NOTE: &str = "requires a paid Snowflake account (or CoCo CLI trial) plus the SNOWFLAKE.CORTEX_USER database role; no BYO provider endpoint; connections.toml is shared with the Snowflake CLI";

/// Concrete adapter for `CoCo`.
#[derive(Debug, Clone)]
pub struct CocoAdapter {
    id: HarnessId,
}

impl CocoAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_snowflake_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(CONFIG_ENV_VAR)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir));
        }
        let home = super::home_dir()?;
        Some(home.join(".snowflake"))
    }

    fn default_cortex_dir() -> Option<PathBuf> {
        Some(Self::default_snowflake_root()?.join("cortex"))
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "detection branches are explicit for evidence"
    )]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        match Self::default_snowflake_root() {
            Some(root) => {
                if root.exists() {
                    evidence.push(format!("snowflake root exists at {}", root.display()));
                    let connections = root.join("connections.toml");
                    if connections.exists() {
                        evidence.push(format!(
                            "connections.toml present at {}",
                            connections.display()
                        ));
                    }
                    match Self::default_cortex_dir() {
                        Some(cortex) if cortex.exists() => {
                            evidence.push(format!("cortex dir exists at {}", cortex.display()));
                            for file in ["settings.json", "permissions.json", "mcp.json"] {
                                let path = cortex.join(file);
                                if path.exists() {
                                    evidence.push(format!("{file} found at {}", path.display()));
                                } else {
                                    evidence.push(format!("{file} missing at {}", path.display()));
                                }
                            }
                        }
                        Some(cortex) => {
                            evidence.push(format!("cortex dir missing at {}", cortex.display()));
                        }
                        None => {}
                    }
                } else {
                    evidence.push(format!("snowflake root missing at {}", root.display()));
                }
            }
            None => {
                evidence.push("could not resolve snowflake root (no HOME)".to_owned());
            }
        }
        if let Ok(dir) = std::env::var(CONFIG_ENV_VAR)
            && !dir.trim().is_empty()
        {
            evidence.push(format!("{CONFIG_ENV_VAR} set to {dir}"));
        } else {
            evidence.push(format!("{CONFIG_ENV_VAR} not set, using ~/.snowflake"));
        }
        if Path::new(".cortex/settings.json").exists() {
            evidence.push(".cortex/settings.json present in cwd (project)".to_owned());
        }
    }
}

impl Default for CocoAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for CocoAdapter {
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
        super::resolution_from_detection(detection, "coco", SCHEMA_VERSION_STR)
    }

    #[expect(clippy::too_many_lines, reason = "surfaces are declarative")]
    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let settings_resolver = PathResolver::new(
            Some("$SNOWFLAKE_HOME/cortex/settings.json"),
            Some("$SNOWFLAKE_HOME/cortex/settings.json"),
            Some("%SNOWFLAKE_HOME%\\cortex\\settings.json"),
            "~/.snowflake/cortex/settings.json",
        );
        let mut settings = ConfigSurface::new(
            "cortex/settings.json",
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
        settings.restart_behavior = RestartBehavior::Reload;
        surfaces.push(settings);

        let permissions_resolver = PathResolver::new(
            Some("$SNOWFLAKE_HOME/cortex/permissions.json"),
            Some("$SNOWFLAKE_HOME/cortex/permissions.json"),
            Some("%SNOWFLAKE_HOME%\\cortex\\permissions.json"),
            "~/.snowflake/cortex/permissions.json",
        );
        let mut permissions = ConfigSurface::new(
            "cortex/permissions.json",
            permissions_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        permissions.precedence = 10;
        permissions.owned_selectors = PERMISSIONS_OWNED_SELECTORS
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        permissions.backup_required = true;
        surfaces.push(permissions);

        let mcp_resolver = PathResolver::new(
            Some("$SNOWFLAKE_HOME/cortex/mcp.json"),
            Some("$SNOWFLAKE_HOME/cortex/mcp.json"),
            Some("%SNOWFLAKE_HOME%\\cortex\\mcp.json"),
            "~/.snowflake/cortex/mcp.json",
        );
        let mut mcp = ConfigSurface::new(
            "cortex/mcp.json",
            mcp_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        mcp.precedence = 11;
        mcp.owned_selectors = vec!["mcpServers".to_owned()];
        mcp.backup_required = true;
        mcp.restart_behavior = RestartBehavior::Reload;
        surfaces.push(mcp);

        let hooks_resolver = PathResolver::new(
            Some("$SNOWFLAKE_HOME/cortex/hooks.json"),
            Some("$SNOWFLAKE_HOME/cortex/hooks.json"),
            Some("%SNOWFLAKE_HOME%\\cortex\\hooks.json"),
            "~/.snowflake/cortex/hooks.json",
        );
        let mut hooks = ConfigSurface::new(
            "cortex/hooks.json",
            hooks_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        hooks.precedence = 9;
        hooks.owned_selectors = vec!["hooks".to_owned()];
        hooks.backup_required = true;
        surfaces.push(hooks);

        let connections_resolver = PathResolver::new(
            Some("$SNOWFLAKE_HOME/connections.toml"),
            Some("$SNOWFLAKE_HOME/connections.toml"),
            Some("%SNOWFLAKE_HOME%\\connections.toml"),
            "~/.snowflake/connections.toml",
        );
        let mut connections = ConfigSurface::new(
            "connections.toml",
            connections_resolver,
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        connections.precedence = 5;
        connections.backup_required = false;
        connections.restart_behavior = RestartBehavior::ReLogin;
        surfaces.push(connections);

        for name in ["skills", "agents", "commands"] {
            let resolver = PathResolver::new(
                Some(&format!("$SNOWFLAKE_HOME/cortex/{name}/")),
                Some(&format!("$SNOWFLAKE_HOME/cortex/{name}/")),
                Some(&format!("%SNOWFLAKE_HOME%\\cortex\\{name}\\")),
                &format!("~/.snowflake/cortex/{name}/"),
            );
            let mut surface = ConfigSurface::new(
                &format!("cortex/{name}"),
                resolver,
                DocumentKind::TextFragment,
                ConfigScope::User,
                SurfaceOwnership::UserEditable,
            );
            surface.precedence = 7;
            surface.backup_required = false;
            surfaces.push(surface);
        }

        let project_resolver = PathResolver::fallback_only(".cortex/settings.json (project)");
        let mut project = ConfigSurface::new(
            "project .cortex/settings.json",
            project_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project.precedence = 14;
        project.backup_required = false;
        surfaces.push(project);

        let runtime_resolver =
            PathResolver::fallback_only("$SNOWFLAKE_HOME/cortex (conversations, cache, logs)");
        let mut runtime = ConfigSurface::new(
            "cortex runtime state",
            runtime_resolver,
            DocumentKind::Opaque,
            ConfigScope::Internal,
            SurfaceOwnership::HarnessManaged,
        );
        runtime.precedence = 0;
        runtime.backup_required = false;
        surfaces.push(runtime);

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
            "conversations/*".to_owned(),
            "cache/*".to_owned(),
            "remote_cache/*".to_owned(),
            "logs/*".to_owned(),
            "memory/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("relocated root via SNOWFLAKE_HOME");
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.description = format!(
            " Wrapper sets {}={} and execs `{}` (auth stays the Snowflake account; no BYO endpoint)",
            CONFIG_ENV_VAR, instance.config_root, EXECUTABLE
        );
        plan.shared_state_warnings = vec![
            format!(
                "paid Snowflake account and {} connections are shared across profiles",
                CONFIG_ENV_VAR
            ),
            "MCP server env values migrate into the OS keychain on first use (shared)".to_owned(),
        ];
        plan.auth_prerequisites = vec![crate::adapter::AuthPrerequisite::harness_login(
            "first-run wizard picks or creates a connection in connections.toml",
        )];
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.snowflake".to_owned(),
            "$SNOWFLAKE_HOME".to_owned(),
            "~/.snowflake/cortex".to_owned(),
            ".cortex/settings.json".to_owned(),
            ".cortex/skills".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("coco requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "cortex/mcp.json",
            "mcpServers",
            DocumentKind::Json,
            ConfigScope::User,
            RestartBehavior::Reload,
        ))
    }

    fn plugin_decl(&self) -> Option<crate::adapter::PluginAdapterDecl> {
        Some(crate::adapter::PluginAdapterDecl::directory_bundle(
            "cortex/plugins",
            Some("plugin.json"),
            RestartBehavior::Reload,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{
        CONFIG_ENV_VAR, CocoAdapter, HARNESS_ID_STR, PERMISSIONS_OWNED_SELECTORS,
        SETTINGS_OWNED_SELECTORS,
    };
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> CocoAdapter {
        CocoAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-coco-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::RelocatedRoot,
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
    fn config_surfaces_include_cortex_json_tree() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 10);
        let settings = surfaces
            .iter()
            .find(|s| s.id == "cortex/settings.json")
            .expect("settings surface must exist");
        assert_eq!(settings.kind, DocumentKind::Json);
        assert_eq!(settings.scope, ConfigScope::User);
        assert!(settings.backup_required);
        assert_eq!(
            settings.owned_selectors.len(),
            SETTINGS_OWNED_SELECTORS.len()
        );

        let permissions = surfaces
            .iter()
            .find(|s| s.id == "cortex/permissions.json")
            .expect("permissions surface");
        for sel in PERMISSIONS_OWNED_SELECTORS {
            assert!(permissions.owned_selectors.contains(&(*sel).to_owned()));
        }

        let connections = surfaces
            .iter()
            .find(|s| s.id == "connections.toml")
            .expect("connections.toml surface");
        assert_eq!(connections.kind, DocumentKind::Toml);
        assert_eq!(
            connections.ownership,
            SurfaceOwnership::ExternalSecretStore,
            "connections.toml carries credentials shared with the Snowflake CLI"
        );
        assert!(connections.owned_selectors.is_empty());

        for other in [
            "cortex/mcp.json",
            "cortex/hooks.json",
            "cortex/skills",
            "project .cortex/settings.json",
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
        for required in [
            "detect",
            "read_config",
            "write_config",
            "manage_mcp",
            "manage_plugins",
            "plan_wrapper",
        ] {
            assert!(names.contains(required), "missing op {required}");
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_conversations_and_logs() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        let is_excluded = |file: &str| crate::adapters::exclusion_matches(&exclusions, file);
        assert!(!is_excluded("cortex/settings.json"));
        assert!(!is_excluded("cortex/mcp.json"));
        assert!(is_excluded("conversations/abc.json"));
        assert!(is_excluded("logs/coco.log"));
        assert!(is_excluded("remote_cache/entry"));
    }

    #[test]
    fn plan_wrapper_sets_snowflake_home() {
        let tmp_root = crate::test_util::tmp_abs_str(".snowflake-work");
        let a = adapter();
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars
                .iter()
                .any(|(k, v)| k == CONFIG_ENV_VAR && v == tmp_root.as_str())
        );
        assert!(!plan.description.is_empty());
        assert!(plan.description.contains(CONFIG_ENV_VAR));
        assert!(plan.args.is_empty());
    }

    #[test]
    fn plan_wrapper_quoting_with_spaces() {
        let tmp_root = crate::test_util::tmp_abs_str("my snowflake work");
        let a = adapter();
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        let env_val = plan
            .env_vars
            .iter()
            .find(|(k, _)| k == CONFIG_ENV_VAR)
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert_eq!(env_val, tmp_root.as_str());
        assert!(!env_val.contains('"'));
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".snowflake-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_snowflake_root() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(candidates.iter().any(|c| c.contains(".snowflake")));
        assert!(candidates.iter().any(|c| c.contains(CONFIG_ENV_VAR)));
        assert!(candidates.iter().any(|c| c.contains(".cortex")));
    }

    #[test]
    fn validate_instance_accepts_relocated_root() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".snowflake-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".snowflake-work"));
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
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".snowflake-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/coco")
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
        for key in SETTINGS_OWNED_SELECTORS {
            assert!(map.contains_key(*key), "populated fixture must carry {key}");
        }
    }

    #[test]
    fn fixture_foreign_preserves_unknown_keys_on_edit() {
        let path = fixture_path("settings.foreign.json");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::json::load(&path).unwrap();
        assert!(
            original.contains_key("snowflakeExtra") || original.contains_key("foreignKey"),
            "foreign fixture must carry a foreign key"
        );
        let dir = crate::test_util::temp_dir_unique("coco");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("settings.foreign.copy.json");
        std::fs::copy(&path, &tmp).unwrap();
        superai_config::json::edit(&tmp, |map| {
            map.insert(
                "theme".to_owned(),
                serde_json::Value::String("light".to_owned()),
            );
        })
        .unwrap();
        let after = superai_config::json::load(&tmp).unwrap();
        assert!(
            after.contains_key("snowflakeExtra") || after.contains_key("foreignKey"),
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
    fn mcp_decl_targets_cortex_mcp_json() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "cortex/mcp.json");
        assert_eq!(decl.dest_key, "mcpServers");
        assert!(decl.read_only.is_none());
        assert!(a.mcp_absence_reason().is_none());
    }

    #[test]
    fn plugin_decl_is_a_staged_directory_bundle() {
        let a = adapter();
        let decl = a.plugin_decl().unwrap();
        assert!(!decl.requires_execution);
        assert_eq!(decl.dest_dir.as_deref(), Some("cortex/plugins"));
        assert_eq!(decl.discovery_manifest.as_deref(), Some("plugin.json"));
        assert!(a.plugin_absence_reason().is_none());
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
