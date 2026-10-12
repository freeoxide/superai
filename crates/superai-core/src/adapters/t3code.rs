//! T3 Code adapter: agent-harness control surface (Electron desktop, web,
//! mobile, `t3` CLI server) driving pre-authenticated CLIs; repo `t3.json`
//! is the only file config; `--base-dir` is the isolation knob. Research:
//! docs/harness-configs/t3code.md.

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
pub const HARNESS_ID_STR: &str = "t3code";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "T3 Code";

/// Binary name resolved on PATH during detection (the CLI server).
pub const EXECUTABLE: &str = "t3";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/t3code.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside t3.json.
pub const OWNED_SELECTORS: &[&str] = &["worktreeSubmodules"];

/// Constrained note: settings live in t3's own undocumented store.
pub const CONSTRAINED_NOTE: &str = "environment settings live in the t3 server's own store with no documented on-disk path; the editable file surface is the committed repo t3.json, and --base-dir is the only isolation knob (t3code.md sections 2, 7)";

/// Concrete adapter for T3 Code.
#[derive(Debug, Clone)]
pub struct T3codeAdapter {
    id: HarnessId,
}

impl T3codeAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        if std::path::Path::new("t3.json").exists() {
            evidence
                .push("repo t3.json present in cwd (workspace and worktreeSubmodules)".to_owned());
        }
        if let Some(home) = super::home_dir()
            && home.join(".local").join("bin").join(EXECUTABLE).exists()
        {
            evidence.push("t3 binary present under ~/.local/bin (installer default)".to_owned());
        }
    }
}

impl Default for T3codeAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for T3codeAdapter {
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
        let mut binary_path: Option<PathBuf> = None;

        match super::find_in_path(&[EXECUTABLE]) {
            Some(path) => {
                evidence.push(format!("found binary `{EXECUTABLE}` at {}", path.display()));
                evidence.push(
                    "no dedicated version command is documented (`t3 update` upgrades)".to_owned(),
                );
                binary_path = Some(path);
            }
            None => evidence.push(format!("binary `{EXECUTABLE}` not found in PATH")),
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), false);
        let confidence = super::detection_confidence(binary_path.is_some(), false, false);
        DetectionResult::new(present, None, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        super::resolution_from_detection(detection, HARNESS_ID_STR, SCHEMA_VERSION_STR)
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut t3json = ConfigSurface::new(
            "t3.json",
            PathResolver::fallback_only("<repo>/t3.json (committed repo-level settings)"),
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        t3json.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        vec![t3json]
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
        let mut plan = WrapperPlan::new("--base-dir independent server root");
        plan.description = format!(
            " t3 is a control surface over CLIs you authenticate yourself: `t3 --base-dir {}` \
             runs an independent server with its own root; agent isolation happens in the \
             wrapped CLIs' own config trees (CODEX_HOME, CLAUDE_CONFIG_DIR, and friends), not \
             in t3 (t3code.md sections 5-6)",
            instance.config_root
        );
        plan.args.push("--base-dir".to_owned());
        plan.args.push(instance.config_root.to_string());
        plan.env_unset = ["T3CODE_CHANNEL", "T3CODE_VERSION"]
            .iter()
            .map(|var| (*var).to_owned())
            .collect();
        plan.shared_state_warnings.push(
            "device preferences live on the device only and environment settings live in the \
             server's undocumented store"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["t3.json".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot
            | Isolation::ExplicitConfig
            | Isolation::ProjectScope
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("t3code requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "the fetched pages document no MCP surface of t3's own; MCP servers configured for the underlying CLIs apply as those CLIs see them (t3code.md section 5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "t3 launches and supervises externally owned CLIs; no t3-side plugin mechanism is documented (t3code.md sections 1, 5)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, T3codeAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> T3codeAdapter {
        T3codeAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-t3-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".t3-work")).unwrap(),
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
    fn detection_never_claims_a_version() {
        let result = adapter().detection();
        assert!(result.version.is_none(), "no version command is documented");
    }

    #[test]
    fn wrapper_uses_the_base_dir_flag() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.args.contains(&"--base-dir".to_owned()));
        assert!(
            plan.env_unset.contains(&"T3CODE_CHANNEL".to_owned()),
            "channel pins must not leak across instances"
        );
        assert!(plan.env_vars.is_empty());
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("orca").unwrap();
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
