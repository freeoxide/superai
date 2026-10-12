//! agentty adapter: single static binary, `AGENTTY_HOME` whole-root
//! relocation plus per-dir `AGENTTY_*_DIR` overrides, `~/.agentty/` JSON
//! tree with hand-edited `mcp.json`. Research: docs/harness-configs/agentty.md.

use std::path::PathBuf;

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
pub const HARNESS_ID_STR: &str = "agentty";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "agentty";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "agentty";

/// Env var relocating the whole `~/.agentty` root.
pub const CONFIG_ENV_VAR: &str = "AGENTTY_HOME";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/agentty.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside the hand-edited `mcp.json` container.
pub const OWNED_SELECTORS: &[&str] = &["mcpServers"];

/// Concrete adapter for agentty.
#[derive(Debug, Clone)]
pub struct AgenttyAdapter {
    id: HarnessId,
}

impl AgenttyAdapter {
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
        super::home_dir().map(|home| home.join(".agentty"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        let Some(root) = Self::default_config_root() else {
            evidence.push("could not resolve config root (no HOME)".to_owned());
            return;
        };
        let mcp = root.join("mcp.json");
        let credentials = root.join("credentials");
        if root.exists() {
            evidence.push(format!("config root exists at {}", root.display()));
            evidence.push(if mcp.exists() {
                format!("mcp.json found at {}", mcp.display())
            } else {
                format!("mcp.json missing at {}", mcp.display())
            });
            if credentials.exists() {
                evidence.push("credentials dir present (0700 upstream)".to_owned());
            }
        } else {
            evidence.push(format!("config root missing at {}", root.display()));
        }
        if std::path::Path::new(".agentty").join("mcp.json").exists() {
            evidence.push(".agentty/mcp.json present in cwd (project scope)".to_owned());
        }
    }
}

impl Default for AgenttyAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for AgenttyAdapter {
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
                evidence.push(format!("found binary `{EXECUTABLE}` at {}", path.display()));
                match super::probe_version(&path) {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `{EXECUTABLE} --version`"));
                        version = Some(v);
                    }
                    None => evidence.push(format!(
                        "version probe failed for `{EXECUTABLE} --version` (no flag documented; timeout or non-zero)"
                    )),
                }
                binary_path = Some(path);
            }
            None => evidence.push(format!("binary `{EXECUTABLE}` not found in PATH")),
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), version.is_some());
        let confidence =
            super::detection_confidence(binary_path.is_some(), version.is_some(), false);
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, HARNESS_ID_STR, SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let mcp_resolver = PathResolver::new(
            Some("$AGENTTY_HOME/mcp.json"),
            Some("$AGENTTY_HOME/mcp.json"),
            Some("%USERPROFILE%\\.agentty\\mcp.json"),
            "~/.agentty/mcp.json",
        );
        let mut mcp = ConfigSurface::new(
            "mcp.json",
            mcp_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        mcp.precedence = 10;
        mcp.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        mcp.restart_behavior = RestartBehavior::Restart;
        surfaces.push(mcp);

        let project_resolver =
            PathResolver::fallback_only(".agentty/mcp.json (project, untrusted by default)");
        let mut project = ConfigSurface::new(
            "project .agentty/mcp.json",
            project_resolver,
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project.precedence = 20;
        project.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        surfaces.push(project);

        let hooks_resolver = PathResolver::fallback_only("$AGENTTY_HOME/hooks.json");
        let hooks = ConfigSurface::new(
            "hooks.json",
            hooks_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        surfaces.push(hooks);

        let agents_resolver = PathResolver::fallback_only("$AGENTTY_HOME/AGENTS.md");
        let agents = ConfigSurface::new(
            "AGENTS.md",
            agents_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        surfaces.push(agents);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Full)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "threads/*".to_owned(),
            "credentials/*".to_owned(),
            "cache/*".to_owned(),
            "logs/*".to_owned(),
            "state/*".to_owned(),
            "memory.jsonl".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("AGENTTY_HOME whole-root relocation");
        plan.description = format!(
            " agentty relocates its whole root with {CONFIG_ENV_VAR}; config_root {} carries \
             settings, mcp.json, hooks, skills, and threads; per-dir AGENTTY_*_DIR overrides \
             can punch holes back to shared locations, so keep them unset inside the wrapper \
             (agentty.md sections 2-3)",
            instance.config_root
        );
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.env_unset = [
            "AGENTTY_THREADS_DIR",
            "AGENTTY_CREDENTIALS_DIR",
            "AGENTTY_STATE_DIR",
        ]
        .iter()
        .map(|var| (*var).to_owned())
        .collect();
        plan.state_paths
            .push(format!("{CONFIG_ENV_VAR}={}", instance.config_root));
        plan.shared_state_warnings.push(
            "credentials default to plaintext JSON under the root; keystore encryption is opt-in \
             (AGENTTY_USE_KEYSTORE)"
                .to_owned(),
        );
        plan.auth_prerequisites
            .push(crate::adapter::AuthPrerequisite::env_var(
                "ANTHROPIC_API_KEY",
            ));
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.agentty".to_owned(),
            "$AGENTTY_HOME".to_owned(),
            ".agentty/mcp.json".to_owned(),
            "AGENTS.md".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot
            | Isolation::EnvOnly
            | Isolation::ExplicitConfig
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("agentty requires isolation relocated_root, got {other}"),
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
            RestartBehavior::Restart,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "skills drop into .agentty/skills as SKILL.md; no plugin mechanism is documented (agentty.md section 4)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AgenttyAdapter, CONFIG_ENV_VAR, EXECUTABLE, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> AgenttyAdapter {
        AgenttyAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-agentty-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".agentty-work"))
                .unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::RelocatedRoot,
            origin: InstanceOrigin::Created,
            ownership: Ownership::SuperaiCreated,
            template: None,
            created_at: "2026-10-12T00:00:00Z".to_owned(),
            adapter_revision: crate::adapter::ADAPTER_REVISION.to_owned(),
        }
    }

    #[test]
    fn adapter_identity_matches_catalog() {
        let a = adapter();
        let entry = crate::harness_catalog::find_by_id(HARNESS_ID_STR).unwrap();
        assert_eq!(a.id().as_str(), HARNESS_ID_STR);
        assert_eq!(a.display_name(), entry.display_name);
        assert_eq!(a.research_doc_link(), entry.research_doc);
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn detection_reports_binary_and_config_evidence() {
        let result = adapter().detection();
        assert!(!result.evidence.is_empty());
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains(EXECUTABLE) || e.contains("config root"))
        );
        if result.present == InstallPresence::Present {
            assert!(result.version.is_some());
        }
    }

    #[test]
    fn version_resolution_maps_detected_version() {
        let res = adapter().version_resolution();
        if res.detected_version.is_some() {
            assert_eq!(
                res.schema_version.as_deref(),
                Some(super::SCHEMA_VERSION_STR)
            );
            assert!(res.compatible);
        } else {
            assert!(!res.compatible);
        }
    }

    #[test]
    fn supported_operations_are_full_across_the_row() {
        for (name, support) in adapter().supported_operations() {
            assert_eq!(support, AdapterSupport::Full, "operation {name}");
        }
    }

    #[test]
    fn wrapper_sets_only_the_documented_root_var() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, CONFIG_ENV_VAR);
        assert!(
            plan.env_unset
                .contains(&"AGENTTY_CREDENTIALS_DIR".to_owned()),
            "per-dir overrides must not leak into an isolated root"
        );
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("codex-cli").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_relocated_and_rejects_fixed_single() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::FixedPathSingle;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_decl_targets_user_mcp_json() {
        let decl = adapter().mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "mcp.json");
        assert_eq!(decl.dest_key, "mcpServers");
        assert!(decl.read_only.is_none());
        assert!(adapter().plugin_absence_reason().is_some());
    }
}
