//! zaly adapter: hackable alpha terminal agent with `ZALY_ROOT` and
//! per-key `ZALY_{CONFIG,DATA,CACHE,STATE,LOG,TMP}` overrides (source-
//! verified path module), permission presets, resource packs and TS
//! plugins; no MCP yet. Research: docs/harness-configs/zaly.md.

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
pub const HARNESS_ID_STR: &str = "zaly";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "zaly";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "zaly";

/// Env var holding the single root for config/data/state/log/cache/tmp.
pub const CONFIG_ENV_VAR: &str = "ZALY_ROOT";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/zaly.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: alpha with unpublished config schema.
pub const CONSTRAINED_NOTE: &str = "alpha by upstream's own warning (APIs, configuration, and session formats may change before 1.0); the settings file name and key schema inside the config dir are unpublished (zaly.md sections 2, 5)";

/// Concrete adapter for zaly.
#[derive(Debug, Clone)]
pub struct ZalyAdapter {
    id: HarnessId,
}

impl ZalyAdapter {
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
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME")
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir).join("zaly"));
        }
        super::home_dir().map(|home| home.join(".config").join("zaly"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        let Some(root) = Self::default_config_root() else {
            evidence.push("could not resolve config root (no HOME)".to_owned());
            return;
        };
        if root.exists() {
            evidence.push(format!("config root exists at {}", root.display()));
            if root.join("models.json").exists() {
                evidence.push("models.json found (ModelProvider overrides)".to_owned());
            }
        } else {
            evidence.push(format!("config root missing at {}", root.display()));
        }
        if std::path::Path::new(".zaly").exists() {
            evidence.push(
                ".zaly project dir present in cwd (project scope also reads .agents/)".to_owned(),
            );
        }
    }
}

impl Default for ZalyAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ZalyAdapter {
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
                    None => evidence.push(
                        "version probe failed; the npm version is the only documented pin (zaly.md section 5)"
                            .to_owned(),
                    ),
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
        let models = ConfigSurface::new(
            "models.json",
            PathResolver::fallback_only(
                "$ZALY_ROOT or ~/.config/zaly/models.json (ModelProvider overrides)",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        let project = ConfigSurface::new(
            "project .zaly",
            PathResolver::fallback_only(
                "<project>/.zaly/ (project config root; .agents/ also read)",
            ),
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![models, project]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "sessions/*".to_owned(),
            "plugins/*".to_owned(),
            "state.json".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("ZALY_ROOT single-root relocation");
        plan.description = format!(
            " zaly resolves one root from {CONFIG_ENV_VAR} covering config, data, state, log, \
             cache, and tmp; per-key ZALY_{{CONFIG,DATA,CACHE,STATE,LOG,TMP}} overrides and \
             ZALY_APPNAME exist for finer splits, so unset them inside the wrapper to keep \
             config_root {} whole; alpha warning: config and session formats may change before \
             1.0 (zaly.md sections 2, 5)",
            instance.config_root
        );
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.env_unset = [
            "ZALY_CONFIG",
            "ZALY_DATA",
            "ZALY_CACHE",
            "ZALY_STATE",
            "ZALY_LOG",
            "ZALY_TMP",
            "ZALY_APPNAME",
        ]
        .iter()
        .map(|var| (*var).to_owned())
        .collect();
        plan.state_paths
            .push(format!("{CONFIG_ENV_VAR}={}", instance.config_root));
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.config/zaly".to_owned(),
            "$ZALY_ROOT".to_owned(),
            ".zaly".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot
            | Isolation::EnvOnly
            | Isolation::ProjectScope
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("zaly requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP is not mentioned anywhere in the README; nothing to document yet at 0.0.x (zaly.md sections 4-5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "resource packs and TypeScript plugins load at user or project scope but their manifest formats are unpublished alpha (zaly.md sections 4-5)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_ENV_VAR, HARNESS_ID_STR, ZalyAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ZalyAdapter {
        ZalyAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-zaly-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".zaly-work")).unwrap(),
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
        assert_eq!(a.product_status(), entry.product_status);
        assert_eq!(a.last_verified_date(), entry.last_verified);
    }

    #[test]
    fn detection_carries_the_alpha_warning() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("alpha") && e.contains("1.0"))
        );
    }

    #[test]
    fn wrapper_sets_the_root_and_unsets_per_key_overrides() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, CONFIG_ENV_VAR);
        assert!(plan.env_unset.contains(&"ZALY_DATA".to_owned()));
        assert!(plan.env_unset.contains(&"ZALY_APPNAME".to_owned()));
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("omp").unwrap();
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
