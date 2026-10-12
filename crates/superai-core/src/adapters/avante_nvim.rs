//! avante.nvim adapter: Lua plugin with no config files; `setup()` or
//! `vim.g.avante` only, MCP through mcphub.nvim. Research:
//! docs/harness-configs/avante-nvim.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, SurfaceOwnership, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "avante-nvim";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "avante.nvim";

/// Host editor binary probed during detection; the plugin ships no binary.
pub const EXECUTABLE: &str = "nvim";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/avante-nvim.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: configuration is Lua, not files.
pub const CONSTRAINED_NOTE: &str = "all configuration is Lua (vim.g.avante or setup()); no config file exists, so superai manages only the instruction and rules files the plugin reads (avante-nvim.md section 2)";

/// Concrete adapter for avante.nvim.
#[derive(Debug, Clone)]
pub struct AvanteNvimAdapter {
    id: HarnessId,
}

impl AvanteNvimAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for AvanteNvimAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for AvanteNvimAdapter {
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
        let mut version: Option<String> = None;
        let mut binary_found = false;

        match super::find_in_path(&[EXECUTABLE]) {
            Some(path) => {
                binary_found = true;
                evidence.push(format!(
                    "host binary `{EXECUTABLE}` found at {}",
                    path.display()
                ));
                match super::probe_version(&path) {
                    Some(v) => {
                        evidence.push(format!(
                            "host nvim version `{v}` (plugin version is the pinned commit)"
                        ));
                        version = Some(v);
                    }
                    None => evidence.push("host nvim version probe failed".to_owned()),
                }
            }
            None => evidence.push(format!("host binary `{EXECUTABLE}` not found in PATH")),
        }
        if std::path::Path::new("avante.md").exists() {
            evidence.push("avante.md project instructions present in cwd".to_owned());
        }

        let present = super::install_presence(binary_found, version.is_some());
        let confidence = super::detection_confidence(binary_found, version.is_some(), false);
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, HARNESS_ID_STR, SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let instructions = ConfigSurface::new(
            "avante.md",
            PathResolver::fallback_only("<project>/avante.md (relocatable via instructions_file)"),
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        let rules = ConfigSurface::new(
            "avanterules",
            PathResolver::fallback_only("*.planning.avanterules at project root"),
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![instructions, rules]
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
        let mut plan = WrapperPlan::new("neovim config-root isolation");
        plan.description = format!(
            " avante.nvim configuration lives in the Neovim Lua spec (no config file); isolate \
             per instance with a separate NVIM_APPNAME or XDG_CONFIG_HOME pointing at config_root \
             {}; AVANTE_* API-key vars select provider identities per process \
             (avante-nvim.md sections 2-3)",
            instance.config_root
        );
        plan.env_vars.push((
            "NVIM_APPNAME".to_owned(),
            instance.config_root.as_path().file_name().map_or_else(
                || "avante-work".to_owned(),
                |n| n.to_string_lossy().into_owned(),
            ),
        ));
        plan.shared_state_warnings.push(
            "provider API keys are process env vars (AVANTE_* or legacy ANTHROPIC_API_KEY family), \
             shared when inherited"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["avante.md".to_owned(), ".avanterules".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::IdeUserData | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("avante-nvim requires isolation ide_user_data, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no own MCP destination: MCP servers are configured in mcphub.nvim's config, not in avante's setup (avante-nvim.md section 4)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "the harness is itself a Neovim plugin; installation is the plugin manager's surface, not a file superai can stage (avante-nvim.md section 1)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AvanteNvimAdapter, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> AvanteNvimAdapter {
        AvanteNvimAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-avante-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".avante-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::IdeUserData,
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
    fn detection_explains_the_lua_only_surface() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("constrained") && e.contains("Lua"))
        );
    }

    #[test]
    fn config_surfaces_are_instruction_files_only() {
        let surfaces = adapter().config_surfaces();
        assert_eq!(surfaces.len(), 2);
        assert!(
            surfaces.iter().all(|s| s.owned_selectors.is_empty()),
            "no Lua keys may be fabricated"
        );
    }

    #[test]
    fn wrapper_keeps_env_vars_to_the_host_editor() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.env_vars.iter().all(|(k, _)| k == "NVIM_APPNAME"));
    }

    #[test]
    fn validate_instance_accepts_ide_user_data() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::FixedPathSingle;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_absence_names_the_mcphub_handoff() {
        let reason = adapter().mcp_absence_reason().unwrap();
        assert!(reason.contains("mcphub"));
        assert!(adapter().mcp_decl().is_none());
        assert!(adapter().plugin_absence_reason().is_some());
        assert!(
            adapter()
                .supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
