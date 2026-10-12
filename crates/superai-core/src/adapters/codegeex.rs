//! `CodeGeeX` adapter: Zhipu in-editor completion/chat plugin (VS Code,
//! `JetBrains`, `HBuilderX`, `DeepIn-IDE`); host-IDE settings only, no published
//! key list, no MCP, no env vars, no local files. Research-blocked.
//! Research: docs/harness-configs/codegeex.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigSurface, DetectionResult, Platform, ProductStatus,
    VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "codegeex";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "CodeGeeX";

/// Host IDE binary probed during detection; the plugin ships no binary.
pub const EXECUTABLE: &str = "code";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/codegeex.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Research-blocked reason: no verified key surface.
pub const RESEARCH_BLOCKED_NOTE: &str = "every setting is contributed through the host IDE's settings system and no official key list is published, so a config-writing adapter has no verified surface to target; install detection is the entire adapter surface (codegeex.md sections 2, 4)";

/// Concrete adapter for the Zhipu plugin.
#[derive(Debug, Clone)]
pub struct CodegeexAdapter {
    id: HarnessId,
}

impl CodegeexAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for CodegeexAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for CodegeexAdapter {
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
        let mut evidence = vec![format!("research blocked: {RESEARCH_BLOCKED_NOTE}")];
        // The VS Code `code` binary is host evidence only: CodeGeeX is an
        // extension inside whichever IDE profile has it installed.
        match super::find_in_path(&[EXECUTABLE]) {
            Some(path) => {
                evidence.push(format!(
                    "host IDE binary `{EXECUTABLE}` found at {} (extension presence per profile is unverifiable from here)",
                    path.display()
                ));
            }
            None => {
                evidence.push(format!("host IDE binary `{EXECUTABLE}` not found in PATH"));
            }
        }
        DetectionResult::new(
            crate::state::InstallPresence::Absent,
            None,
            evidence,
            crate::adapter::DetectionConfidence::Low,
        )
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, HARNESS_ID_STR, SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        Vec::new()
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::ResearchBlocked)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        Vec::new()
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("host-IDE isolation only");
        plan.description = format!(
            " CodeGeeX is a plugin inside the host IDE with no published settings keys, env \
             vars, or local files; isolation follows the host (code --user-data-dir {} or the \
             JetBrains config dir); superai's surface is install detection only \
             (codegeex.md section 4)",
            instance.config_root
        );
        plan.args.push("--user-data-dir".to_owned());
        plan.args.push(instance.config_root.to_string());
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
                reason: format!("codegeex requires isolation ide_user_data, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP is not mentioned anywhere on the fetched listing, site, or model repos; Zhipu's agent effort went to ZCode, catalogued separately (codegeex.md section 3)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "the harness is itself a host-IDE extension installed through the host's marketplace; no separate plugin mechanism exists (codegeex.md section 1)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CodegeexAdapter, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> CodegeexAdapter {
        CodegeexAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-codegeex-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".codegeex-work"))
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
    fn detection_never_claims_presence_without_a_surface() {
        let result = adapter().detection();
        assert_eq!(
            result.present,
            crate::state::InstallPresence::Absent,
            "extension presence inside an IDE profile is unverifiable"
        );
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("research blocked"))
        );
    }

    #[test]
    fn config_surfaces_are_empty_by_contract() {
        assert!(
            adapter().config_surfaces().is_empty(),
            "no published settings keys; surfaces must stay empty"
        );
    }

    #[test]
    fn wrapper_defers_to_host_ide_isolation() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.args.contains(&"--user-data-dir".to_owned()));
        assert!(plan.env_vars.is_empty());
    }

    #[test]
    fn validate_instance_accepts_matching_harness() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("zcode").unwrap();
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
                .all(|(_, s)| *s == AdapterSupport::ResearchBlocked)
        );
    }
}
