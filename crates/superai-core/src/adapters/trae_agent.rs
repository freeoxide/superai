//! Trae Agent adapter: explicit-config via `--config-file` / `TRAE_CONFIG_FILE`;
//! YAML `trae_config.yaml` (or legacy JSON).

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
pub const HARNESS_ID_STR: &str = "trae-agent";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Trae Agent";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "trae-cli";

/// Older binary name the detection lookup also accepts.
pub const EXECUTABLE_ALT: &str = "trae";

/// Environment variable that overrides the config file path.
pub const CONFIG_ENV_VAR: &str = "TRAE_CONFIG_FILE";

/// Default config file fallback.
pub const DEFAULT_CONFIG_FILE_FALLBACK: &str = "trae_config.yaml";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/trae-agent.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors for Trae Agent inside `trae_config.yaml` (YAML).
pub const OWNED_SELECTORS: &[&str] = &[
    "agents",
    "agents.trae_agent.model",
    "agents.trae_agent.max_steps",
    "model_providers",
    "models",
    "lakeview",
    "allow_mcp_servers",
    "mcp_servers",
];

/// Concrete adapter for Trae Agent.
#[derive(Debug, Clone)]
pub struct TraeAgentAdapter {
    id: HarnessId,
}

impl TraeAgentAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_path() -> PathBuf {
        if let Ok(val) = std::env::var(CONFIG_ENV_VAR)
            && !val.trim().is_empty()
        {
            return PathBuf::from(val);
        }
        PathBuf::from(DEFAULT_CONFIG_FILE_FALLBACK)
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        let default_path = Self::default_config_path();
        if default_path.exists() {
            evidence.push(format!("config file exists at {}", default_path.display()));
            match std::fs::read_to_string(&default_path) {
                Ok(text) if text.contains("model_providers") => {
                    evidence.push("config contains model_providers".to_owned());
                }
                Ok(_) => {}
                Err(err) => evidence.push(format!(
                    "config unreadable at {}: {err}",
                    default_path.display()
                )),
            }
        } else {
            evidence.push(format!("config file missing at {}", default_path.display()));
            let json_fallback = Path::new("trae_config.json");
            if json_fallback.exists() {
                evidence.push(format!(
                    "legacy json config found at {}",
                    json_fallback.display()
                ));
            }
        }
        if let Ok(val) = std::env::var(CONFIG_ENV_VAR)
            && !val.trim().is_empty()
        {
            evidence.push(format!("{CONFIG_ENV_VAR} set to {val}"));
            let p = Path::new(&val);
            if p.exists() {
                evidence.push(format!(
                    "env config {CONFIG_ENV_VAR} exists at {}",
                    p.display()
                ));
            }
        } else {
            evidence.push(format!(
                "{CONFIG_ENV_VAR} not set, using {}",
                default_path.display()
            ));
        }
        if Path::new(".env").exists() {
            evidence.push(".env present (api keys)".to_owned());
        }
        if Path::new("trajectories").exists() {
            evidence.push("trajectories directory present".to_owned());
        }
        if Path::new("trae_config.yaml.example").exists() {
            evidence.push("example config present".to_owned());
        }
    }
}

impl Default for TraeAgentAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for TraeAgentAdapter {
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

