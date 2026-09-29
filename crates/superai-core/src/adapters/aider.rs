//! Aider adapter: explicit-config via `--config`/`--env-file` plus HOME relocation.
//! Research source: `docs/harness-configs/aider.md` (last verified 2026-08-25).

use std::path::{Path, PathBuf};

use superai_config::document::ValueType;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, RestartBehavior, RootShape, SurfaceOwnership,
    SurfaceSchema, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "aider";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Aider";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "aider";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/aider.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Kebab-case keys mirroring long CLI options; other keys round-trip untouched.
pub const OWNED_SELECTORS: &[&str] = &[
    "model",
    "weak-model",
    "editor-model",
    "dark-mode",
    "auto-commits",
    "map-tokens",
    "edit-format",
];

/// `explicit-config` isolation: relocated `HOME` plus `--config`/`--env-file` paths.
#[derive(Debug, Clone)]
pub struct AiderAdapter {
    id: HarnessId,
}

impl AiderAdapter {
    /// Create an adapter, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_home() -> Option<PathBuf> {
        super::home_dir()
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "detection branches are explicit for evidence"
    )]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        if let Some(home) = Self::default_home() {
            let yml = home.join(".aider.conf.yml");
            if yml.exists() {
                evidence.push(format!("yaml config exists at {}", yml.display()));
                match std::fs::read_to_string(&yml) {
                    Ok(text) if text.contains("model:") => {
                        evidence.push("yaml config contains model key".to_owned());
                    }
                    Ok(_) => {}
                    Err(err) => {
                        evidence.push(format!("config unreadable at {}: {err}", yml.display()));
                    }
                }
            } else {
                evidence.push(format!("yaml config missing at {}", yml.display()));
            }

            let env = home.join(".env");
            if env.exists() {
                evidence.push(format!(".env file present at {}", env.display()));
            }

            let metadata = home.join(".aider.model.metadata.json");
            if metadata.exists() {
                evidence.push(format!("model metadata present at {}", metadata.display()));
            }

            let settings = home.join(".aider.model.settings.yml");
            if settings.exists() {
                evidence.push(format!("model settings present at {}", settings.display()));
            }

            let cwd_yml = Path::new(".aider.conf.yml");
            if cwd_yml.exists() {
                evidence.push(format!("cwd yaml config found at {}", cwd_yml.display()));
            }
            let cwd_env = Path::new(".env");
            if cwd_env.exists() {
                evidence.push(format!("cwd .env found at {}", cwd_env.display()));
            }
        } else {
            evidence.push("could not resolve HOME for config lookup".to_owned());
        }
    }
}

impl Default for AiderAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for AiderAdapter {
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

