//! Prime Agent adapter: self-improving RLM harness with daemon-backed
//! sessions; config dir `~/.prime/agent` (source-verified), env overrides
//! `PRIME_AGENT_CODING_AGENT_DIR` and friends; MCP surface undocumented.
//! Research: docs/harness-configs/prime-agent.md.

use std::path::PathBuf;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, SurfaceOwnership, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "prime-agent";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Prime Agent";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "prime-agent";

/// Env var overriding the agent config dir (~/.prime/agent).
pub const CONFIG_ENV_VAR: &str = "PRIME_AGENT_CODING_AGENT_DIR";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/prime-agent.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside settings.json.
pub const OWNED_SELECTORS: &[&str] = &["decisionApi"];

/// Constrained note: schema sparse; no security sandbox.
pub const CONSTRAINED_NOTE: &str = "the settings.json schema beyond decisionApi.systemOneModel is unpublished, and upstream states plainly that worker/kernel isolation improves lifecycle handling but is not a security sandbox (prime-agent.md sections 2, 4)";

/// Concrete adapter for Prime Agent.
#[derive(Debug, Clone)]
pub struct PrimeAgentAdapter {
    id: HarnessId,
}

impl PrimeAgentAdapter {
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
        super::home_dir().map(|home| home.join(".prime").join("agent"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        match Self::default_config_root() {
            Some(root) => {
                let settings = root.join("settings.json");
                if settings.exists() {
                    evidence.push(format!("settings.json found at {}", settings.display()));
                } else {
                    evidence.push(format!("settings.json missing at {}", settings.display()));
                }
            }
            None => evidence.push("could not resolve config root (no HOME)".to_owned()),
        }
    }
}

impl Default for PrimeAgentAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for PrimeAgentAdapter {
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
                    None => {
                        evidence.push("version probe failed; `prime-agent update` is the documented version surface".to_owned());
                    }
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
        let mut settings = ConfigSurface::new(
            "settings.json",
            PathResolver::fallback_only(
                "$PRIME_AGENT_CODING_AGENT_DIR (default ~/.prime/agent)/settings.json",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        settings.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        vec![settings]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["sessions/*".to_owned(), "snapshots/*".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("PRIME_AGENT_CODING_AGENT_DIR relocation");
        plan.description = format!(
            " prime-agent moves its agent dir with {CONFIG_ENV_VAR}; config_root {} carries \
             settings.json and saved sessions, while the daemon socket moves with \
             PRIME_AGENT_DAEMON_SOCKET (or --daemon-socket) so instances do not share a \
             supervisor; whether the dir override also moves auth storage is unverified \
             (prime-agent.md sections 2, 4)",
            instance.config_root
        );
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.env_unset = ["PRIME_AGENT_SESSION_DIR", "PRIME_AGENT_DAEMON_SOCKET"]
            .iter()
            .map(|var| (*var).to_owned())
            .collect();
        plan.state_paths
            .push(format!("{CONFIG_ENV_VAR}={}", instance.config_root));
        plan.shared_state_warnings.push(
            "the worker/kernel isolation is a lifecycle feature, not a security sandbox; model \
             code runs with user permissions"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.prime/agent".to_owned(),
            "$PRIME_AGENT_CODING_AGENT_DIR".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot
            | Isolation::EnvOnly
            | Isolation::DaemonService
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("prime-agent requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "the MCP Python SDK is vendored as a dependency but no MCP configuration surface is documented upstream (prime-agent.md sections 4-5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "skills are importable Python packages with a built-in creator; no plugin mechanism is documented (prime-agent.md section 3)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_ENV_VAR, HARNESS_ID_STR, PrimeAgentAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> PrimeAgentAdapter {
        PrimeAgentAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-prime-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".prime-work")).unwrap(),
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
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn detection_carries_the_no_security_sandbox_warning() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("not a security sandbox"))
        );
    }

    #[test]
    fn wrapper_sets_the_dir_and_unsets_daemon_couplings() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, CONFIG_ENV_VAR);
        assert!(
            plan.env_unset
                .contains(&"PRIME_AGENT_DAEMON_SOCKET".to_owned())
        );
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("openfang").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_relocated_root() {
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
