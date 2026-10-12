//! Terminus adapter: research-preview benchmark agent from the
//! Terminal-Bench team, pre-integrated into Harbor (variants terminus,
//! terminus-1, terminus-2); no standalone install, no user config files;
//! credentials per-run via harbor --ak/--ae. Research-blocked. Research:
//! docs/harness-configs/terminus.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigSurface, DetectionResult, Platform, ProductStatus,
    VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "terminus";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Terminus (Terminal-Bench)";

/// Host framework binary probed during detection; Terminus ships only
/// inside Harbor.
pub const HOST_EXECUTABLE: &str = "harbor";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/terminus.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Research-blocked reason: benchmark harness, no user surface.
pub const RESEARCH_BLOCKED_NOTE: &str = "a benchmark agent in the swe-agent mold: it ships only as pre-integrated Harbor agents (terminus/terminus-1/terminus-2) with no standalone install, no user config files, and per-run credentials through harbor --ak/--ae; instances are benchmark runs isolated by Harbor's task containers (terminus.md sections 1-4)";

/// Concrete adapter for Terminus.
#[derive(Debug, Clone)]
pub struct TerminusAdapter {
    id: HarnessId,
}

impl TerminusAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for TerminusAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for TerminusAdapter {
    fn id(&self) -> HarnessId {
        self.id.clone()
    }

    fn display_name(&self) -> &str {
        DISPLAY_NAME
    }

    fn product_status(&self) -> ProductStatus {
        ProductStatus::Preview
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
        match super::find_in_path(&[HOST_EXECUTABLE]) {
            Some(path) => evidence.push(format!(
                "host binary `{HOST_EXECUTABLE}` found at {}; Terminus itself has no standalone install (the PyPI `terminus` name is an unrelated Debian tool)",
                path.display()
            )),
            None => evidence.push(format!(
                "host binary `{HOST_EXECUTABLE}` not found in PATH; Terminus is unreachable without Harbor (and the PyPI `terminus` name is an unrelated Debian tool)"
            )),
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
        let mut plan = WrapperPlan::new("benchmark runs; no personal surface to wrap");
        plan.description = format!(
            " {RESEARCH_BLOCKED_NOTE}; config_root {} is informative only; runs go through \
             `harbor run --dataset terminal-bench@2.0 --agent terminus --model ... --ak ...` \
             with agent env injected via --ae; treat exported run artifacts as secret-bearing \
             (API keys can end up in them) (terminus.md sections 2-3)",
            instance.config_root
        );
        plan.shared_state_warnings.push(
            "environment variables containing API keys can end up in Harbor run artifacts and \
             logs; exported run data is secret-bearing"
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
            Isolation::OsBound | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("terminus requires isolation os_bound, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "terminus-2 accepts MCP configuration passed through by Harbor per run (--ae/--ak); no standalone MCP config file exists (terminus.md sections 2-3)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "pre-integrated Harbor agents need no per-agent install and expose no plugin mechanism (terminus.md section 1)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, TerminusAdapter};
    use crate::adapter::Adapter;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> TerminusAdapter {
        TerminusAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-terminus-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".terminus-work"))
                .unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::OsBound,
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
        assert_eq!(a.product_status(), entry.product_status);
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn detection_is_absent_and_names_the_pypi_name_trap() {
        let result = adapter().detection();
        assert_eq!(result.present, crate::state::InstallPresence::Absent);
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("unrelated Debian tool"))
        );
    }

    #[test]
    fn wrapper_sets_no_env_vars_and_warns_about_secret_artifacts() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.env_vars.is_empty());
        assert!(
            plan.shared_state_warnings
                .iter()
                .any(|w| w.contains("secret-bearing"))
        );
    }

    #[test]
    fn validate_instance_accepts_os_bound() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("swe-agent").unwrap();
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
