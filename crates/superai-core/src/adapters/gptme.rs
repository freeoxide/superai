//! gptme adapter: workspace via `GPTME_WORKSPACE`/`--workspace`, global TOML
//! `~/.config/gptme/config.toml` plus project `gptme.toml`; cloud service excluded.

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
pub const HARNESS_ID_STR: &str = "gptme";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "gptme";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "gptme";

/// Env var carrying the isolated workspace root.
pub const WORKSPACE_ENV_VAR: &str = "GPTME_WORKSPACE";

/// Logs home env var.
pub const LOGS_ENV_VAR: &str = "GPTME_LOGS_HOME";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/gptme.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors for gptme inside `config.toml` TOML and `gptme.toml`.
pub const OWNED_SELECTORS: &[&str] = &[
    "env",
    "env.MODEL",
    "env.OPENAI_API_KEY",
    "env.ANTHROPIC_API_KEY",
    "models",
    "models.default",
    "prompt",
    "mcp",
    "settings",
    "settings.gear",
];

/// Concrete adapter for gptme.
#[derive(Debug, Clone)]
pub struct GptmeAdapter {
    id: HarnessId,
}

impl GptmeAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME")
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir).join("gptme"));
        }
        let home = super::home_dir()?;
        Some(home.join(".config").join("gptme"))
    }

    fn config_path_for_root(root: &Path) -> PathBuf {
        root.join("config.toml")
    }

    #[expect(clippy::excessive_nesting, reason = "detection branches are explicit")]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        match Self::default_config_root() {
            Some(root) => {
                if root.exists() {
                    evidence.push(format!("config root exists at {}", root.display()));
                    let cfg = Self::config_path_for_root(&root);
                    if cfg.exists() {
                        evidence.push(format!("config.toml found at {}", cfg.display()));
                        match std::fs::read_to_string(&cfg) {
                            Ok(text) if text.contains("[env]") => {
                                evidence.push("config.toml contains [env]".to_owned());
                            }
                            Ok(_) => {}
                            Err(err) => evidence
                                .push(format!("config unreadable at {}: {err}", cfg.display())),
                        }
                    } else {
                        evidence.push(format!("config.toml missing at {}", cfg.display()));
                    }
                    if root.join("config.local.toml").exists() {
                        evidence.push("config.local.toml present".to_owned());
                    }
                    if root.join("credentials.toml").exists() {
                        evidence.push("credentials.toml present".to_owned());
                    }
                } else {
                    evidence.push(format!("config root missing at {}", root.display()));
                }
            }
            None => {
                evidence.push("could not resolve default config root (no HOME)".to_owned());
            }
        }
        if let Ok(ws) = std::env::var(WORKSPACE_ENV_VAR)
            && !ws.trim().is_empty()
        {
            evidence.push(format!("{WORKSPACE_ENV_VAR} set to {ws}"));
        } else {
            evidence.push(format!("{WORKSPACE_ENV_VAR} not set, using cwd workspace"));
        }
        if Path::new("gptme.toml").exists() {
            evidence.push("project gptme.toml found".to_owned());
        }
        if let Ok(logs) = std::env::var(LOGS_ENV_VAR)
            && !logs.trim().is_empty()
        {
            evidence.push(format!("{LOGS_ENV_VAR} set to {logs}"));
        }
        let log_root = Self::default_log_root();
        if let Some(p) = log_root
            && p.exists()
        {
            evidence.push(format!("logs root exists at {}", p.display()));
        }
    }

    fn default_log_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(LOGS_ENV_VAR)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir));
        }
        let home = super::home_dir()?;
        Some(home.join(".local").join("share").join("gptme").join("logs"))
    }
}

