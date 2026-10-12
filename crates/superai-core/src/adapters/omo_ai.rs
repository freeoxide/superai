//! `OmO` adapter (`omo-ai`): standalone `omo` CLI on a pi fork (senpi), one
//! unified `~/.omo/omo.jsonc` with a project layer walk to `$HOME`,
//! `OMO_PROFILE` layering, migration engine with backups; the `[codex]`
//! block writes Codex settings. Research: docs/harness-configs/omo-ai.md.

use std::path::PathBuf;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, RestartBehavior, SurfaceOwnership, VersionResolution,
    WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "omo-ai";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "OmO";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "omo";

/// Env var selecting the `profiles.<name>` layer.
pub const PROFILE_ENV_VAR: &str = "OMO_PROFILE";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/omo-ai.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside the user layer.
pub const OWNED_SELECTORS: &[&str] = &["mcp"];

/// Constrained note: schema not fully extracted; [codex] cross-writes.
pub const CONSTRAINED_NOTE: &str = "the full omo.jsonc key schema was not extracted (agents, categories, provider chains sections), and the [codex] block writes into Codex-harness settings, coupling this adapter to codex-cli (omo-ai.md sections 2, 6)";

/// Concrete adapter for the omo CLI.
#[derive(Debug, Clone)]
pub struct OmoAiAdapter {
    id: HarnessId,
}

impl OmoAiAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        super::home_dir().map(|home| home.join(".omo"))
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
            let user_layer = root.join("omo.jsonc").exists() || root.join("omo.json").exists();
            if user_layer {
                evidence.push("user layer omo.json[c] found".to_owned());
            }
        } else {
            evidence.push(format!("config root missing at {}", root.display()));
        }
        if std::path::Path::new(".omo").join("omo.jsonc").exists() {
            evidence.push(".omo/omo.jsonc present in cwd (nearest project layer wins)".to_owned());
        }
    }
}

impl Default for OmoAiAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for OmoAiAdapter {
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
                    "found binary `{EXECUTABLE}` at {}",
                    path.display()
                ));
                match super::probe_version(&path) {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `{EXECUTABLE} --version`"));
                        version = Some(v);
                    }
                    None => evidence.push(format!(
                        "version probe failed for `{EXECUTABLE} --version`"
                    )),
                }
                binary_path = Some(path);
            }
            None => evidence.push(format!(
                "binary `{EXECUTABLE}` not found in PATH (npm package is omo-ai; the `omo` npm name is someone else's)"
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
        let mut user = ConfigSurface::new(
            "omo.jsonc",
            PathResolver::fallback_only("~/.omo/omo.jsonc (user layer; omo.json accepted)"),
            DocumentKind::Jsonc,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        user.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        user.restart_behavior = RestartBehavior::Restart;

        let project = ConfigSurface::new(
            "project omo.jsonc",
            PathResolver::fallback_only(".omo/omo.json[c] walked cwd to $HOME; nearest wins"),
            DocumentKind::Jsonc,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![user, project]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["sessions/*".to_owned(), "migration-backup-*".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("profile layering over a shared ~/.omo root");
        plan.description = format!(
            " omo reads one unified ~/.omo/omo.jsonc plus project .omo/ layers walked to $HOME; \
             no root-relocation var is documented, so config_root {} is informative only and \
             per-instance separation is {PROFILE_ENV_VAR} profile layers; mcp_env_allowlist is \
             honored only from the user layer, and project layers cannot extend it \
             (omo-ai.md sections 2, 6)",
            instance.config_root
        );
        plan.env_vars
            .push((PROFILE_ENV_VAR.to_owned(), "superai".to_owned()));
        plan.shared_state_warnings.push(
            "subscription logins (/login) are per-user and shared across profiles; the [codex] \
             block writes into Codex settings, coupling instances to the codex-cli tree"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.omo".to_owned(), ".omo".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ProjectScope
            | Isolation::ExplicitConfig
            | Isolation::RelocatedRoot
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("omo-ai requires isolation project_scope, got {other}"),
            }),
        }
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(
            crate::adapter::McpAdapterDecl::new(
                "omo.jsonc",
                "mcp",
                DocumentKind::Jsonc,
                ConfigScope::User,
                RestartBehavior::Restart,
            )
            .with_read_only(
                "jsonc writes refuse (LossyWrite) until a preserving codec exists; inspect/diff only; mcp_env_allowlist is honored from the user layer alone",
            ),
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism beyond built-in skills is documented; the [opencode] block runs inside OpenCode's own plugin system, owned by the opencode adapter (omo-ai.md sections 2, 4)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, OmoAiAdapter, PROFILE_ENV_VAR};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> OmoAiAdapter {
        OmoAiAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-omo-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".omo-work")).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::ProjectScope,
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
    fn detection_names_the_npm_name_trap() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("omo-ai") && e.contains("someone else's"))
        );
    }

    #[test]
    fn wrapper_selects_a_profile_layer() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, PROFILE_ENV_VAR);
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("oh-my-opencode").unwrap();
        match adapter().plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_accepts_project_scope() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::Unsupported;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_decl_targets_the_unified_jsonc() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "omo.jsonc");
        assert_eq!(decl.dest_key, "mcp");
        assert_eq!(decl.kind, crate::adapter::DocumentKind::Jsonc);
        assert!(decl.read_only.is_some(), "jsonc dests are inspect-only");
        assert!(a.plugin_absence_reason().is_some());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
