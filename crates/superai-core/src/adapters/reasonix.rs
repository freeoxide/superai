//! Reasonix adapter: DeepSeek-native terminal agent with `~/.reasonix/
//! config.json` plus project `.reasonix/` overrides; stdio/SSE/
//! streamable-HTTP MCP; skills cross-read `.claude/skills`. Research:
//! docs/harness-configs/reasonix.md.

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
pub const HARNESS_ID_STR: &str = "reasonix";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Reasonix";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "reasonix";

/// Companion shim installed by the same npm package.
pub const EXECUTABLE_ALT: &str = "dsnix";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/reasonix.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: schema unfetched; DeepSeek-only; TS/Go drift.
pub const CONSTRAINED_NOTE: &str = "the config.json key reference was linked, not fetched, and the TS 0.x line coexists with the Go rewrite on main-v2 (the new default), so config drift between the lines is unverified; DeepSeek is the only supported provider by design (reasonix.md sections 1-2, 5)";

/// Concrete adapter for Reasonix.
#[derive(Debug, Clone)]
pub struct ReasonixAdapter {
    id: HarnessId,
}

impl ReasonixAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        super::home_dir().map(|home| home.join(".reasonix"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        match Self::default_config_root() {
            Some(root) => {
                let config = root.join("config.json");
                if config.exists() {
                    evidence.push(format!("config.json found at {}", config.display()));
                } else {
                    evidence.push(format!("config.json missing at {}", config.display()));
                }
            }
            None => evidence.push("could not resolve config root (no HOME)".to_owned()),
        }
        if std::path::Path::new(".reasonix").exists() {
            evidence.push(".reasonix project overrides present in cwd".to_owned());
        }
        if std::path::Path::new(".claude").join("skills").exists() {
            evidence.push(
                ".claude/skills present in cwd: reasonix reads Claude-format skills alongside native paths"
                    .to_owned(),
            );
        }
    }
}

impl Default for ReasonixAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ReasonixAdapter {
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

        match super::find_in_path(&[EXECUTABLE, EXECUTABLE_ALT]) {
            Some(path) => {
                evidence.push(format!(
                    "found binary `{}` at {}",
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    path.display()
                ));
                match super::probe_version(&path) {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `--version`"));
                        version = Some(v);
                    }
                    None => evidence.push("version probe failed (timeout or non-zero)".to_owned()),
                }
                binary_path = Some(path);
            }
            None => evidence.push(format!(
                "binaries `{EXECUTABLE}`/`{EXECUTABLE_ALT}` not found in PATH"
            )),
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
        let config = ConfigSurface::new(
            "config.json",
            PathResolver::fallback_only(
                "~/.reasonix/config.json (fixed root; key schema unfetched)",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        let project = ConfigSurface::new(
            "project .reasonix",
            PathResolver::fallback_only("<project>/.reasonix/ (per-project overrides and skills)"),
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![config, project]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "sessions/*".to_owned(),
            "events/*".to_owned(),
            "index/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("fixed root; --dir workspace scoping");
        plan.description = format!(
            " reasonix documents no config-dir relocation: the root stays at ~/.reasonix and \
             config_root {} is informative only; `--dir` retargets the workspace (mid-session \
             switching is unsupported by design) and DeepSeek keys paste on first run; \
             permissions use a per-workspace exact-prefix shell allowlist (reasonix.md sections \
             2, 4, 7)",
            instance.config_root
        );
        plan.shared_state_warnings.push(
            "skills cross-read <project>/.claude/skills and ~/.claude/skills; a second writer on \
             the claude-code tree changes this instance's skill set"
                .to_owned(),
        );
        plan.auth_prerequisites
            .push(crate::adapter::AuthPrerequisite::env_var(
                "DEEPSEEK_API_KEY",
            ));
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.reasonix".to_owned(), ".reasonix".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::FixedPathSingle
            | Isolation::ExplicitConfig
            | Isolation::ProjectScope
            | Isolation::RelocatedRoot
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("reasonix requires isolation fixed_path_single, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "config.json carries MCP servers over stdio, SSE, and Streamable HTTP, but the key name and shape live on a hosted configuration guide that was not fetched; no destination key may be invented (reasonix.md sections 2, 5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism is documented; skills are markdown playbooks in inline or subagent mode (reasonix.md sections 2-3)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, ReasonixAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ReasonixAdapter {
        ReasonixAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-reasonix-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".reasonix-work"))
                .unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::FixedPathSingle,
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
    fn detection_notes_the_dual_line_drift_risk() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("main-v2") || e.contains("constrained"))
        );
    }

    #[test]
    fn config_surfaces_own_no_keys() {
        assert!(
            adapter()
                .config_surfaces()
                .iter()
                .all(|s| s.owned_selectors.is_empty()),
            "the key schema was not fetched"
        );
    }

    #[test]
    fn wrapper_sets_no_env_vars_for_a_fixed_root() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.env_vars.is_empty());
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("jcode").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_fixed_root() {
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
