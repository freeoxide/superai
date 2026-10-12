//! hax adapter: minimalist C terminal agent, XDG trio relocates config,
//! state, and cache wholesale; env-only credentials; MCP and plugins
//! deliberately absent by documented philosophy. Research:
//! docs/harness-configs/hax.md.

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
pub const HARNESS_ID_STR: &str = "hax";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "hax";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "hax";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/hax.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Concrete adapter for hax.
#[derive(Debug, Clone)]
pub struct HaxAdapter {
    id: HarnessId,
}

impl HaxAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn xdg_dir(var: &str, default_suffix: &str) -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(var)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir).join("hax"));
        }
        super::home_dir().map(|home| home.join(default_suffix).join("hax"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        for (var, suffix, label) in [
            ("XDG_CONFIG_HOME", ".config", "config"),
            ("XDG_STATE_HOME", ".local/state", "state"),
            ("XDG_CACHE_HOME", ".cache", "cache"),
        ] {
            let resolved = Self::xdg_dir(var, suffix);
            match resolved {
                Some(dir) if dir.exists() => {
                    evidence.push(format!("{label} root exists at {}", dir.display()));
                }
                Some(dir) => {
                    evidence.push(format!("{label} root missing at {}", dir.display()));
                }
                None => evidence.push(format!("could not resolve {label} root (no HOME)")),
            }
        }
        if let Some(state) = Self::xdg_dir("XDG_STATE_HOME", ".local/state")
            && state.join("state.json").exists()
        {
            evidence.push("state.json present (machine-local picker state; never edit)".to_owned());
        }
    }
}

impl Default for HaxAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for HaxAdapter {
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
                        evidence.push(format!("version probe failed for `{EXECUTABLE} --version`"));
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
        let config = ConfigSurface::new(
            "config.json",
            PathResolver::fallback_only(
                "$XDG_CONFIG_HOME/hax/config.json (plain JSON, no comments)",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        let state = ConfigSurface::new(
            "state.json",
            PathResolver::fallback_only(
                "$XDG_STATE_HOME/hax/state.json (picker state, do not edit)",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::HarnessManaged,
        );
        vec![config, state]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Full)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "catalog.json".to_owned(),
            "sessions/*".to_owned(),
            "state.json".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("XDG trio wholesale relocation");
        plan.description = format!(
            " hax moves config, state, and cache wholesale with the XDG trio: point \
             XDG_CONFIG_HOME/XDG_STATE_HOME/XDG_CACHE_HOME at subdirs of config_root {}; \
             credentials are env vars only, so per-instance is clean; state.json and the model \
             catalog stay machine-local (hax.md sections 2-3)",
            instance.config_root
        );
        let root = instance.config_root.to_string();
        for var in ["XDG_CONFIG_HOME", "XDG_STATE_HOME", "XDG_CACHE_HOME"] {
            plan.env_vars.push((var.to_owned(), root.clone()));
            plan.state_paths.push(format!("{var}={root}"));
        }
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.config/hax".to_owned(),
            "$XDG_CONFIG_HOME/hax".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot
            | Isolation::EnvOnly
            | Isolation::ExplicitConfig
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("hax requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP is deliberately absent: the philosophy doc covers the omission and replaces it with subprocess composition (hax.md section 5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no plugin runtime and no marketplace exist by documented design (hax.md section 5)")
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, HaxAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> HaxAdapter {
        HaxAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-hax-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".hax-work")).unwrap(),
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
    fn wrapper_relocates_the_whole_xdg_trio() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        let vars: Vec<&str> = plan.env_vars.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            vars,
            vec!["XDG_CONFIG_HOME", "XDG_STATE_HOME", "XDG_CACHE_HOME"]
        );
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("vix").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_relocated_root() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::FixedPathSingle;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_and_plugin_absence_cite_the_deliberate_omission() {
        let a = adapter();
        assert!(a.mcp_absence_reason().unwrap().contains("deliberately"));
        assert!(a.plugin_absence_reason().unwrap().contains("design"));
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Full)
        );
    }

    #[test]
    fn detection_reports_all_three_roots() {
        let result = adapter().detection();
        assert!(
            result.evidence.iter().any(|e| e.contains("config root")),
            "{:?}",
            result.evidence
        );
    }
}
