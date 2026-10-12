//! Ori (`OpenRouter`) adapter: binary-only CLI from a private source repo;
//! documented surface is install/update only (`ORI_INSTALL_DIR`,
//! `ORI_CHANNEL`); it wraps claude-code/codex-cli/hermes-agent as a second
//! model-selection writer. Research-blocked. Research:
//! docs/harness-configs/ori.md.

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigSurface, DetectionResult, Platform, ProductStatus,
    VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "ori";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Ori (OpenRouter)";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "ori";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/ori.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Research-blocked reason: closed source, install-only surface.
pub const RESEARCH_BLOCKED_NOTE: &str = "the source repository is private and the published pages document no config file, no runtime ORI_* variables, and no per-harness settings; only install/update is documented, and what Ori injects into wrapped CLIs is unpublished (ori.md sections 2-4)";

/// Concrete adapter for Ori.
#[derive(Debug, Clone)]
pub struct OriAdapter {
    id: HarnessId,
}

impl OriAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for OriAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for OriAdapter {
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
        let mut version: Option<String> = None;
        let mut binary_found = false;

        match super::find_in_path(&[EXECUTABLE]) {
            Some(path) => {
                binary_found = true;
                evidence.push(format!("found binary `{EXECUTABLE}` at {}", path.display()));
                match super::probe_version(&path) {
                    Some(v) => {
                        evidence.push(format!(
                            "version `{v}` (releases are tagged cli-<version>-<commit>, immutable)"
                        ));
                        version = Some(v);
                    }
                    None => evidence.push("version probe failed".to_owned()),
                }
            }
            None => evidence.push(format!("binary `{EXECUTABLE}` not found in PATH")),
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
        super::operations_at(AdapterSupport::ResearchBlocked)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        Vec::new()
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("install-surface only; closed source");
        plan.description = format!(
            " ori documents only its install surface (ORI_INSTALL_DIR, ori update --alpha/--stable; \
             checksums, not signatures); `ori claude`/`ori codex`/`ori hermes` are launcher verbs \
             that keep those harnesses' own config trees while moving model selection to \
             OpenRouter, so config_root {} is informative only and a wrapper should expect a \
             second writer on the wrapped CLIs (ori.md sections 1, 3)",
            instance.config_root
        );
        plan.shared_state_warnings.push(
            "OpenRouter account auth flows through the wrapped harness; per-identity isolation \
             means separate OpenRouter accounts"
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
            Isolation::Unknown | Isolation::RelocatedRoot | Isolation::ExplicitConfig => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!(
                    "ori isolation is unknown (no documented runtime surface), got {other}"
                ),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no config file, runtime env var, or per-harness setting is documented beyond install/update; there is no MCP surface to declare (ori.md section 2)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "binary-only distribution with a private source repo; no plugin mechanism is published (ori.md sections 2, 4)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, OriAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> OriAdapter {
        OriAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-ori-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".ori-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::Unknown,
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
    fn detection_names_the_closed_source_block() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("research blocked") && e.contains("private"))
        );
    }

    #[test]
    fn config_surfaces_are_empty_by_contract() {
        assert!(
            adapter().config_surfaces().is_empty(),
            "no runtime surface is documented; surfaces must stay empty"
        );
    }

    #[test]
    fn wrapper_sets_no_env_vars() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.env_vars.is_empty());
        assert!(
            plan.description.contains("second writer"),
            "the wrapping interplay must be visible in the plan"
        );
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("hermes-agent").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_unknown_isolation() {
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
                .all(|(_, s)| *s == AdapterSupport::ResearchBlocked)
        );
    }
}
