//! Forge adapter: `forge` binary, config root `~/.forge` or `$FORGE_CONFIG`, writable `.forge.toml`.
//! Research source: `docs/harness-configs/forge.md` (last verified 2026-08-25).

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
pub const HARNESS_ID_STR: &str = "forge";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Forge";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "forge";

/// Environment variable that relocates the config root.
pub const CONFIG_ENV_VAR: &str = "FORGE_CONFIG";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/forge.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors for provider/model mutation inside `.forge.toml`.
pub const OWNED_SELECTORS: &[&str] = &[
    "providers",
    "session",
    "session.provider_id",
    "session.model_id",
    "model",
    "retry",
    "http",
    "compact",
    "restricted",
];

/// Concrete adapter for Forge.
#[derive(Debug, Clone)]
pub struct ForgeAdapter {
    id: HarnessId,
}

impl ForgeAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(CONFIG_ENV_VAR)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir));
        }
        let home = super::home_dir()?;
        Some(home.join(".forge"))
    }

    fn config_path_for_root(root: &Path) -> PathBuf {
        root.join(".forge.toml")
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
                    let cfg = Self::config_path_for_root(&root);
                    if cfg.exists() {
                        evidence.push(format!(".forge.toml found at {}", cfg.display()));
                        match std::fs::read_to_string(&cfg) {
                            Ok(text)
                                if (text.contains("providers") || text.contains("[session")) =>
                            {
                                evidence.push(".forge.toml contains providers/session".to_owned());
                            }
                            Ok(_) => {}
                            Err(err) => evidence
                                .push(format!("config unreadable at {}: {err}", cfg.display())),
                        }
                    } else {
                        evidence.push(format!(".forge.toml missing at {}", cfg.display()));
                    }
                    let perms = root.join("permissions.yaml");
                    if perms.exists() {
                        evidence.push(format!("permissions.yaml present at {}", perms.display()));
                    }
                    let mcp = root.join(".mcp.json");
                    if mcp.exists() {
                        evidence.push(format!(".mcp.json present at {}", mcp.display()));
                    }
                } else {
                    evidence.push(format!("config root missing at {}", root.display()));
                }
            }
            None => {
                evidence.push("could not resolve default config root (no HOME)".to_owned());
            }
        }
        if let Ok(dir) = std::env::var(CONFIG_ENV_VAR)
            && !dir.trim().is_empty()
        {
            evidence.push(format!("{CONFIG_ENV_VAR} set to {dir}"));
        } else {
            evidence.push(format!("{CONFIG_ENV_VAR} not set, using ~/.forge"));
        }
        let project_cfg = Path::new("forge.yaml");
        if project_cfg.exists() {
            evidence.push(format!(
                "project forge.yaml found at {}",
                project_cfg.display()
            ));
        }
        if Path::new(".forge").exists() {
            evidence.push(".forge directory present in cwd".to_owned());
        }
    }
}

