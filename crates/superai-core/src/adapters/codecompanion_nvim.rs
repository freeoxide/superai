//! codecompanion.nvim adapter: Lua plugin, `setup()` table only, native
//! stdio MCP under the `mcp` key, built-in ACP. Research:
//! docs/harness-configs/codecompanion-nvim.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigSurface, DetectionResult, Platform, ProductStatus,
    VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "codecompanion-nvim";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "codecompanion.nvim";

/// Host editor binary probed during detection; the plugin ships no binary.
pub const EXECUTABLE: &str = "nvim";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/codecompanion-nvim.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: the Lua table is the only surface.
pub const CONSTRAINED_NOTE: &str = "configuration is the Lua setup() table only; no file-based config is documented, so superai manages nothing but the isolated Neovim config root (codecompanion-nvim.md sections 2, 7)";

/// Concrete adapter for codecompanion.nvim.
#[derive(Debug, Clone)]
pub struct CodecompanionNvimAdapter {
    id: HarnessId,
}

impl CodecompanionNvimAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for CodecompanionNvimAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for CodecompanionNvimAdapter {
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
                        evidence.push(format!("host nvim version `{v}`; :checkhealth codecompanion is the plugin diagnostic"));
                        version = Some(v);
                    }
                    None => evidence.push("host nvim version probe failed".to_owned()),
                }
            }
            None => evidence.push(format!("host binary `{EXECUTABLE}` not found in PATH")),
        }

        let present = super::install_presence(binary_found, version.is_some());
        let confidence = super::detection_confidence(binary_found, version.is_some(), false);
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, HARNESS_ID_STR, SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        Vec::new()
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
            " codecompanion.nvim configuration lives in the Neovim init files (setup() table, \
             including the native mcp key with stdio servers); isolate per instance with \
             XDG_CONFIG_HOME pointing at config_root {}, so nvim resolves its config at \
             config_root/nvim whatever the root's basename is; an inherited NVIM_APPNAME is \
             unset to keep that location fixed; adapter API keys are per-process env vars \
             (codecompanion-nvim.md sections 3, 6-7)",
            instance.config_root
        );
        plan.env_vars.push((
            "XDG_CONFIG_HOME".to_owned(),
            instance.config_root.to_string(),
        ));
        plan.env_unset.push("NVIM_APPNAME".to_owned());
        plan.shared_state_warnings.push(
            "only the Neovim config root is isolated; nvim data, state, and cache stay in the \
             user's XDG tree (the fetched pages discuss config roots only)"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        Vec::new()
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::IdeUserData | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("codecompanion-nvim requires isolation ide_user_data, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "native stdio MCP is configured in the Lua setup() table under the mcp key (protocol 2025-11-25); no file destination exists for superai to manage (codecompanion-nvim.md section 4)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "the harness is itself a Neovim plugin; installation is the plugin manager's surface, not a file superai can stage (codecompanion-nvim.md section 1)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CodecompanionNvimAdapter, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> CodecompanionNvimAdapter {
        CodecompanionNvimAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-cc-nvim-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".cc-nvim-work"))
                .unwrap(),
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
    fn config_surfaces_are_empty_by_contract() {
        assert!(
            adapter().config_surfaces().is_empty(),
            "no file-based config is documented; surfaces must stay empty"
        );
    }

    #[test]
    fn detection_explains_the_lua_only_surface() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("constrained") && e.contains("setup() table"))
        );
    }

    #[test]
    fn wrapper_relocates_the_nvim_config_root() {
        let inst = sample_instance();
        let plan = adapter().plan_wrapper(&inst).unwrap();
        let xdg = plan
            .env_vars
            .iter()
            .find(|(k, _)| k == "XDG_CONFIG_HOME")
            .expect("XDG_CONFIG_HOME must be set");
        assert_eq!(xdg.1, inst.config_root.to_string());
        assert!(
            plan.env_vars.iter().all(|(k, _)| k != "NVIM_APPNAME"),
            "NVIM_APPNAME must not be derived from the root basename"
        );
        assert!(plan.env_unset.contains(&"NVIM_APPNAME".to_owned()));
    }

    #[test]
    fn same_basename_config_roots_resolve_apart() {
        let mut a = sample_instance();
        a.config_root = AbsolutePath::new(
            &crate::test_util::tmp_abs("cc-nvim-collide-a")
                .join("work")
                .to_string_lossy(),
        )
        .unwrap();
        let mut b = sample_instance();
        b.config_root = AbsolutePath::new(
            &crate::test_util::tmp_abs("cc-nvim-collide-b")
                .join("work")
                .to_string_lossy(),
        )
        .unwrap();
        assert_eq!(
            a.config_root.as_path().file_name(),
            b.config_root.as_path().file_name(),
            "the fixture must exercise the basename collision"
        );
        let resolved = |inst: &Instance| {
            let plan = adapter().plan_wrapper(inst).unwrap();
            let xdg = plan
                .env_vars
                .iter()
                .find(|(k, _)| k == "XDG_CONFIG_HOME")
                .expect("XDG_CONFIG_HOME must be set");
            std::path::Path::new(&xdg.1).join("nvim")
        };
        let (ra, rb) = (resolved(&a), resolved(&b));
        assert!(ra.starts_with(a.config_root.as_path()));
        assert!(rb.starts_with(b.config_root.as_path()));
        assert_ne!(ra, rb, "distinct roots must resolve to distinct configs");
    }

    #[test]
    fn validate_instance_accepts_ide_user_data() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::FixedPathSingle;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_absence_names_the_lua_table() {
        let a = adapter();
        let reason = a.mcp_absence_reason().unwrap();
        assert!(reason.contains("setup()"));
        assert!(a.mcp_decl().is_none());
        assert!(a.plugin_absence_reason().is_some());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
