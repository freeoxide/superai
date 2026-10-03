//! Continue adapter: project/explicit via `~/.continue/config.yaml` and `--config`.
//! Research source: `docs/harness-configs/continue-dev.md` (last verified 2026-08-25).

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
pub const HARNESS_ID_STR: &str = "continue-dev";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Continue";

/// Binary name resolved on PATH during detection (CLI `cn`).
pub const EXECUTABLE: &str = "cn";

/// Older binary name the detection lookup also accepts.
pub const EXECUTABLE_ALT: &str = "continue";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/continue-dev.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Hosted features (hub, cloud crawling, data export) stay foreign.
pub const OWNED_SELECTORS: &[&str] = &[
    "models",
    "model",
    "provider",
    "apiBase",
    "apiKey",
    "mcpServers",
    "rules",
    "context",
    "prompts",
    "docs",
];

/// Selectors for legacy JSON config.
pub const LEGACY_OWNED_SELECTORS: &[&str] = &[
    "models",
    "tabAutocompleteModel",
    "embeddingsProvider",
    "contextProviders",
    "mcpServers",
    "systemMessage",
];

/// Isolation via `project-scope` with an explicit `--config` overlay; project `.continue/` dirs are preserved.
#[derive(Debug, Clone)]
pub struct ContinueDevAdapter {
    id: HarnessId,
}

impl ContinueDevAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        let home = super::home_dir()?;
        Some(home.join(".continue"))
    }

    fn config_path_for_root(root: &Path) -> PathBuf {
        root.join("config.yaml")
    }

    #[expect(clippy::excessive_nesting, reason = "detection branches are explicit")]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        match Self::default_config_root() {
            Some(root) => {
                if root.exists() {
                    evidence.push(format!("config root exists at {}", root.display()));
                    let cfg = Self::config_path_for_root(&root);
                    let cfg_json = root.join("config.json");
                    if cfg.exists() {
                        evidence.push(format!("config.yaml found at {}", cfg.display()));
                        match std::fs::read_to_string(&cfg) {
                            Ok(text) if text.contains("models:") => {
                                evidence.push("config.yaml contains models".to_owned());
                            }
                            Ok(_) => {}
                            Err(err) => evidence
                                .push(format!("config unreadable at {}: {err}", cfg.display())),
                        }
                    } else if cfg_json.exists() {
                        evidence.push(format!(
                            "legacy config.json found at {}",
                            cfg_json.display()
                        ));
                    } else {
                        evidence.push(format!("config.yaml missing at {}", cfg.display()));
                    }
                    if root.join(".env").exists() || Path::new(".env").exists() {
                        evidence.push(".env present".to_owned());
                    }
                    if root.join("rules").exists() || Path::new(".continue/rules").exists() {
                        evidence.push("rules directory present".to_owned());
                    }
                    if root.join("mcpServers").exists() {
                        evidence.push("mcpServers directory present".to_owned());
                    }
                } else {
                    evidence.push(format!("config root missing at {}", root.display()));
                }
            }
            None => {
                evidence.push("could not resolve default config root (no HOME)".to_owned());
            }
        }
        if Path::new(".continue/config.yaml").exists() || Path::new(".continuerc.json").exists() {
            evidence.push("project .continue config present".to_owned());
        }
        if Path::new("config.yaml").exists() {
            evidence.push("cwd config.yaml present (explicit)".to_owned());
        }
    }
}

impl Default for ContinueDevAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ContinueDevAdapter {
    fn id(&self) -> HarnessId {
        self.id.clone()
    }

    fn display_name(&self) -> &str {
        DISPLAY_NAME
    }

