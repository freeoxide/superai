//! Orca (Stably AI) adapter: agent development environment driving BYO CLI
//! fleets; file anchors are `~/.orca/keybindings.json` and repo
//! `orca.yaml`/`.worktreeinclude`; settings storage path undocumented.
//! Research: docs/harness-configs/orca.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, SurfaceOwnership, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "orca";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Orca (Stably AI)";

/// CLI binary shipped with the desktop app.
pub const EXECUTABLE: &str = "orca";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/orca.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside orca.yaml.
pub const OWNED_SELECTORS: &[&str] = &["worktree.sharedDirectories"];

/// Constrained note: settings state path undocumented.
pub const CONSTRAINED_NOTE: &str = "the app stores its settings state at an undocumented location; the only file-path anchors are ~/.orca/keybindings.json and repo-level orca.yaml/.worktreeinclude (orca.md sections 2, 7)";

/// Concrete adapter for Orca.
#[derive(Debug, Clone)]
pub struct OrcaAdapter {
    id: HarnessId,
}

impl OrcaAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for OrcaAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for OrcaAdapter {
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
        let mut evidence = vec![format!("constrained: {CONSTRAINED_NOTE}")];
        let mut binary_found = false;

        match super::find_in_path(&[EXECUTABLE]) {
            Some(path) => {
                binary_found = true;
                evidence.push(format!(
                    "cli binary `{EXECUTABLE}` found at {} (ships with the desktop app)",
                    path.display()
                ));
            }
            None => evidence.push(format!(
                "cli binary `{EXECUTABLE}` not found in PATH (ships with the desktop app)"
            )),
        }
        if let Some(home) = super::home_dir()
            && home.join(".orca").join("keybindings.json").exists()
        {
            evidence.push("~/.orca/keybindings.json present".to_owned());
        }
        if std::path::Path::new("orca.yaml").exists() {
            evidence.push("repo orca.yaml present in cwd".to_owned());
        }

        // The app self-updates; no version-probe command is documented.
        let present = super::install_presence(binary_found, false);
        let confidence = super::detection_confidence(binary_found, false, false);
        DetectionResult::new(present, None, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, HARNESS_ID_STR, SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let keybindings = ConfigSurface::new(
            "keybindings.json",
            PathResolver::fallback_only("~/.orca/keybindings.json (customized tab navigation)"),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );

        let mut repo = ConfigSurface::new(
            "orca.yaml",
            PathResolver::fallback_only(
                "<repo>/orca.yaml (worktree.sharedDirectories, base ref, hooks)",
            ),
            DocumentKind::Yaml,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        repo.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();

        let include = ConfigSurface::new(
            ".worktreeinclude",
            PathResolver::fallback_only(
                "<repo>/.worktreeinclude (files copied into every worktree)",
            ),
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![keybindings, repo, include]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        Vec::new()
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("ADE wrapping BYO agent fleets");
        plan.description = format!(
            " Orca is itself the wrapper: per-agent binary path, command, args, and env \
             overrides are its native routing mechanism over CLIs you authenticate yourself; no \
             config-dir relocation for Orca exists, so config_root {} is informative only and \
             isolation happens inside the wrapped CLIs' own config trees (orca.md sections 3, 6)",
            instance.config_root
        );
        plan.shared_state_warnings.push(
            "account lists (Claude, Codex, OpenCode, Devin, Antigravity, GLM) and folder-trust \
             bypass defaults live in app state at an undocumented path, shared across launches"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.orca".to_owned(),
            "orca.yaml".to_owned(),
            ".worktreeinclude".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ProjectScope | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("orca requires isolation project_scope, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP servers are configured under Settings, Integrations, but the write mechanics live on a dedicated page that was not fetched; treat the path as unverified (orca.md sections 4, 7)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "the plugin system is experimental with git-based marketplaces and no documented filesystem destination (orca.md section 4)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, OrcaAdapter};
    use crate::adapter::Adapter;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> OrcaAdapter {
        OrcaAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-orca-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".orca-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::ProjectScope,
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
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn detection_never_claims_a_version() {
        let result = adapter().detection();
        assert!(result.version.is_none(), "no version probe is documented");
        assert!(result.evidence.iter().any(|e| e.contains("undocumented")));
    }

    #[test]
    fn wrapper_sets_no_env_vars() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no relocation knob is documented: {:?}",
            plan.env_vars
        );
    }

    #[test]
    fn validate_instance_accepts_project_scope() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::Unsupported;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_and_plugin_absence_are_explicit() {
        let a = adapter();
        assert!(a.mcp_absence_reason().is_some());
        assert!(a.mcp_decl().is_none());
        assert!(a.plugin_absence_reason().is_some());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
