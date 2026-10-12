//! ECA adapter: editor-agnostic LSP-like server (`eca server`) with layered
//! JSON config (`~/.config/eca/config.json`, `.eca/config.json`,
//! `ECA_CONFIG` inline, `extraConfigs`); key schema unpublished. Research:
//! docs/harness-configs/eca.md.

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
pub const HARNESS_ID_STR: &str = "eca";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "ECA (Editor Code Assistant)";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "eca";

/// Env var carrying inline JSON config at server spawn.
pub const INLINE_CONFIG_ENV_VAR: &str = "ECA_CONFIG";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/eca.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: config schema lives on unfetched subpages.
pub const CONSTRAINED_NOTE: &str = "the config.json key schema beyond defaultAgent examples lives on eca.dev subpages that were not fetched, so superai models the layering but owns no keys (eca.md sections 2, 5)";

/// Concrete adapter for ECA.
#[derive(Debug, Clone)]
pub struct EcaAdapter {
    id: HarnessId,
}

impl EcaAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME")
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir).join("eca"));
        }
        super::home_dir().map(|home| home.join(".config").join("eca"))
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
        if std::path::Path::new(".eca").join("config.json").exists() {
            evidence.push(".eca/config.json present in cwd (workspace layer)".to_owned());
        }
    }
}

impl Default for EcaAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for EcaAdapter {
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
                    None => evidence.push(
                        "version probe failed; releases are tagged but no CLI version flag is documented (eca.md section 1)"
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
        let global = ConfigSurface::new(
            "config.json",
            PathResolver::fallback_only("$XDG_CONFIG_HOME/eca/config.json (global layer)"),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        let local = ConfigSurface::new(
            "project config.json",
            PathResolver::fallback_only("<workspace>/.eca/config.json (workspace layer)"),
            DocumentKind::Json,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![global, local]
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
        let mut plan = WrapperPlan::new("--config-file replaces the lookup; ECA_CONFIG inlines");
        plan.description = format!(
            " eca resolves built-in defaults, initializationOptions, {INLINE_CONFIG_ENV_VAR}, \
             global config.json, then workspace .eca/config.json; per-instance isolation points \
             `eca --config-file {}` at the instance root, which replaces the global/local lookup \
             entirely (eca.md section 2)",
            instance.config_root.as_path().join("config.json").display()
        );
        plan.args.push("--config-file".to_owned());
        plan.args.push(
            instance
                .config_root
                .as_path()
                .join("config.json")
                .display()
                .to_string(),
        );
        plan.env_unset.push(INLINE_CONFIG_ENV_VAR.to_owned());
        plan.shared_state_warnings.push(
            format!("an inherited {INLINE_CONFIG_ENV_VAR} outranks the global file and would leak shared config into the instance"),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.config/eca".to_owned(),
            "$XDG_CONFIG_HOME/eca".to_owned(),
            ".eca".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ExplicitConfig
            | Isolation::RelocatedRoot
            | Isolation::ProjectScope
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("eca requires isolation explicit_config, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP resources and prompts are supported as context sources but no MCP config key or file shape is documented on fetched pages (eca.md sections 4, 5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "editor plugins (Emacs, VS Code, Neovim, IntelliJ, Desktop) spawn the server; no ECA-side plugin mechanism is documented (eca.md section 1)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{EcaAdapter, HARNESS_ID_STR, INLINE_CONFIG_ENV_VAR};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> EcaAdapter {
        EcaAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-eca-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".eca-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::ExplicitConfig,
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
    fn detection_notes_the_unpublished_schema() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("constrained") && e.contains("subpages"))
        );
    }

    #[test]
    fn config_surfaces_own_no_keys() {
        assert!(
            adapter()
                .config_surfaces()
                .iter()
                .all(|s| s.owned_selectors.is_empty()),
            "no key list may be invented"
        );
    }

    #[test]
    fn wrapper_unsets_the_inline_config_var() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.args.contains(&"--config-file".to_owned()));
        assert!(plan.env_unset.contains(&INLINE_CONFIG_ENV_VAR.to_owned()));
        assert!(plan.env_vars.is_empty());
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("devin").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_explicit_config() {
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