impl Default for ForgeAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ForgeAdapter {
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
        super::resolution_from_detection(detection, "forge", SCHEMA_VERSION_STR)
    }

    #[expect(clippy::too_many_lines, reason = "surfaces are declarative")]
    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let forge_resolver = PathResolver::new(
            Some("$FORGE_CONFIG/.forge.toml"),
            Some("$FORGE_CONFIG/.forge.toml"),
            Some("%FORGE_CONFIG%\\.forge.toml"),
            "~/.forge/.forge.toml",
        );
        let mut forge_surface = ConfigSurface::new(
            ".forge.toml",
            forge_resolver,
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        forge_surface.precedence = 10;
        forge_surface.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        forge_surface.backup_required = true;
        forge_surface.restart_behavior = RestartBehavior::Reload;
        surfaces.push(forge_surface);

        let perms_resolver = PathResolver::new(
            Some("$FORGE_CONFIG/permissions.yaml"),
            Some("$FORGE_CONFIG/permissions.yaml"),
            Some("%FORGE_CONFIG%\\permissions.yaml"),
            "~/.forge/permissions.yaml",
        );
        let mut perms_surface = ConfigSurface::new(
            "permissions.yaml",
            perms_resolver,
            DocumentKind::Yaml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        perms_surface.precedence = 8;
        perms_surface.backup_required = true;
        surfaces.push(perms_surface);

        let mcp_resolver = PathResolver::new(
            Some("$FORGE_CONFIG/.mcp.json"),
            Some("$FORGE_CONFIG/.mcp.json"),
            Some("%FORGE_CONFIG%\\.mcp.json"),
            "~/.forge/.mcp.json",
        );
        let mut mcp_surface = ConfigSurface::new(
            ".mcp.json",
            mcp_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        mcp_surface.precedence = 12;
        mcp_surface.owned_selectors = vec!["mcpServers".to_owned()];
        mcp_surface.backup_required = true;
        surfaces.push(mcp_surface);

        let project_forge_yaml_resolver = PathResolver::fallback_only("forge.yaml (project)");
        let mut project_yaml = ConfigSurface::new(
            "forge.yaml",
            project_forge_yaml_resolver,
            DocumentKind::Yaml,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project_yaml.precedence = 15;
        project_yaml.backup_required = false;
        surfaces.push(project_yaml);

        let project_mcp_resolver = PathResolver::fallback_only(".mcp.json (project)");
        let mut project_mcp = ConfigSurface::new(
            "project.mcp.json",
            project_mcp_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project_mcp.precedence = 14;
        project_mcp.backup_required = false;
        surfaces.push(project_mcp);

        let skills_resolver = PathResolver::new(
            Some("$FORGE_CONFIG/skills/<name>/SKILL.md"),
            Some("$FORGE_CONFIG/skills/<name>/SKILL.md"),
            Some("%FORGE_CONFIG%\\skills\\<name>\\SKILL.md"),
            "~/.forge/skills/<name>/SKILL.md",
        );
        let mut skills = ConfigSurface::new(
            "skills",
            skills_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        skills.precedence = 7;
        skills.backup_required = false;
        surfaces.push(skills);

        let agents_resolver = PathResolver::new(
            Some("$FORGE_CONFIG/agents/<name>.md"),
            Some("$FORGE_CONFIG/agents/<name>.md"),
            Some("%FORGE_CONFIG%\\agents\\<name>.md"),
            "~/.forge/agents/<name>.md",
        );
        let mut agents = ConfigSurface::new(
            "agents",
            agents_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        agents.precedence = 7;
        agents.backup_required = false;
        surfaces.push(agents);

        let commands_resolver = PathResolver::new(
            Some("$FORGE_CONFIG/commands/<name>.md"),
            Some("$FORGE_CONFIG/commands/<name>.md"),
            Some("%FORGE_CONFIG%\\commands\\<name>.md"),
            "~/.forge/commands/<name>.md",
        );
        let mut commands = ConfigSurface::new(
            "commands",
            commands_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        commands.precedence = 6;
        commands.backup_required = false;
        surfaces.push(commands);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::all_operations_full()
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "sessions/*".to_owned(),
            "logs/*".to_owned(),
            "cache/*".to_owned(),
            "tmp/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "*.lock".to_owned(),
            "telemetry/*".to_owned(),
            "state/*".to_owned(),
            "data/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("relocated config via FORGE_CONFIG");
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.description = format!(
            " Wrapper sets {}={} and execs `{}`",
            CONFIG_ENV_VAR, instance.config_root, EXECUTABLE
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.forge".to_owned(),
            "~/.forge-work".to_owned(),
            "~/.forge-personal".to_owned(),
            "$FORGE_CONFIG".to_owned(),
            "forge.yaml".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("forge requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            ".mcp.json",
            "mcpServers",
            DocumentKind::Json,
            ConfigScope::User,
            RestartBehavior::Reload,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no plugin mechanism documented (forge.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::{CONFIG_ENV_VAR, ForgeAdapter, HARNESS_ID_STR, OWNED_SELECTORS};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ForgeAdapter {
        ForgeAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-forge-1").unwrap(),
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
    fn config_surfaces_include_writable_toml_and_mcp() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 5);
        let forge_toml = surfaces
            .iter()
            .find(|s| s.id == ".forge.toml")
            .expect(".forge.toml surface must exist");
        assert_eq!(forge_toml.kind, DocumentKind::Toml);
        assert_eq!(forge_toml.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(forge_toml.scope, ConfigScope::User);
        assert!(forge_toml.backup_required);
        for selector in ["providers", "session"] {
            assert!(
                forge_toml.owned_selectors.contains(&selector.to_owned()),
                "owned_selectors must contain {selector}"
            );
        }
        for sel in &forge_toml.owned_selectors {
            assert!(!sel.is_empty());
        }
        for sel in OWNED_SELECTORS {
            assert!(forge_toml.owned_selectors.contains(&(*sel).to_owned()));
        }

        let mcp = surfaces
            .iter()
            .find(|s| s.id == ".mcp.json")
            .expect(".mcp.json");
        assert_eq!(mcp.kind, DocumentKind::Json);

        let perms = surfaces
            .iter()
            .find(|s| s.id == "permissions.yaml")
            .expect("permissions.yaml");
        assert_eq!(perms.kind, DocumentKind::Yaml);
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
        let must_contain = ["logs/*", "cache/*", "*.lock", "sessions/*"];
        for pat in must_contain {
            assert!(
                exclusions.contains(&pat.to_owned()),
                "exclusions must contain {pat}"
            );
        }
        assert!(!exclusions.contains(&".forge.toml".to_owned()));
    }

    #[test]
    fn plan_mirror_includes_config_and_excludes_sessions() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        let is_excluded = |file: &str| crate::adapters::exclusion_matches(&exclusions, file);
        assert!(!is_excluded(".forge.toml"));
        assert!(!is_excluded(".mcp.json"));
        assert!(is_excluded("sessions/abc.jsonl"));
        assert!(is_excluded("logs/forge.log"));
        assert!(is_excluded("cache/data"));
    }

    #[test]
    fn plan_wrapper_sets_forge_config() {
        let tmp_root = crate::test_util::tmp_abs_str(".forge-work");
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
        let tmp_root = crate::test_util::tmp_abs_str("my forge work");
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
        assert!(env_val.contains(' '));
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".forge-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_default_root() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.contains(".forge")));
        assert!(candidates.iter().any(|c| c.contains(CONFIG_ENV_VAR)));
    }

    #[test]
    fn validate_instance_accepts_relocated_root() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".forge-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".forge-work"));
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
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".forge-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    #[test]
    fn path_resolution_resolver_fallbacks() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        let forge_toml = surfaces.iter().find(|s| s.id == ".forge.toml").unwrap();
        assert_eq!(forge_toml.path_resolver.fallback, "~/.forge/.forge.toml");
        let resolver = &forge_toml.path_resolver;
        assert!(resolver.linux.as_deref().unwrap().contains(CONFIG_ENV_VAR));
        assert!(resolver.macos.as_deref().unwrap().contains(CONFIG_ENV_VAR));
        assert!(
            resolver
                .windows
                .as_deref()
                .unwrap()
                .contains(CONFIG_ENV_VAR)
        );
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/forge")
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
        let path = fixture_path("forge.minimal.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let doc = superai_config::toml_file::load(&path).unwrap();
        assert!(doc.is_empty() || doc.contains_key("providers") || doc.len() <= 4);
    }

    #[test]
    fn fixture_populated_parses_and_has_expected_keys() {
        let path = fixture_path("forge.populated.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let doc = superai_config::toml_file::load(&path).unwrap();
        let text = doc.to_string();
        assert!(
            text.contains("providers") || text.contains("session") || doc.contains_key("providers")
        );
    }

    #[test]
    fn fixture_foreign_preserves_unknown_keys_on_edit() {
        let path = fixture_path("forge.foreign.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let original = superai_config::toml_file::load(&path).unwrap();
        assert!(
            original.to_string().contains("foreignKey")
                || original.to_string().contains("unknownTopLevel")
                || original.contains_key("foreignKey")
        );
        let dir = crate::test_util::temp_dir_unique("forge");
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("forge.foreign.copy.toml");
        std::fs::copy(&path, &tmp).unwrap();
        superai_config::toml_file::edit(&tmp, |doc| {
            doc["model"] = toml_edit::value("preserve-test");
            assert!(
                doc.to_string().contains("foreignKey")
                    || doc.to_string().contains("unknownTopLevel")
            );
        })
        .unwrap();
        let after = superai_config::toml_file::load(&tmp).unwrap();
        let text = after.to_string();
        let foreign_preserved = text.contains("foreignKey")
            || text.contains("unknownTopLevel")
            || text.contains("customField");
        assert!(
            foreign_preserved,
            "foreign keys must be preserved, got {after:?}"
        );
        drop(std::fs::remove_file(&tmp));
    }

    #[test]
    fn fixture_malformed_fails_to_parse() {
        let path = fixture_path("forge.malformed.toml");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let result = superai_config::toml_file::load(&path);
        assert!(result.is_err(), "malformed fixture must fail to parse");
    }

    #[test]
    fn wrapper_env_var_isolation_is_relocated_root() {
        let tmp_root = crate::test_util::tmp_abs_str("user/.forge-isolated");
        let a = adapter();
        assert!(a.scan_candidates().len() >= 3);
        let inst = sample_instance_with_root(&tmp_root);
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(!plan.env_vars.is_empty());
        let (key, val) = &plan.env_vars[0];
        assert_eq!(key, CONFIG_ENV_VAR);
        assert_eq!(val, tmp_root.as_str());
    }

    #[test]
    fn registry_no_harness_value_leak() {
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".forge-work"));
        let json = serde_json::to_string(&inst).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let forbidden = [
            "model", "endpoint", "api_key", "skill", "plugin", "mcp", "baseUrl", "base_url",
        ];
        let text = json.to_lowercase();
        for field in forbidden {
            if let serde_json::Value::Object(map) = &v {
                assert!(
                    !map.contains_key(field),
                    "forbidden field `{field}` must not be emitted, json: {json}"
                );
            }
            assert!(
                !text.contains(&format!("\"{field}\"")),
                "forbidden field `{field}` appears in json: {json}"
            );
        }
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