    fn version_resolution(&self) -> VersionResolution {
        self.version_resolution_from(&self.detection())
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection.clone(), "trae-agent", SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let yaml_resolver = PathResolver::new(
            Some("$TRAE_CONFIG_FILE or trae_config.yaml"),
            Some("$TRAE_CONFIG_FILE or trae_config.yaml"),
            Some("%TRAE_CONFIG_FILE% or trae_config.yaml"),
            "trae_config.yaml (or $TRAE_CONFIG_FILE)",
        );
        let mut yaml = ConfigSurface::new(
            "trae_config.yaml",
            yaml_resolver,
            DocumentKind::Yaml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        yaml.precedence = 10;
        yaml.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        yaml.backup_required = true;
        yaml.restart_behavior = RestartBehavior::Reload;
        surfaces.push(yaml);

        let json_resolver = PathResolver::new(
            Some("trae_config.json (legacy)"),
            Some("trae_config.json (legacy)"),
            Some("trae_config.json (legacy)"),
            "trae_config.json (legacy)",
        );
        let mut legacy = ConfigSurface::new(
            "trae_config.json",
            json_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        legacy.precedence = 9;
        legacy.owned_selectors = vec![
            "default_provider".to_owned(),
            "model_providers".to_owned(),
            "max_steps".to_owned(),
        ];
        legacy.backup_required = true;
        surfaces.push(legacy);

        let env_resolver = PathResolver::new(
            Some(".env"),
            Some(".env"),
            Some(".env"),
            ".env (api keys via python-dotenv)",
        );
        let mut env = ConfigSurface::new(
            ".env",
            env_resolver,
            DocumentKind::Env,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        env.precedence = 11;
        surfaces.push(env);

        let traj_resolver = PathResolver::new(
            Some("trajectories/trajectory_*.json"),
            Some("trajectories/trajectory_*.json"),
            Some("trajectories\\trajectory_*.json"),
            "trajectories/trajectory_*.json",
        );
        let mut traj = ConfigSurface::new(
            "trajectories",
            traj_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::HarnessManaged,
        );
        traj.precedence = 0;
        traj.backup_required = false;
        surfaces.push(traj);

        let skills_resolver = PathResolver::new(
            Some("skills/<name>/SKILL.md"),
            Some("skills/<name>/SKILL.md"),
            Some("skills\\<name>\\SKILL.md"),
            "skills/<name>/SKILL.md",
        );
        let mut skills = ConfigSurface::new(
            "skills",
            skills_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        skills.precedence = 5;
        surfaces.push(skills);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::all_operations_full()
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "trajectories/*".to_owned(),
            "trajectories/**/*".to_owned(),
            "trae-workspace/*".to_owned(),
            "results/*".to_owned(),
            "evaluation/results/*".to_owned(),
            "logs/*".to_owned(),
            "cache/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "tmp/*".to_owned(),
            "*.lock".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("explicit-config via TRAE_CONFIG_FILE/--config-file");
        let config_path = instance.config_root.as_path().join("trae_config.yaml");
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), config_path.display().to_string()));
        plan.args.push("--config-file".to_owned());
        plan.args.push(config_path.display().to_string());
        plan.description = format!(
            " Wrapper sets {}={} and execs `{} --config-file {}`",
            CONFIG_ENV_VAR,
            config_path.display(),
            EXECUTABLE,
            config_path.display()
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "trae_config.yaml".to_owned(),
            "trae_config.json".to_owned(),
            "$TRAE_CONFIG_FILE".to_owned(),
            ".env".to_owned(),
            "trajectories/*".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ExplicitConfig | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("trae-agent requires isolation explicit_config, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "trae_config.yaml",
            "mcp_servers",
            DocumentKind::Yaml,
            ConfigScope::User,
            RestartBehavior::Restart,
        ).with_read_only("yaml writes refuse (LossyWrite) until a preserving codec exists; inspect/diff only"))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no plugin mechanism documented (trae-agent.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{CONFIG_ENV_VAR, HARNESS_ID_STR, TraeAgentAdapter};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> TraeAgentAdapter {
        TraeAgentAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-trae-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::ExplicitConfig,
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
    fn config_surfaces_include_yaml_explicit() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 4);
        let yaml = surfaces
            .iter()
            .find(|s| s.id == "trae_config.yaml")
            .expect("trae_config.yaml");
        assert_eq!(yaml.kind, DocumentKind::Yaml);
        assert_eq!(yaml.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(yaml.scope, ConfigScope::User);
        for sel in ["model_providers", "models"] {
            assert!(yaml.owned_selectors.contains(&sel.to_owned()));
        }
        let legacy = surfaces
            .iter()
            .find(|s| s.id == "trae_config.json")
            .expect("legacy");
        assert_eq!(legacy.kind, DocumentKind::Json);
        let env = surfaces.iter().find(|s| s.id == ".env").expect(".env");
        assert_eq!(env.kind, DocumentKind::Env);
    }

    #[test]
    fn supported_operations_cover_full() {
        let a = adapter();
        let ops = a.supported_operations();
        for (_, support) in &ops {
            assert_eq!(*support, AdapterSupport::Full);
        }
        let names: HashSet<String> = ops.iter().map(|(n, _)| n.clone()).collect();
        for required in ["detect", "read_config", "write_config", "plan_wrapper"] {
            assert!(names.contains(required));
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_trajectories_and_locks() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        assert!(exclusions.contains(&"trajectories/*".to_owned()));
        assert!(exclusions.contains(&"*.lock".to_owned()));
        assert!(!exclusions.contains(&"trae_config.yaml".to_owned()));
    }

    #[test]
    fn plan_wrapper_sets_env_and_args() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("trae-work"));
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars
                .iter()
                .any(|(k, v)| k == CONFIG_ENV_VAR && v.contains("trae-work"))
        );
        assert!(plan.args.contains(&"--config-file".to_owned()));
        let cfg = plan
            .args
            .windows(2)
            .find(|w| w[0] == "--config-file")
            .unwrap()[1]
            .clone();
        assert!(cfg.contains("trae-work"));
        assert!(cfg.contains("trae_config.yaml"));
    }

    #[test]
    fn plan_wrapper_quoting_with_spaces() {
        let root = crate::test_util::tmp_abs_str("my trae work");
        let a = adapter();
        let inst = sample_instance_with_root(&root);
        let plan = a.plan_wrapper(&inst).unwrap();
        let val = plan
            .env_vars
            .iter()
            .find(|(k, _)| k == CONFIG_ENV_VAR)
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert_eq!(
            val,
            std::path::Path::new(&root)
                .join("trae_config.yaml")
                .display()
                .to_string()
        );
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("trae-work"));
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
        assert!(candidates.iter().any(|c| c.contains("trae_config.yaml")));
        assert!(candidates.iter().any(|c| c.contains(CONFIG_ENV_VAR)));
    }

    #[test]
    fn validate_instance_accepts_explicit_config() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("trae-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("trae-work"));
        inst.isolation = Isolation::ProjectScope;
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "isolation"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn validate_instance_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("trae-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/trae_agent")
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
        let path = fixture_path("trae_config.minimal.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(
            map.contains_key("agents")
                || map.contains_key("model_providers")
                || map.contains_key("models")
                || map.len() <= 3
        );
    }

    #[test]
    fn fixture_populated_parses_and_has_expected_keys() {
        let path = fixture_path("trae_config.populated.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(
            map.contains_key("model_providers")
                || map.contains_key("models")
                || map.contains_key("agents")
        );
    }

    #[test]
    fn fixture_foreign_survive_because_changing_edit_refuses() {
        let path = fixture_path("trae_config.foreign.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::yaml::load(&path).unwrap();
        assert!(original.contains_key("foreignKey") || original.contains_key("unknownTopLevel"));
        let dir = crate::test_util::temp_dir_unique("trae");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("trae.foreign.copy.yaml");
        std::fs::copy(&path, &tmp).unwrap();
        // Changing YAML writes on existing files are refused outright, so
        // foreign keys survive because nothing is written.
        let result = superai_config::yaml::edit(&tmp, |map| {
            map.insert(
                "allow_mcp_servers".to_owned(),
                serde_json::Value::Array(vec![]),
            );
            assert!(map.contains_key("foreignKey") || map.contains_key("unknownTopLevel"));
        });
        assert!(matches!(
            result,
            Err(superai_config::ConfigError::LossyWrite { format: "yaml", .. })
        ));
        let after = superai_config::yaml::load(&tmp).unwrap();
        assert!(after.contains_key("foreignKey") || after.contains_key("unknownTopLevel"));
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("trae_config.malformed.yaml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::yaml::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn fixture_env_minimal_parses() {
        let path = fixture_path("env.minimal");
        assert!(path.exists(), "env minimal missing: {}", path.display());
        let map = superai_config::env_file::load(&path).unwrap();
        assert!(map.contains_key("OPENAI_API_KEY"));
    }

    #[test]
    fn fixture_legacy_json_minimal_parses() {
        let path = fixture_path("trae_config.legacy.json");
        assert!(path.exists(), "legacy json missing: {}", path.display());
        let map = superai_config::json::load(&path).unwrap();
        assert!(map.contains_key("default_provider"));
    }
}