    fn product_status(&self) -> ProductStatus {
        ProductStatus::Acquired
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

        match super::find_in_path(&[EXECUTABLE, EXECUTABLE_ALT]) {
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
        super::resolution_from_detection(detection, "continue", SCHEMA_VERSION_STR)
    }

    #[expect(clippy::too_many_lines, reason = "surfaces are declarative")]
    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let config_resolver = PathResolver::new(
            Some("~/.continue/config.yaml"),
            Some("~/.continue/config.yaml"),
            Some("%USERPROFILE%\\.continue\\config.yaml"),
            "~/.continue/config.yaml",
        );
        let mut config = ConfigSurface::new(
            "config.yaml",
            config_resolver,
            DocumentKind::Yaml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        config.precedence = 10;
        config.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        config.backup_required = true;
        config.restart_behavior = RestartBehavior::Reload;
        surfaces.push(config);

        let legacy_resolver = PathResolver::new(
            Some("~/.continue/config.json"),
            Some("~/.continue/config.json"),
            Some("%USERPROFILE%\\.continue\\config.json"),
            "~/.continue/config.json (legacy)",
        );
        let mut legacy = ConfigSurface::new(
            "config.json",
            legacy_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        legacy.precedence = 9;
        legacy.owned_selectors = LEGACY_OWNED_SELECTORS
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        legacy.backup_required = true;
        surfaces.push(legacy);

        let env_resolver = PathResolver::new(
            Some("~/.continue/.env"),
            Some("~/.continue/.env"),
            Some("%USERPROFILE%\\.continue\\.env"),
            "~/.continue/.env",
        );
        let mut env = ConfigSurface::new(
            ".env",
            env_resolver,
            DocumentKind::Env,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        env.precedence = 11;
        env.backup_required = false;
        surfaces.push(env);

        let workspace_env_resolver = PathResolver::new(
            Some(".continue/.env"),
            Some(".continue/.env"),
            Some(".continue\\.env"),
            ".continue/.env (workspace)",
        );
        let mut ws_env = ConfigSurface::new(
            "workspace/.env",
            workspace_env_resolver,
            DocumentKind::Env,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        ws_env.precedence = 12;
        surfaces.push(ws_env);

        let rules_resolver = PathResolver::new(
            Some("~/.continue/rules/<name>.md"),
            Some("~/.continue/rules/<name>.md"),
            Some("%USERPROFILE%\\.continue\\rules\\<name>.md"),
            "~/.continue/rules/<name>.md",
        );
        let mut rules = ConfigSurface::new(
            "rules",
            rules_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        rules.precedence = 8;
        surfaces.push(rules);

        let prompts_resolver = PathResolver::new(
            Some("~/.continue/prompts/<name>.md"),
            Some("~/.continue/prompts/<name>.md"),
            Some("%USERPROFILE%\\.continue\\prompts\\<name>.md"),
            "~/.continue/prompts/<name>.md",
        );
        let mut prompts = ConfigSurface::new(
            "prompts",
            prompts_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        prompts.precedence = 7;
        surfaces.push(prompts);

        let skills_resolver = PathResolver::new(
            Some("~/.continue/skills/<name>/SKILL.md"),
            Some("~/.continue/skills/<name>/SKILL.md"),
            Some("%USERPROFILE%\\.continue\\skills\\<name>\\SKILL.md"),
            "~/.continue/skills/<name>/SKILL.md",
        );
        let mut skills = ConfigSurface::new(
            "skills",
            skills_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        skills.precedence = 6;
        surfaces.push(skills);

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
            "index/*".to_owned(),
            "embeddings/*".to_owned(),
            "cache/*".to_owned(),
            "logs/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "tmp/*".to_owned(),
            "*.lock".to_owned(),
            "data/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("project/explicit via --config <instance>/config.yaml");
        let config_path = instance.config_root.as_path().join("config.yaml");
        plan.args.push("--config".to_owned());
        plan.args.push(config_path.display().to_string());
        plan.description = format!(
            " Wrapper execs `{} --config {}` (hosted hub/data excluded, project .continue overlay)",
            EXECUTABLE,
            config_path.display()
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.continue/config.yaml".to_owned(),
            "~/.continue/config.json".to_owned(),
            ".continue/config.yaml".to_owned(),
            ".continuerc.json".to_owned(),
            "~/.continue/.env".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ProjectScope
            | Isolation::ExplicitConfig
            | Isolation::RelocatedRoot
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!(
                    "continue-dev requires isolation project_scope or explicit_config, got {other}"
                ),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "config.yaml",
            "mcpServers",
            DocumentKind::Yaml,
            ConfigScope::User,
            RestartBehavior::Reload,
        ).with_shape(crate::adapter::McpDestShape::IdentityList { key: "name".to_owned() })
            .with_read_only("yaml writes refuse (LossyWrite) until a preserving codec exists; inspect/diff only; container is a name-keyed YAML list"))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism documented; IDE extensions are the product distribution, not plugins (continue-dev.md)",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{ContinueDevAdapter, HARNESS_ID_STR};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ContinueDevAdapter {
        ContinueDevAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-continue-1").unwrap(),
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
        } else {
            assert!(!res.compatible);
        }
        assert!(!res.notes.is_empty());
    }

    #[test]
    fn config_surfaces_include_yaml_and_legacy() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 5);
        let yaml = surfaces
            .iter()
            .find(|s| s.id == "config.yaml")
            .expect("config.yaml");
        assert_eq!(yaml.kind, DocumentKind::Yaml);
        assert_eq!(yaml.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(yaml.scope, ConfigScope::User);
        assert!(yaml.backup_required);
        for sel in ["models", "mcpServers", "rules"] {
            assert!(yaml.owned_selectors.contains(&sel.to_owned()));
        }
        let legacy = surfaces
            .iter()
            .find(|s| s.id == "config.json")
            .expect("legacy");
        assert_eq!(legacy.kind, DocumentKind::Json);
        let env = surfaces.iter().find(|s| s.id == ".env").expect(".env");
        assert_eq!(env.kind, DocumentKind::Env);
    }

    #[test]
    fn supported_operations_are_constrained() {
        let a = adapter();
        let ops = a.supported_operations();
        for (_, support) in &ops {
            assert_eq!(*support, AdapterSupport::Constrained);
        }
        let names: HashSet<String> = ops.iter().map(|(n, _)| n.clone()).collect();
        for required in ["detect", "read_config", "write_config", "plan_wrapper"] {
            assert!(names.contains(required));
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_cache_and_logs() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        assert!(exclusions.contains(&"cache/*".to_owned()));
        assert!(exclusions.contains(&"logs/*".to_owned()));
        assert!(!exclusions.contains(&"config.yaml".to_owned()));
    }

    #[test]
    fn plan_wrapper_sets_config_flag() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".continue-work"));
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(plan.args.contains(&"--config".to_owned()));
        let cfg = plan.args.windows(2).find(|w| w[0] == "--config").unwrap()[1].clone();
        assert!(cfg.contains(".continue-work"));
        assert!(cfg.contains("config.yaml"));
        assert!(plan.description.contains("--config"));
        assert!(plan.description.contains("hub"));
    }

    #[test]
    fn plan_wrapper_quoting_with_spaces() {
        let root = crate::test_util::tmp_abs_str("my continue work");
        let a = adapter();
        let inst = sample_instance_with_root(&root);
        let plan = a.plan_wrapper(&inst).unwrap();
        let cfg = plan.args.windows(2).find(|w| w[0] == "--config").unwrap()[1].clone();
        assert_eq!(
            cfg,
            std::path::Path::new(&root)
                .join("config.yaml")
                .display()
                .to_string()
        );
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".continue-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_default_root() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(candidates.iter().any(|c| c.contains(".continue")));
        assert!(candidates.iter().any(|c| c.contains("config.yaml")));
    }

    #[test]
    fn validate_instance_accepts_project_and_explicit() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".continue-work"));
        inst.isolation = Isolation::ProjectScope;
        a.validate_instance(&inst).unwrap();
        inst.isolation = Isolation::ExplicitConfig;
        a.validate_instance(&inst).unwrap();
        inst.isolation = Isolation::Unknown;
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".continue-work"));
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
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".continue-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/continue_dev")
    }

    fn fixture_path(name: &str) -> PathBuf {
        fixtures_root().join(name)
    }

    #[test]
    fn fixture_missing_file_loads_as_empty() {
        let path = fixture_path("nonexistent.yaml");
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn fixture_minimal_parses() {
        let path = fixture_path("config.minimal.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(
            map.contains_key("name")
                || map.contains_key("version")
                || map.is_empty()
                || map.len() <= 3
        );
    }

    #[test]
    fn fixture_populated_parses_and_has_expected_keys() {
        let path = fixture_path("config.populated.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(
            map.contains_key("models")
                || map.contains_key("rules")
                || map.contains_key("mcpServers")
        );
    }

    #[test]
    fn fixture_foreign_survive_because_changing_edit_refuses() {
        let path = fixture_path("config.foreign.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::yaml::load(&path).unwrap();
        assert!(original.contains_key("foreignKey") || original.contains_key("unknownTopLevel"));
        let dir = crate::test_util::temp_dir_unique("continue");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("continue.foreign.copy.yaml");
        std::fs::copy(&path, &tmp).unwrap();
        let result = superai_config::yaml::edit(&tmp, |map| {
            map.insert("rules".to_owned(), serde_json::Value::Array(vec![]));
            assert!(map.contains_key("foreignKey") || map.contains_key("unknownTopLevel"));
        });
        match result {
            Err(superai_config::ConfigError::LossyWrite { format, .. }) => {
                assert_eq!(format, "yaml");
            }
            other => panic!("expected LossyWrite, got {other:?}"),
        }
        let after = superai_config::yaml::load(&tmp).unwrap();
        assert!(
            after.contains_key("foreignKey")
                || after.contains_key("unknownTopLevel")
                || after.contains_key("customField")
        );
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("config.malformed.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::yaml::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn fixture_legacy_json_populated_parses() {
        let path = fixture_path("config.legacy.json");
        assert!(path.exists(), "legacy fixture missing: {}", path.display());
        let map = superai_config::json::load(&path).unwrap();
        assert!(!map.is_empty());
    }

    #[test]
    fn unknown_keys_survive_because_changing_yaml_edit_refuses() {
        let dir = crate::test_util::temp_dir_unique("continue");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("preserve.yaml");
        std::fs::write(&path, "name: test\nforeignKey: keep-me\n").unwrap();
        // codec-honesty (DOC-06): changing YAML writes on existing files are
        // refused outright; preservation is expressed by refusing.
        let result = superai_config::yaml::edit(&path, |map| {
            map.insert("models".to_owned(), serde_json::Value::Array(vec![]));
        });
        assert!(matches!(
            result,
            Err(superai_config::ConfigError::LossyWrite { format: "yaml", .. })
        ));
        let after = superai_config::yaml::load(&path).unwrap();
        assert_eq!(
            after["foreignKey"],
            serde_json::Value::String("keep-me".to_owned())
        );
        drop(std::fs::remove_file(&path));
    }
}