impl Default for GptmeAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for GptmeAdapter {
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
        self.version_resolution_from(&self.detection())
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection.clone(), "gptme", SCHEMA_VERSION_STR)
    }

    #[expect(clippy::too_many_lines, reason = "surfaces are declarative")]
    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let global_resolver = PathResolver::new(
            Some("$XDG_CONFIG_HOME/gptme/config.toml"),
            Some("$XDG_CONFIG_HOME/gptme/config.toml"),
            Some("%USERPROFILE%\\.config\\gptme\\config.toml"),
            "~/.config/gptme/config.toml",
        );
        let mut global = ConfigSurface::new(
            "config.toml",
            global_resolver,
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        global.precedence = 10;
        global.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        global.backup_required = true;
        global.restart_behavior = RestartBehavior::Reload;
        surfaces.push(global);

        let local_resolver = PathResolver::new(
            Some("$XDG_CONFIG_HOME/gptme/config.local.toml"),
            Some("$XDG_CONFIG_HOME/gptme/config.local.toml"),
            Some("%USERPROFILE%\\.config\\gptme\\config.local.toml"),
            "~/.config/gptme/config.local.toml",
        );
        let mut local = ConfigSurface::new(
            "config.local.toml",
            local_resolver,
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        local.precedence = 11;
        local.backup_required = false;
        surfaces.push(local);

        let project_resolver = PathResolver::new(
            Some("gptme.toml"),
            Some("gptme.toml"),
            Some("gptme.toml"),
            "gptme.toml (workspace root)",
        );
        let mut project = ConfigSurface::new(
            "gptme.toml",
            project_resolver,
            DocumentKind::Toml,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project.precedence = 12;
        project.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        project.backup_required = true;
        surfaces.push(project);

        let project_local_resolver = PathResolver::new(
            Some("gptme.local.toml"),
            Some("gptme.local.toml"),
            Some("gptme.local.toml"),
            "gptme.local.toml (workspace secrets overlay)",
        );
        let mut project_local = ConfigSurface::new(
            "gptme.local.toml",
            project_local_resolver,
            DocumentKind::Toml,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::ExternalSecretStore,
        );
        project_local.precedence = 13;
        surfaces.push(project_local);

        let env_resolver = PathResolver::new(
            Some("~/.config/gptme/.env"),
            Some("~/.config/gptme/.env"),
            Some("%USERPROFILE%\\.config\\gptme\\.env"),
            "~/.config/gptme/.env (fallback env overlay)",
        );
        let mut env = ConfigSurface::new(
            ".env",
            env_resolver,
            DocumentKind::Env,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        env.precedence = 5;
        surfaces.push(env);

        let logs_resolver = PathResolver::new(
            Some("~/.local/share/gptme/logs/<conversation>/config.toml"),
            Some("~/.local/share/gptme/logs/<conversation>/config.toml"),
            Some("%USERPROFILE%\\.local\\share\\gptme\\logs\\<conversation>\\config.toml"),
            "~/.local/share/gptme/logs/<conversation>/config.toml",
        );
        let mut logs = ConfigSurface::new(
            "logs",
            logs_resolver,
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::HarnessManaged,
        );
        logs.precedence = 0;
        logs.backup_required = false;
        surfaces.push(logs);

        let skills_resolver = PathResolver::new(
            Some("~/.config/gptme/skills/<name>/SKILL.md"),
            Some("~/.config/gptme/skills/<name>/SKILL.md"),
            Some("%USERPROFILE%\\.config\\gptme\\skills\\<name>\\SKILL.md"),
            "~/.config/gptme/skills/<name>/SKILL.md",
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
            "logs/*".to_owned(),
            "logs/**/*".to_owned(),
            "cache/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "tmp/*".to_owned(),
            "*.lock".to_owned(),
            "history/*".to_owned(),
        ]
    }

    /// `config.toml` embeds absolute config paths (plugin `paths`, gptme.md §1.1),
    /// so a mirrored copy needs its config-root references rewritten.
    fn mirror_content_rewrite_files(&self) -> Vec<String> {
        vec!["config.toml".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("workspace plus explicit via GPTME_WORKSPACE/--workspace");
        let workspace = instance.config_root.to_string();
        plan.description = format!(
            " Wrapper sets {WORKSPACE_ENV_VAR}={workspace} and execs `{EXECUTABLE} --workspace {workspace}` (cloud service excluded)"
        );
        plan.env_vars
            .push((WORKSPACE_ENV_VAR.to_owned(), workspace.clone()));
        plan.env_vars
            .push((LOGS_ENV_VAR.to_owned(), format!("{workspace}/logs")));
        plan.args.push("--workspace".to_owned());
        plan.args.push(workspace);
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.config/gptme/config.toml".to_owned(),
            "~/.config/gptme/config.local.toml".to_owned(),
            "gptme.toml".to_owned(),
            "gptme.local.toml".to_owned(),
            "$GPTME_WORKSPACE".to_owned(),
            "~/.local/share/gptme/logs".to_owned(),
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
                    "gptme requires isolation project_scope or explicit_config, got {other}"
                ),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "a config.toml [mcp] section is documented but the per-server schema is unverified in corpus (gptme.md points at gptme.org/docs/mcp.html, not fetched)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "python plugin system; config carries path lists (plugins.paths/enabled), no file-staged plugin record documented (gptme.md)",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{GptmeAdapter, HARNESS_ID_STR, WORKSPACE_ENV_VAR};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> GptmeAdapter {
        GptmeAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-gptme-1").unwrap(),
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
    fn config_surfaces_include_toml_and_workspace() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 5);
        let global = surfaces
            .iter()
            .find(|s| s.id == "config.toml")
            .expect("config.toml");
        assert_eq!(global.kind, DocumentKind::Toml);
        assert_eq!(global.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(global.scope, ConfigScope::User);
        for sel in ["env", "models.default"] {
            assert!(global.owned_selectors.contains(&sel.to_owned()));
        }
        let project = surfaces
            .iter()
            .find(|s| s.id == "gptme.toml")
            .expect("gptme.toml");
        assert_eq!(project.kind, DocumentKind::Toml);
        assert_eq!(project.scope, ConfigScope::ProjectWorkspace);
        let logs = surfaces.iter().find(|s| s.id == "logs").expect("logs");
        assert_eq!(logs.ownership, SurfaceOwnership::HarnessManaged);
    }

    #[test]
    fn supported_operations_are_constrained() {
        let a = adapter();
        let ops = a.supported_operations();
        for (_, support) in &ops {
            assert_eq!(*support, AdapterSupport::Constrained);
        }
    }

    #[test]
    fn plan_mirror_exclusions_cover_logs_and_locks() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        assert!(exclusions.contains(&"logs/*".to_owned()));
        assert!(exclusions.contains(&"*.lock".to_owned()));
    }

    #[test]
    fn plan_wrapper_sets_workspace() {
        let tmp_root = crate::test_util::tmp_abs_str("gptme-work");
        let a = adapter();
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars
                .iter()
                .any(|(k, v)| k == WORKSPACE_ENV_VAR && v == tmp_root.as_str())
        );
        assert!(plan.args.contains(&"--workspace".to_owned()));
        let ws = plan
            .args
            .windows(2)
            .find(|w| w[0] == "--workspace")
            .unwrap()[1]
            .clone();
        assert_eq!(ws, tmp_root.as_str());
        assert!(plan.description.contains(WORKSPACE_ENV_VAR));
    }

    #[test]
    fn plan_wrapper_quoting_with_spaces() {
        let tmp_root = crate::test_util::tmp_abs_str("my gptme work");
        let a = adapter();
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        let val = plan
            .env_vars
            .iter()
            .find(|(k, _)| k == WORKSPACE_ENV_VAR)
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert_eq!(val, tmp_root.as_str());
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("gptme-work"));
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
        assert!(candidates.iter().any(|c| c.contains("gptme")));
        assert!(candidates.iter().any(|c| c.contains(WORKSPACE_ENV_VAR)));
    }

    #[test]
    fn validate_instance_accepts_project_scope() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("gptme-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("gptme-work"));
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
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str("gptme-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/gptme")
    }

    fn fixture_path(name: &str) -> PathBuf {
        fixtures_root().join(name)
    }

    #[test]
    fn fixture_missing_file_loads_as_empty() {
        let path = fixture_path("nonexistent.toml");
        let doc = superai_config::toml_file::load(&path).unwrap();
        assert!(doc.is_empty());
    }

    #[test]
    fn fixture_minimal_parses() {
        let path = fixture_path("config.minimal.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::toml_file::load(&path).unwrap();
        assert!(
            map.is_empty()
                || map.contains_key("settings")
                || map.contains_key("env")
                || map.len() <= 2
        );
    }

    #[test]
    fn fixture_populated_parses_and_has_expected_keys() {
        let path = fixture_path("config.populated.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let map = superai_config::toml_file::load(&path).unwrap();
        assert!(
            map.contains_key("env") || map.contains_key("models") || map.contains_key("prompt")
        );
    }

    #[test]
    fn fixture_project_gptme_minimal_parses() {
        let path = fixture_path("gptme.minimal.toml");
        assert!(path.exists(), "project minimal missing: {}", path.display());
        let map = superai_config::toml_file::load(&path).unwrap();
        assert!(map.is_empty() || map.contains_key("prompt") || map.contains_key("files"));
    }

    #[test]
    fn fixture_foreign_preserves_unknown_keys_on_edit() {
        let path = fixture_path("config.foreign.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::toml_file::load(&path).unwrap();
        assert!(
            original.contains_key("foreignKey")
                || original.contains_key("unknownTopLevel")
                || original.contains_key("customField")
        );
        let dir = crate::test_util::temp_dir_unique("gptme");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("gptme.foreign.copy.toml");
        std::fs::copy(&path, &tmp).unwrap();
        superai_config::toml_file::edit(&tmp, |doc| {
            doc["settings"] = toml_edit::table();
            doc["settings"]["gear"] = toml_edit::value(2);
            assert!(
                doc.to_string().contains("foreignKey")
                    || doc.to_string().contains("unknownTopLevel")
                    || doc.to_string().contains("customField")
            );
        })
        .unwrap();
        let after = superai_config::toml_file::load(&tmp).unwrap();
        assert!(
            after.contains_key("foreignKey")
                || after.contains_key("unknownTopLevel")
                || after.contains_key("customField")
        );
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("config.malformed.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::toml_file::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn fixture_env_minimal_parses() {
        let path = fixture_path("env.minimal");
        assert!(path.exists(), "env minimal missing: {}", path.display());
        let map = superai_config::env_file::load(&path).unwrap();
        assert!(map.contains_key("OPENAI_API_KEY"));
    }
}
