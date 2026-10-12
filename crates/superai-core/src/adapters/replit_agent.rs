//! Replit Agent adapter: cloud workspace agent; all configuration is
//! workspace/custom-instructions UI, `/.agents/skills` project files, and
//! replit.com/integrations MCP (HTTPS-only, account-wide). Unsupported
//! precedent: copilot-coding-agent. Research: docs/harness-configs/replit-agent.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, SurfaceOwnership, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "replit-agent";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Replit Agent";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/replit-agent.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Unsupported reason: cloud-owned everything, nothing local to mutate.
pub const UNSUPPORTED_NOTE: &str = "everything runs on Replit's cloud: no local executable, no local config file, no documented env vars; settings are workspace UI, project /.agents/skills files, and account-wide HTTPS MCP at replit.com/integrations (replit-agent.md sections 3-4)";

/// Concrete adapter for the Replit Agent.
#[derive(Debug, Clone)]
pub struct ReplitAgentAdapter {
    id: HarnessId,
}

impl ReplitAgentAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for ReplitAgentAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ReplitAgentAdapter {
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
        let evidence = vec![
            format!("unsupported: {UNSUPPORTED_NOTE}"),
            "the realistic local surface is reading /.agents/skills and instruction files out of a checked-out Replit project".to_owned(),
        ];
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
        let skills = ConfigSurface::new(
            ".agents/skills",
            PathResolver::fallback_only("/.agents/skills/ in the Replit project (SKILL.md format)"),
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![skills]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Unsupported)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        Vec::new()
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("cloud workspace agent; no local surface");
        plan.description = format!(
            " {UNSUPPORTED_NOTE}; config_root {} is informative only; isolation between \
             identities reduces to separate Replit accounts, and every setting is edited through \
             the web UI or by asking Agent itself (replit-agent.md section 4)",
            instance.config_root
        );
        plan.shared_state_warnings.push(
            "connected MCP integrations are account-wide across every project, HTTPS endpoints only"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![".agents/skills".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::Unsupported
            | Isolation::ProjectScope
            | Isolation::RelocatedRoot
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!(
                    "replit-agent requires isolation unsupported (cloud-only), got {other}"
                ),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP servers are configured at replit.com/integrations (HTTPS endpoints only, OAuth per server) and apply account-wide; no local file or stdio surface exists (replit-agent.md section 3)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no local plugin surface: skills are project files and chat attachments; everything else is workspace UI (replit-agent.md section 2)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, ReplitAgentAdapter};
    use crate::adapter::Adapter;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ReplitAgentAdapter {
        ReplitAgentAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-replit-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".replit-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::Unsupported,
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
    fn detection_is_absent_with_the_cloud_only_note() {
        let result = adapter().detection();
        assert_eq!(result.present, crate::state::InstallPresence::Absent);
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("unsupported") && e.contains("Replit's cloud"))
        );
    }

    #[test]
    fn wrapper_sets_no_env_vars_or_args() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.env_vars.is_empty(), "{:?}", plan.env_vars);
        assert!(plan.args.is_empty());
    }

    #[test]
    fn validate_instance_accepts_matching_harness() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("jules").unwrap();
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
                .all(|(_, s)| *s == AdapterSupport::Unsupported)
        );
    }
}
