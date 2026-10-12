//! vix adapter: token-efficient daemon (`vixd`) plus isolated instances;
//! `~/.vix/` and `./.vix/` providers.json merged over embedded defaults,
//! settings.json workflows (schema deferred); macOS/Linux only. Research:
//! docs/harness-configs/vix.md.

use std::path::PathBuf;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, Arch, ConfigScope, ConfigSurface, DetectionResult, DocumentKind, Os,
    PathResolver, Platform, ProductStatus, SurfaceOwnership, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "vix";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "vix";

/// Instance binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "vix";

/// Daemon binary name (start the daemon, then spawn instances).
pub const DAEMON_EXECUTABLE: &str = "vixd";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/vix.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside providers.json.
pub const OWNED_SELECTORS: &[&str] = &["providers"];

/// Constrained note: provider and workflow schemas unfetched.
pub const CONSTRAINED_NOTE: &str = "the providers.json field reference (PROVIDERS.md) and the settings.json workflow schema were linked, not fetched, so superai owns only the providers container (vix.md sections 2, 5)";

/// Concrete adapter for vix.
#[derive(Debug, Clone)]
pub struct VixAdapter {
    id: HarnessId,
}

impl VixAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        super::home_dir().map(|home| home.join(".vix"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        let Some(root) = Self::default_config_root() else {
            evidence.push("could not resolve config root (no HOME)".to_owned());
            return;
        };
        if !root.exists() {
            evidence.push(format!("config root missing at {}", root.display()));
            return;
        }
        evidence.push(format!("config root exists at {}", root.display()));
        for file in ["providers.json", "settings.json"] {
            if root.join(file).exists() {
                evidence.push(format!("{file} found under the root"));
            }
        }
        if std::path::Path::new(".vix").exists() {
            evidence.push(".vix project dir present in cwd (project providers layer)".to_owned());
        }
    }
}

impl Default for VixAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for VixAdapter {
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
        vec![
            Platform::new(Os::Linux, Arch::Any),
            Platform::new(Os::Macos, Arch::Any),
        ]
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

        match super::find_in_path_dir_first(&[EXECUTABLE, DAEMON_EXECUTABLE]) {
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
                    None => evidence.push(
                        "version probe failed; no version convention is documented (vix.md section 5)"
                            .to_owned(),
                    ),
                }
                binary_path = Some(path);
            }
            None => evidence.push(format!(
                "binaries `{EXECUTABLE}`/`{DAEMON_EXECUTABLE}` not found in PATH"
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
        let mut providers = ConfigSurface::new(
            "providers.json",
            PathResolver::fallback_only(
                "~/.vix/providers.json and ./.vix/providers.json merged over embedded defaults",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        providers.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();

        let workflows = ConfigSurface::new(
            "settings.json",
            PathResolver::fallback_only(
                "~/.vix/settings.json (multi-phase workflows; schema deferred upstream)",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        vec![providers, workflows]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["sessions/*".to_owned(), "watchers/*".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("isolated vix instances over a shared vixd daemon");
        plan.description = format!(
            " vix's documented concurrency model is a daemon (vixd) plus N isolated instances; \
             each instance already isolates, so config_root {} holds the instance's providers \
             and settings layer; no config-dir relocation var is documented for ~/.vix itself, \
             and the daemon is shared per host (vix.md sections 1-2)",
            instance.config_root
        );
        plan.env_unset.push("ANTHROPIC_API_KEY".to_owned());
        plan.shared_state_warnings.push(
            "vix writes its own scheduled jobs, watchers, and alerts; expect background work \
             started by the agent itself, and the vixd daemon serves every instance"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.vix".to_owned(), ".vix".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::DaemonService | Isolation::RelocatedRoot | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("vix requires isolation daemon_service, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP servers are a listed capability but their configuration files are not spelled out in the README; the path is unverified (vix.md sections 4-5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism is mentioned; workflows in settings.json and Tree-sitter tooling are the extension surface (vix.md sections 2, 4)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, VixAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> VixAdapter {
        VixAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-vix-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".vix-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::DaemonService,
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
    fn supported_platforms_exclude_windows() {
        let os_set: std::collections::HashSet<String> = adapter()
            .supported_platforms()
            .iter()
            .map(|p| p.os.to_string())
            .collect();
        assert!(os_set.contains("linux") && os_set.contains("macos"));
        assert!(!os_set.contains("windows"), "macOS and Linux only upstream");
    }

    #[test]
    fn wrapper_sets_no_invented_env_vars() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no relocation var is documented: {:?}",
            plan.env_vars
        );
        assert!(
            plan.env_unset.contains(&"ANTHROPIC_API_KEY".to_owned()),
            "the documented auth var must not leak across instances"
        );
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("hax").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_daemon_service() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::FixedPathSingle;
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
