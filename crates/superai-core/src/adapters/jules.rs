//! Jules CLI adapter: `jules` binary, thin cloud-session client. No local
//! config files, env vars, MCP, or skills surface; sessions run on Google
//! cloud VMs (`jules remote new`/`remote pull`).

use std::path::PathBuf;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigSurface, DetectionResult, Platform, ProductStatus,
    VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "jules";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Jules CLI";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "jules";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/jules.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: work happens remotely; the CLI is an orchestrator.
pub const CONSTRAINED_NOTE: &str = "tasks run on Google cloud VMs against GitHub repos; the CLI documents no local config files, env vars, or sandbox; isolation reduces to separate Google accounts";

/// Concrete adapter for the Jules CLI.
#[derive(Debug, Clone)]
pub struct JulesAdapter {
    id: HarnessId,
}

impl JulesAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }
}

impl Default for JulesAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for JulesAdapter {
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
                match super::probe_version_with_fallback(
                    &path,
                    &["version"],
                    &[],
                    std::time::Duration::from_secs(2),
                ) {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `{EXECUTABLE} version`"));
                        version = Some(v);
                    }
                    None => {
                        evidence.push(format!(
                            "version probe failed for `{EXECUTABLE} version` (timeout or non-zero)"
                        ));
                    }
                }
                binary_path = Some(path);
            }
            None => {
                evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            }
        }

        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        evidence
            .push("no local config files or env vars documented (jules.md section 3)".to_owned());

        let present = super::install_presence(binary_path.is_some(), version.is_some());

        let confidence =
            super::detection_confidence(binary_path.is_some(), version.is_some(), false);
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, "jules", SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        Vec::new()
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
        Vec::new()
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("cloud session client; no local config surface");
        plan.description = format!(
            " jules runs sessions on Google cloud VMs; the CLI documents no local \
             config or MCP surface, so config_root {} is informative only; the \
             realistic adapter surface is version probe, login state, and scripted \
             `jules remote new`/`remote pull` (jules.md sections 3-5)",
            instance.config_root
        );
        plan.shared_state_warnings = vec![
            "isolation reduces to separate Google accounts (jules login per environment)"
                .to_owned(),
            "session history and plan usage limits are server-side and shared per account"
                .to_owned(),
        ];
        plan.auth_prerequisites = vec![crate::adapter::AuthPrerequisite::harness_login(
            "jules login (Google browser OAuth); no API-key path documented",
        )];
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        Vec::new()
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::DaemonService | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("jules requires isolation daemon_service, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no local MCP surface: sessions run on Google cloud VMs and the CLI reference documents no MCP mechanism (jules.md sections 3, 6)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no local plugin surface: the CLI is a session orchestrator (jules.md section 3)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{HARNESS_ID_STR, JulesAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> JulesAdapter {
        JulesAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-jules-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::DaemonService,
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
        assert_eq!(a.display_name(), entry.display_name);
        assert_eq!(a.product_status(), entry.product_status);
        assert_eq!(a.research_doc_link(), entry.research_doc);
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn supported_platforms_covers_all() {
        let a = adapter();
        let os_set: HashSet<String> = a
            .supported_platforms()
            .iter()
            .map(|p| p.os.to_string())
            .collect();
        assert!(os_set.contains("linux"));
        assert!(os_set.contains("macos"));
    }

    #[test]
    fn detection_returns_evidence_and_confidence() {
        let a = adapter();
        let result = a.detection();
        assert!(!result.evidence.is_empty());
        assert!(result.evidence.iter().any(|e| e.contains("constrained")));
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("no local config files"))
        );
        match result.present {
            InstallPresence::Absent => {
                assert!(result.version.is_none());
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
    fn config_surfaces_are_empty_by_contract() {
        let a = adapter();
        assert!(
            a.config_surfaces().is_empty(),
            "jules documents no local config files; surfaces must stay empty"
        );
    }

    #[test]
    fn supported_operations_are_constrained() {
        let a = adapter();
        let ops = a.supported_operations();
        for (name, support) in &ops {
            assert_eq!(
                *support,
                AdapterSupport::Constrained,
                "operation {name} should be Constrained"
            );
        }
        let names: HashSet<String> = ops.iter().map(|(n, _)| n.clone()).collect();
        for required in ["detect", "plan_wrapper", "scan_candidates"] {
            assert!(names.contains(required), "missing op {required}");
        }
    }

    #[test]
    fn scan_candidates_are_empty_by_contract() {
        let a = adapter();
        assert!(
            a.scan_candidates().is_empty(),
            "no local config roots are documented for jules"
        );
    }

    #[test]
    fn plan_wrapper_declares_no_env_vars_or_args() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".jules-work"));
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no env vars may be fabricated: {:?}",
            plan.env_vars
        );
        assert!(plan.args.is_empty());
        assert!(!plan.description.is_empty());
        assert!(plan.description.contains("remote new"));
        assert!(
            plan.shared_state_warnings
                .iter()
                .any(|w| w.contains("Google accounts"))
        );
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".jules-work"));
        inst.harness = HarnessId::new("codex-cli").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_daemon_service() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".jules-work"));
        a.validate_instance(&inst).unwrap();
        let mut relocated =
            sample_instance_with_root(&crate::test_util::tmp_abs_str(".jules-work"));
        relocated.isolation = Isolation::RelocatedRoot;
        a.validate_instance(&relocated).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".jules-work"));
        inst.isolation = Isolation::FixedPathSingle;
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "isolation"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn validate_instance_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".jules-work"));
        inst.harness = HarnessId::new("aider").unwrap();
        assert!(a.validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_absence_is_explicit_and_corporeal() {
        let a = adapter();
        let reason = a.mcp_absence_reason().unwrap();
        assert!(reason.contains("cloud VMs"));
        assert!(a.mcp_decl().is_none());
    }

    #[test]
    fn plugin_absence_is_explicit() {
        let a = adapter();
        assert!(a.plugin_absence_reason().is_some());
        assert!(a.plugin_decl().is_none());
    }

    #[test]
    fn adapter_is_object_safe() {
        let a = adapter();
        let boxed: Box<dyn Adapter> = Box::new(a);
        assert_eq!(boxed.id().as_str(), HARNESS_ID_STR);
        assert!(boxed.config_surfaces().is_empty());
        assert!(boxed.plan_mirror_exclusions().is_empty());
        assert!(boxed.mcp_decl().is_none());
    }
}