    fn version_resolution(&self) -> VersionResolution {
        super::resolution_from_detection(self.detection(), "aider", SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let yml_resolver = PathResolver::new(
            Some(
                "~/.aider.conf.yml / ./.aider.conf.yml / $GIT_ROOT/.aider.conf.yml (or --config <path>)",
            ),
            Some("~/.aider.conf.yml / ./.aider.conf.yml"),
            Some("%USERPROFILE%\\.aider.conf.yml / .\\.aider.conf.yml"),
            "~/.aider.conf.yml (also ./.aider.conf.yml, --config overrides)",
        );
        let mut yml_surface = ConfigSurface::new(
            ".aider.conf.yml",
            yml_resolver,
            DocumentKind::Yaml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        yml_surface.precedence = 10;
        yml_surface.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        yml_surface.backup_required = true;
        yml_surface.restart_behavior = RestartBehavior::Reload;
        surfaces.push(yml_surface);

        let env_resolver = PathResolver::new(
            Some("~/.env / ./.env / $GIT_ROOT/.env (or --env-file <path>)"),
            Some("~/.env / ./.env"),
            Some("%USERPROFILE%\\.env / .\\.env"),
            "~/.env (also ./.env, --env-file overrides)",
        );
        let mut env_surface = ConfigSurface::new(
            ".env",
            env_resolver,
            DocumentKind::Env,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        env_surface.precedence = 20;
        env_surface.owned_selectors = vec![
            "OPENAI_API_KEY".to_owned(),
            "ANTHROPIC_API_KEY".to_owned(),
            "OPENAI_API_BASE".to_owned(),
        ];
        env_surface.backup_required = true;
        env_surface.restart_behavior = RestartBehavior::Reload;
        surfaces.push(env_surface);

        let settings_resolver = PathResolver::new(
            Some("~/.aider.model.settings.yml / ./.aider.model.settings.yml"),
            Some("~/.aider.model.settings.yml"),
            Some("%USERPROFILE%\\.aider.model.settings.yml"),
            "~/.aider.model.settings.yml (also ./.aider.model.settings.yml, --model-settings-file overrides)",
        );
        let mut settings_surface = ConfigSurface::new(
            ".aider.model.settings.yml",
            settings_resolver,
            DocumentKind::Yaml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        settings_surface.precedence = 15;
        settings_surface.backup_required = true;
        surfaces.push(settings_surface);

        let metadata_resolver = PathResolver::new(
            Some("~/.aider.model.metadata.json / ./.aider.model.metadata.json"),
            Some("~/.aider.model.metadata.json"),
            Some("%USERPROFILE%\\.aider.model.metadata.json"),
            "~/.aider.model.metadata.json",
        );
        let mut metadata_surface = ConfigSurface::new(
            ".aider.model.metadata.json",
            metadata_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        metadata_surface.precedence = 12;
        metadata_surface.backup_required = true;
        surfaces.push(metadata_surface);

        let history_resolver = PathResolver::fallback_only(
            ".aider.chat.history.md (project root or --chat-history-file)",
        );
        let mut history_surface = ConfigSurface::new(
            ".aider.chat.history.md",
            history_resolver,
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        history_surface.precedence = 5;
        history_surface.backup_required = false;
        surfaces.push(history_surface);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::all_operations_full()
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            ".aider.chat.history.md".to_owned(),
            ".aider.input.history".to_owned(),
            ".aider.llm.history".to_owned(),
            "history.jsonl".to_owned(),
            ".aider*history*".to_owned(),
            "cache/*".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "*.lock".to_owned(),
            "logs/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan =
            WrapperPlan::new("explicit-config via --config/--env-file + HOME relocation");
        plan.env_vars
            .push(("HOME".to_owned(), instance.config_root.to_string()));
        let config_path = instance.config_root.as_path().join(".aider.conf.yml");
        let env_path = instance.config_root.as_path().join(".env");
        plan.args.push("--config".to_owned());
        plan.args.push(config_path.display().to_string());
        plan.args.push("--env-file".to_owned());
        plan.args.push(env_path.display().to_string());
        plan.description = format!(
            " Wrapper sets HOME={} and execs `{} --config {}` with `--env-file {}`",
            instance.config_root,
            EXECUTABLE,
            config_path.display(),
            env_path.display()
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.aider.conf.yml".to_owned(),
            "./.aider.conf.yml".to_owned(),
            "~/.aider.model.settings.yml".to_owned(),
            "./.env".to_owned(),
            "~/.env".to_owned(),
            "$HOME/.aider.conf.yml via HOME relocation".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ExplicitConfig | Isolation::RelocatedRoot | Isolation::Unknown => {
                crate::adapter::validate_instance_surfaces(self, instance.config_root.as_path())
            }
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("aider requires isolation explicit_config, got {other}"),
            }),
        }
    }

    fn surface_schema(&self, surface_id: &str) -> Option<SurfaceSchema> {
        match surface_id {
            ".aider.conf.yml" | ".aider.model.settings.yml" => Some(
                SurfaceSchema::new()
                    .with_root_shape(RootShape::Mapping)
                    .with_owned_key("model", ValueType::String)
                    .with_owned_key("weak-model", ValueType::String)
                    .with_owned_key("editor-model", ValueType::String)
                    .with_owned_key("edit-format", ValueType::String)
                    .with_owned_key("dark-mode", ValueType::Boolean)
                    .with_owned_key("auto-commits", ValueType::Boolean)
                    .with_owned_key("map-tokens", ValueType::Number)
                    .with_deprecated(
                        "openai-api-type",
                        Some("env:OPENAI_API_TYPE via --set-env".to_owned()),
                    )
                    .with_deprecated(
                        "openai-api-version",
                        Some("env:OPENAI_API_VERSION via --set-env".to_owned()),
                    )
                    .with_deprecated(
                        "openai-api-deployment-id",
                        Some("env:OPENAI_API_DEPLOYMENT_ID via --set-env".to_owned()),
                    )
                    .with_deprecated(
                        "openai-organization-id",
                        Some("env:OPENAI_ORGANIZATION via --set-env".to_owned()),
                    ),
            ),
            ".env" => Some(
                SurfaceSchema::new()
                    .with_root_shape(RootShape::EnvEntries)
                    .with_owned_key("OPENAI_API_KEY", ValueType::String)
                    .with_owned_key("ANTHROPIC_API_KEY", ValueType::String)
                    .with_owned_key("OPENAI_API_BASE", ValueType::String),
            ),
            ".aider.model.metadata.json" => {
                Some(SurfaceSchema::new().with_root_shape(RootShape::Object))
            }
            _ => None,
        }
    }

    fn capability_declarations(&self) -> Vec<crate::adapter::AdapterCapabilityDecl> {
        vec![
            crate::adapter::AdapterCapabilityDecl::new(
                crate::capability::Capability::WebSearch,
                crate::capability::Support::Absent,
                "no web-search tool in the aider harness",
            ),
            crate::adapter::AdapterCapabilityDecl::new(
                crate::capability::Capability::Vision,
                crate::capability::Support::Absent,
                "no image input transport in the aider harness",
            ),
            crate::adapter::AdapterCapabilityDecl::new(
                crate::capability::Capability::ComputerUse,
                crate::capability::Support::Absent,
                "no computer-use loop in the aider harness",
            ),
            crate::adapter::AdapterCapabilityDecl::new(
                crate::capability::Capability::Mcp,
                crate::capability::Support::Absent,
                "no MCP support in the aider harness",
            ),
        ]
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        vec![crate::adapter::SkillMode::CopySelected]
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some("aider documents no MCP server support (docs/harness-configs/aider.md)")
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("aider documents no plugin mechanism (docs/harness-configs/aider.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{AiderAdapter, HARNESS_ID_STR, OWNED_SELECTORS};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> AiderAdapter {
        AiderAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-aider-1").unwrap(),
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
        match result.present {
            InstallPresence::Absent => {
                assert!(result.version.is_none());
                assert!(!result.evidence.is_empty());
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
    fn config_surfaces_include_yaml_and_env() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 4);
        let yml = surfaces
            .iter()
            .find(|s| s.id == ".aider.conf.yml")
            .expect(".aider.conf.yml surface must exist");
        assert_eq!(yml.kind, DocumentKind::Yaml);
        assert_eq!(yml.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(yml.scope, ConfigScope::User);
        assert!(yml.backup_required);
        for selector in ["model", "dark-mode", "auto-commits"] {
            assert!(
                yml.owned_selectors.contains(&selector.to_owned()),
                "owned_selectors must contain {selector}"
            );
        }
        for sel in &yml.owned_selectors {
            assert!(!sel.is_empty());
        }
        for sel in OWNED_SELECTORS {
            assert!(yml.owned_selectors.contains(&(*sel).to_owned()));
        }

        let env = surfaces
            .iter()
            .find(|s| s.id == ".env")
            .expect(".env surface must exist");
        assert_eq!(env.kind, DocumentKind::Env);
        assert_eq!(env.ownership, SurfaceOwnership::UserEditable);

        let metadata = surfaces
            .iter()
            .find(|s| s.id == ".aider.model.metadata.json")
            .expect("metadata surface");
        assert_eq!(metadata.kind, DocumentKind::Json);

        let history = surfaces
            .iter()
            .find(|s| s.id == ".aider.chat.history.md")
            .expect("history");
        assert_eq!(history.kind, DocumentKind::TextFragment);
    }

    #[test]
    fn supported_operations_cover_full() {
        let a = adapter();
        let ops = a.supported_operations();
        assert!(!ops.is_empty());
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
    fn plan_mirror_exclusions_cover_history_and_locks() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        assert!(!exclusions.is_empty());
        let must_contain = [
            ".aider.chat.history.md",
            ".aider.input.history",
            "cache/*",
            "*.lock",
        ];
        for pat in must_contain {
            assert!(
                exclusions.contains(&pat.to_owned()),
                "exclusions must contain {pat}"
            );
        }
        assert!(!exclusions.contains(&".aider.conf.yml".to_owned()));
    }

    #[test]
    fn plan_mirror_includes_config_and_excludes_history() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        let is_excluded = |file: &str| crate::adapters::exclusion_matches(&exclusions, file);
        assert!(!is_excluded(".aider.conf.yml"));
        assert!(!is_excluded(".aider.model.settings.yml"));
        assert!(is_excluded(".aider.chat.history.md"));
        assert!(is_excluded(".aider.input.history"));
        assert!(is_excluded("cache/data.bin"));
    }

    #[test]
    fn plan_wrapper_sets_home_and_explicit_paths() {
        let tmp_root = crate::test_util::tmp_abs_str(".aider-work");
        let a = adapter();
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars
                .iter()
                .any(|(k, v)| k == "HOME" && v == tmp_root.as_str())
        );
        assert!(plan.args.contains(&"--config".to_owned()));
        assert!(plan.args.contains(&"--env-file".to_owned()));
        let config_arg = plan
            .args
            .windows(2)
            .find(|w| w[0] == "--config")
            .map(|w| w[1].as_str())
            .unwrap();
        assert!(config_arg.contains(".aider-work"));
        assert!(config_arg.contains(".aider.conf.yml"));
        assert!(!plan.description.is_empty());
        assert!(plan.description.contains("HOME"));
    }

    #[test]
    fn plan_wrapper_quoting_with_spaces() {
        let tmp_root = crate::test_util::tmp_abs_str("my aider work");
        let a = adapter();
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        let env_val = plan
            .env_vars
            .iter()
            .find(|(k, _)| k == "HOME")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert_eq!(env_val, tmp_root.as_str());
        assert!(!env_val.contains('"'));
        assert!(env_val.contains(' '));
        // Args with spaces must be preserved verbatim (shell quoting handled by wrapper generator)
        let config_path = plan
            .args
            .windows(2)
            .find(|w| w[0] == "--config")
            .map(|w| w[1].as_str())
            .unwrap();
        assert!(config_path.contains(' '));
        assert!(!config_path.contains('"'));
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".aider-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_home_and_cwd() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.contains(".aider.conf.yml")));
        assert!(candidates.iter().any(|c| c.contains(".env")));
    }

    #[test]
    fn validate_instance_accepts_explicit_config() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".aider-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_accepts_home_relocation_variant() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".aider-work"));
        inst.isolation = Isolation::RelocatedRoot;
        a.validate_instance(&inst).unwrap();
        inst.isolation = Isolation::Unknown;
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".aider-work"));
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
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".aider-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    #[test]
    fn path_resolution_resolver_fallbacks() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        let yml = surfaces.iter().find(|s| s.id == ".aider.conf.yml").unwrap();
        assert!(yml.path_resolver.fallback.contains(".aider.conf.yml"));
        assert!(yml.path_resolver.linux.is_some());
        let env = surfaces.iter().find(|s| s.id == ".env").unwrap();
        assert!(env.path_resolver.fallback.contains(".env"));
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/aider")
    }

    fn fixture_path(name: &str) -> PathBuf {
        fixtures_root().join(name)
    }

    #[test]
    fn fixture_missing_file_loads_as_empty() {
        let path = fixture_path("nonexistent.yml");
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(map.is_empty());
        let value = superai_config::yaml::load_value(&path).unwrap();
        assert_eq!(value, serde_json::Value::Object(serde_json::Map::default()));
    }

    #[test]
    fn fixture_minimal_parses() {
        let path = fixture_path("aider.minimal.yml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(map.is_empty() || map.contains_key("model") || map.len() <= 2);
    }

    #[test]
    fn fixture_populated_parses_and_has_expected_keys() {
        let path = fixture_path("aider.populated.yml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::yaml::load(&path).unwrap();
        assert!(
            map.contains_key("model")
                || map.contains_key("dark-mode")
                || map.contains_key("auto-commits")
        );
        if let Some(v) = map.get("model") {
            assert!(v.is_string());
        }
    }

    #[test]
    fn fixture_foreign_preserves_unknown_keys_on_edit() {
        let path = fixture_path("aider.foreign.yml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::yaml::load(&path).unwrap();
        assert!(
            original.contains_key("foreignKey")
                || original.contains_key("unknownTopLevel")
                || original.contains_key("customField")
        );
        let dir = crate::test_util::temp_dir_unique("aider");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("aider.foreign.copy.yml");
        std::fs::copy(&path, &tmp).unwrap();
        superai_config::yaml::edit(&tmp, |map| {
            map.insert(
                "model".to_owned(),
                serde_json::Value::String("gpt-4".to_owned()),
            );
            assert!(
                map.contains_key("foreignKey")
                    || map.contains_key("unknownTopLevel")
                    || map.contains_key("customField")
            );
        })
        .unwrap();
        let after = superai_config::yaml::load(&tmp).unwrap();
        assert_eq!(
            after["model"],
            serde_json::Value::String("gpt-4".to_owned())
        );
        let foreign_preserved = after.contains_key("foreignKey")
            || after.contains_key("unknownTopLevel")
            || after.contains_key("customField");
        assert!(
            foreign_preserved,
            "foreign keys must be preserved, got {after:?}"
        );
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("aider.malformed.yml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::yaml::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn fixture_env_minimal_parses() {
        let path = fixture_path(".env.minimal");
        assert!(
            path.exists(),
            "env minimal fixture missing: {}",
            path.display()
        );
        let map = superai_config::env_file::load(&path).unwrap();
        assert!(map.contains_key("OPENAI_API_KEY"));
    }

    #[test]
    fn fixture_env_populated_has_keys() {
        let path = fixture_path(".env.populated");
        assert!(
            path.exists(),
            "env populated fixture missing: {}",
            path.display()
        );
        let map = superai_config::env_file::load(&path).unwrap();
        assert!(
            map.contains_key("OPENAI_API_KEY")
                || map.contains_key("OPENAI_API_BASE")
                || map.contains_key("ANTHROPIC_API_KEY")
        );
    }

    #[test]
    fn fixture_json_metadata_populated_parses() {
        let path = fixture_path("model.metadata.populated.json");
        assert!(
            path.exists(),
            "json metadata fixture missing: {}",
            path.display()
        );
        let map = superai_config::json::load(&path).unwrap();
        assert!(!map.is_empty());
    }

    #[test]
    fn unknown_keys_survive_because_changing_yaml_edit_refuses() {
        let dir = crate::test_util::temp_dir_unique("aider");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("preserve.yml");
        let original = "model: gpt-4\nforeignKey: keep-me\ncustomField: 123\n";
        std::fs::write(&path, original).unwrap();

        // codec-honesty (DOC-06): changing YAML writes on existing files are
        // refused outright; preservation is expressed by refusing.
        let result = superai_config::yaml::edit(&path, |map| {
            map.insert(
                "model".to_owned(),
                serde_json::Value::String("gpt-5".to_owned()),
            );
        });
        match result {
            Err(superai_config::ConfigError::LossyWrite { format, .. }) => {
                assert_eq!(format, "yaml");
            }
            other => panic!("expected LossyWrite, got {other:?}"),
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        let after = superai_config::yaml::load(&path).unwrap();
        assert_eq!(
            after["foreignKey"],
            serde_json::Value::String("keep-me".to_owned())
        );
        assert_eq!(after["customField"], serde_json::Value::Number(123.into()));
        drop(std::fs::remove_file(&path));
    }

    #[test]
    fn provider_mutation_sets_model_and_env() {
        let dir = crate::test_util::temp_dir_unique("aider");
        std::fs::create_dir_all(&dir).unwrap();
        let yml_path = dir.join("aider.yml");
        let env_path = dir.join(".env");
        std::fs::write(&yml_path, "model: gpt-4\ndark-mode: true\n").unwrap();
        std::fs::write(&env_path, "OPENAI_API_KEY=sk-old\n").unwrap();

        // codec-honesty (DOC-06): the YAML leg refuses; the env leg is a
        // preserving codec and still succeeds.
        let yml_result = superai_config::yaml::edit(&yml_path, |map| {
            map.insert(
                "model".to_owned(),
                serde_json::Value::String("openrouter/anthropic/claude-sonnet-4".to_owned()),
            );
        });
        match yml_result {
            Err(superai_config::ConfigError::LossyWrite { format, .. }) => {
                assert_eq!(format, "yaml");
            }
            other => panic!("expected LossyWrite, got {other:?}"),
        }
        superai_config::env_file::edit(&env_path, |map| {
            map.insert("OPENAI_API_KEY".to_owned(), "sk-new".to_owned());
            map.insert(
                "OPENAI_API_BASE".to_owned(),
                "https://api.openrouter.ai/v1".to_owned(),
            );
        })
        .unwrap();

        let after_yml = superai_config::yaml::load(&yml_path).unwrap();
        assert_eq!(
            after_yml["model"],
            serde_json::Value::String("gpt-4".to_owned())
        );
        let after_env = superai_config::env_file::load(&env_path).unwrap();
        assert_eq!(after_env["OPENAI_API_KEY"], "sk-new");
        assert_eq!(after_env["OPENAI_API_BASE"], "https://api.openrouter.ai/v1");
        drop(std::fs::remove_file(&yml_path));
        drop(std::fs::remove_file(&env_path));
    }

    #[test]
    fn wrapper_env_var_isolation_is_explicit() {
        let tmp_root = crate::test_util::tmp_abs_str("user/.aider-isolated");
        let a = adapter();
        assert!(a.scan_candidates().len() >= 3);
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(!plan.env_vars.is_empty());
        let (key, val) = &plan.env_vars[0];
        assert_eq!(key, "HOME");
        assert_eq!(val, tmp_root.as_str());
        assert!(plan.args.contains(&"--config".to_owned()));
    }

    #[test]
    fn yaml_comment_changing_write_refused_and_comments_preserved() {
        // codec-honesty (DOC-06): comments parse on read, but a changing write
        // on a comment-bearing YAML file is refused; the bytes survive verbatim.
        let dir = crate::test_util::temp_dir_unique("aider");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("comment.yml");
        let content = "# Aider config\nmodel: gpt-4 # inline comment\ndark-mode: true\n";
        std::fs::write(&path, content).unwrap();
        let map = superai_config::yaml::load(&path).unwrap();
        assert_eq!(map["model"], serde_json::Value::String("gpt-4".to_owned()));
        assert_eq!(map["dark-mode"], serde_json::Value::Bool(true));
        let result = superai_config::yaml::edit(&path, |m| {
            m.insert(
                "model".to_owned(),
                serde_json::Value::String("gpt-5".to_owned()),
            );
        });
        match result {
            Err(superai_config::ConfigError::LossyWrite { format, .. }) => {
                assert_eq!(format, "yaml");
            }
            other => panic!("expected LossyWrite, got {other:?}"),
        }
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            content,
            "refused edit must leave the file byte-identical"
        );
        drop(std::fs::remove_file(&path));
    }

    #[test]
    fn adapter_is_object_safe() {
        let a = adapter();
        let boxed: Box<dyn Adapter> = Box::new(a);
        assert_eq!(boxed.id().as_str(), HARNESS_ID_STR);
        assert!(!boxed.config_surfaces().is_empty());
        assert!(!boxed.plan_mirror_exclusions().is_empty());
    }

    #[test]
    fn surface_schema_declares_yaml_env_and_metadata_shapes() {
        let a = adapter();
        let yml = a.surface_schema(".aider.conf.yml").expect("yml schema");
        assert_eq!(yml.root_shape, Some(crate::adapter::RootShape::Mapping));
        assert!(yml.owned_key_rules.iter().any(|r| r.path == "model"));
        assert_eq!(yml.deprecated_keys.len(), 4);
        let env = a.surface_schema(".env").expect("env schema");
        assert_eq!(env.root_shape, Some(crate::adapter::RootShape::EnvEntries));
        assert!(a.surface_schema(".aider.model.metadata.json").is_some());
        assert!(a.surface_schema("unknown").is_none());
    }

    #[test]
    fn schema_rejects_wrongly_typed_yaml_key() {
        let diags = crate::adapter::validate_surface_content(
            &adapter(),
            ".aider.conf.yml",
            b"dark-mode: \"yes\"\n",
            superai_config::document::DocumentKind::Yaml,
        );
        assert!(
            diags.iter().any(|d| d
                .message
                .contains("`dark-mode` must hold a value of type boolean")),
            "diags: {diags:?}"
        );
        // Absent owned keys stay legal (minimal config).
        let ok = crate::adapter::validate_surface_content(
            &adapter(),
            ".aider.conf.yml",
            b"other-key: 1\n",
            superai_config::document::DocumentKind::Yaml,
        );
        assert!(ok.is_empty(), "{ok:?}");
    }

    #[test]
    fn schema_flags_deprecated_legacy_openai_switches() {
        let diags = crate::adapter::validate_surface_content(
            &adapter(),
            ".aider.conf.yml",
            b"model: gpt-4\nopenai-api-type: azure\n",
            superai_config::document::DocumentKind::Yaml,
        );
        let dep = diags
            .iter()
            .find(|d| d.severity == superai_config::document::DiagnosticSeverity::Deprecation)
            .expect("deprecation diagnostic");
        assert!(dep.message.contains("`openai-api-type` is deprecated"));
        assert!(dep.message.contains("OPENAI_API_TYPE"));
    }

    #[test]
    fn validate_instance_rejects_schema_invalid_config_under_root() {
        let a = adapter();
        let dir = crate::test_util::temp_dir_unique("aider-schema");
        std::fs::create_dir_all(&dir).unwrap();
        let inst = sample_instance_with_root(dir.to_str().unwrap());
        a.validate_instance(&inst).unwrap();
        std::fs::write(dir.join(".aider.conf.yml"), b"model: gpt-4\n").unwrap();
        a.validate_instance(&inst).unwrap();
        // auto-commits is a boolean flag; a string violates the schema.
        std::fs::write(dir.join(".aider.conf.yml"), b"auto-commits: \"yes\"\n").unwrap();
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::SchemaValidation { details, .. } => {
                assert!(details.contains("[aider/.aider.conf.yml]"), "{details}");
            }
            other => panic!("expected SchemaValidation, got {other:?}"),
        }
        drop(std::fs::remove_dir_all(&dir));
    }

    #[test]
    fn boundary_fixtures_split_model_metadata_eras() {
        let legacy = fixture_path("model.metadata.boundary_legacy.json");
        let current = fixture_path("model.metadata.boundary_current.json");
        assert!(legacy.exists(), "missing {}", legacy.display());
        assert!(current.exists(), "missing {}", current.display());

        let legacy_map = superai_config::json::load(&legacy).unwrap();
        let current_map = superai_config::json::load(&current).unwrap();
        assert!(legacy_map.contains_key("openai/gpt-4"));
        assert!(!legacy_map.contains_key("openrouter/anthropic/claude-sonnet-4"));
        assert!(current_map.contains_key("openrouter/anthropic/claude-sonnet-4"));

        for path in [&legacy, &current] {
            let content = std::fs::read(path).unwrap();
            let diags = crate::adapter::validate_surface_content(
                &adapter(),
                ".aider.model.metadata.json",
                &content,
                superai_config::document::DocumentKind::StrictJson,
            );
            assert!(
                diags
                    .iter()
                    .all(|d| d.severity != superai_config::document::DiagnosticSeverity::Error),
                "boundary fixture must satisfy the declared schema: {diags:?}"
            );
        }
    }

    #[test]
    fn boundary_version_resolution_is_compatible_for_recorded_detection() {
        // version.txt records `aider 0.84.0`, inside the compatible range.
        let version_text = std::fs::read_to_string(fixture_path("version.txt"))
            .unwrap()
            .trim()
            .to_owned();
        let parsed = crate::adapters::parse_version_output(&version_text);
        assert_eq!(parsed.as_deref(), Some("0.84.0"));
        let res = adapter().version_resolution();
        if res.detected_version.as_deref() == parsed.as_deref() {
            assert!(res.compatible);
            assert_eq!(
                res.schema_version.as_deref(),
                Some(super::SCHEMA_VERSION_STR)
            );
        } else {
            assert!(!res.compatible, "unknown versions must block writes");
        }
    }
}
