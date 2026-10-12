//! `OpenFang` adapter: Rust agent-OS daemon with `~/.openfang/config.toml`,
//! `OPENFANG_HOME` relocation, `[[mcp_servers]]` array entries, secrets only
//! as env-var names. Research: docs/harness-configs/openfang.md.

use std::path::PathBuf;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    McpDestShape, PathResolver, Platform, ProductStatus, RestartBehavior, SurfaceOwnership,
    VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "openfang";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "OpenFang";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "openfang";

/// Env var holding the home directory (config/agents/skills root).
pub const CONFIG_ENV_VAR: &str = "OPENFANG_HOME";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/openfang.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside config.toml.
pub const OWNED_SELECTORS: &[&str] = &[
    "home_dir",
    "data_dir",
    "api_listen",
    "api_key",
    "mcp_servers",
    "a2a",
    "channels",
];

/// Constrained note: pre-1.0 with secrets-by-name semantics.
pub const CONSTRAINED_NOTE: &str = "pre-1.0 by its own warning, and secrets never live in config.toml: api_key_env-style fields hold the NAME of an environment variable that holds the secret (openfang.md sections 2-3)";

/// Concrete adapter for the openfang daemon.
#[derive(Debug, Clone)]
pub struct OpenfangAdapter {
    id: HarnessId,
}

impl OpenfangAdapter {
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
        super::home_dir().map(|home| home.join(".openfang"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        match Self::default_config_root() {
            Some(root) => {
                let config = root.join("config.toml");
                if config.exists() {
                    evidence.push(format!("config.toml found at {}", config.display()));
                } else {
                    evidence.push(format!(
                        "config.toml missing at {} (boots with Anthropic defaults when absent)",
                        config.display()
                    ));
                }
            }
            None => evidence.push("could not resolve config root (no HOME)".to_owned()),
        }
    }
}

impl Default for OpenfangAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for OpenfangAdapter {
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
        let mut config = ConfigSurface::new(
            "config.toml",
            PathResolver::fallback_only(
                "$OPENFANG_HOME/config.toml (default ~/.openfang; Docker sets /data)",
            ),
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        config.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        config.restart_behavior = RestartBehavior::Restart;
        vec![config]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["data/*".to_owned(), "logs/*".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("OPENFANG_HOME whole-root relocation");
        plan.description = format!(
            " openfang resolves its home from {CONFIG_ENV_VAR} (the Docker image pins it to \
             /data, where any other location is silently ignored); config_root {} carries \
             config.toml, agents, and skills; secrets stay in env vars referenced by name from \
             api_key_env-style fields (openfang.md sections 2-3)",
            instance.config_root
        );
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.env_unset = [
            "OPENFANG_LISTEN",
            "OPENFANG_API_KEY",
            "OPENFANG_ALLOW_NO_AUTH",
            "OPENFANG_AGENTS_DIR",
        ]
        .iter()
        .map(|var| (*var).to_owned())
        .collect();
        plan.state_paths
            .push(format!("{CONFIG_ENV_VAR}={}", instance.config_root));
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.openfang".to_owned(), "$OPENFANG_HOME".to_owned()]
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
                reason: format!("openfang requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(
            crate::adapter::McpAdapterDecl::new(
                "config.toml",
                "mcp_servers",
                DocumentKind::Toml,
                ConfigScope::User,
                RestartBehavior::Restart,
            )
            .with_shape(McpDestShape::IdentityList {
                key: "name".to_owned(),
            }),
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "skills install through `openfang skill install` and Hands are HAND.toml directories; no plugin staging path is documented (openfang.md section 5)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_ENV_VAR, HARNESS_ID_STR, OpenfangAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> OpenfangAdapter {
        OpenfangAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-openfang-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".openfang-work"))
                .unwrap(),
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
    fn detection_notes_the_secrets_by_name_rule() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("secrets never live in config.toml"))
        );
    }

    #[test]
    fn wrapper_sets_home_and_unsets_runtime_overrides() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, CONFIG_ENV_VAR);
        assert!(
            plan.env_unset.contains(&"OPENFANG_API_KEY".to_owned()),
            "runtime overrides must not leak across instances"
        );
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("agentty").unwrap();
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
    fn mcp_decl_is_an_identity_list() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "config.toml");
        assert_eq!(
            decl.shape,
            crate::adapter::McpDestShape::IdentityList {
                key: "name".to_owned()
            }
        );
        assert!(a.plugin_absence_reason().is_some());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
