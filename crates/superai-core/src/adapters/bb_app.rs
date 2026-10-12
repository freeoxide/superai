//! bb (get-bb) adapter: agent IDE driving externally authenticated CLIs;
//! `BB_DATA_DIR` whole-root relocation with `~/.bb/config.json` +
//! `env.json` + `client.json`; MCP unverified. Research:
//! docs/harness-configs/bb-app.md.

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
pub const HARNESS_ID_STR: &str = "bb-app";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "bb (get-bb)";

/// Binary name resolved on PATH during detection (npm package bb-app).
pub const EXECUTABLE: &str = "bb-app";

/// Env var relocating the whole data root (default `~/.bb`).
pub const CONFIG_ENV_VAR: &str = "BB_DATA_DIR";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/bb-app.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: MCP unverified and plugin state is `SQLite`.
pub const CONSTRAINED_NOTE: &str = "MCP support is unverified (absent from the configuration doc) and per-plugin state lives in SQLite data.db files, so writes stop at the JSON config layer (bb-app.md sections 4-5)";

/// Concrete adapter for bb.
#[derive(Debug, Clone)]
pub struct BbAppAdapter {
    id: HarnessId,
}

impl BbAppAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_data_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(CONFIG_ENV_VAR)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir));
        }
        super::home_dir().map(|home| home.join(".bb"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        let Some(root) = Self::default_data_root() else {
            evidence.push("could not resolve data root (no HOME)".to_owned());
            return;
        };
        if !root.exists() {
            evidence.push(format!("data root missing at {}", root.display()));
            return;
        }
        evidence.push(format!("data root exists at {}", root.display()));
        for file in ["config.json", "env.json", "client.json"] {
            let path = root.join(file);
            evidence.push(if path.exists() {
                format!("{file} found at {}", path.display())
            } else {
                format!("{file} missing at {}", path.display())
            });
        }
    }
}

impl Default for BbAppAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for BbAppAdapter {
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
                        "version probe failed; the npm package version and bb-app-update.json are the documented version surfaces".to_owned(),
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
        let root_fallback = "$BB_DATA_DIR (default ~/.bb)";
        let config = ConfigSurface::new(
            "config.json",
            PathResolver::fallback_only(&format!("{root_fallback}/config.json")),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        let env = ConfigSurface::new(
            "env.json",
            PathResolver::fallback_only(&format!(
                "{root_fallback}/env.json (provider credentials)"
            )),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        let client = ConfigSurface::new(
            "client.json",
            PathResolver::fallback_only(&format!(
                "{root_fallback}/client.json (SSH target mappings)"
            )),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        let agents = ConfigSurface::new(
            "AGENTS.md",
            PathResolver::fallback_only("$BB_DATA_DIR/AGENTS.md and <workspace>/.bb/AGENTS.md"),
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![config, env, client, agents]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "plugins/*/data.db".to_owned(),
            "plugins/*/secrets/*".to_owned(),
            "plugins/*/logs/*".to_owned(),
            "bb-app-runtime.json".to_owned(),
            "theme/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("BB_DATA_DIR whole-root relocation");
        plan.description = format!(
            " bb relocates its whole data root with {CONFIG_ENV_VAR}; config_root {} carries \
             config.json, env.json, client.json, skills, and plugins; pair it with distinct \
             BB_SERVER_PORT/BB_HOST_DAEMON_PORT when two instances run side by side, and let the \
             wrapped CLIs keep their own config trees (bb-app.md sections 4, 6)",
            instance.config_root
        );
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.state_paths
            .push(format!("{CONFIG_ENV_VAR}={}", instance.config_root));
        plan.shared_state_warnings.push(
            "bb-community marketplace catalogs and bundled plugins (GitHub, Docs, Memory, Tasks) \
             are shared app state, not per-instance"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.bb".to_owned(),
            "$BB_DATA_DIR".to_owned(),
            ".bb".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot | Isolation::EnvOnly | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("bb-app requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no MCP configuration section exists in the configuration doc; whether bb surfaces MCP to its agents is unverified (bb-app.md sections 4, 7)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "plugins install through bb marketplaces and bundled app state under the data root; no filesystem staging path for third-party plugin code is documented (bb-app.md section 5)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{BbAppAdapter, CONFIG_ENV_VAR, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> BbAppAdapter {
        BbAppAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-bb-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".bb-work")).unwrap(),
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
    fn detection_carries_the_mcp_unverified_note() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("constrained") && e.contains("MCP support is unverified"))
        );
    }

    #[test]
    fn wrapper_sets_only_the_data_dir_var() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, CONFIG_ENV_VAR);
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("jules").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn mirror_exclusions_keep_sqlite_plugin_state_out() {
        let patterns = adapter().plan_mirror_exclusions();
        assert!(patterns.iter().any(|p| p.contains("data.db")));
        assert!(patterns.iter().any(|p| p.contains("secrets")));
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
        assert!(a.plugin_decl().is_none());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
