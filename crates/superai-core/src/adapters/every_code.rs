//! Every Code adapter: Codex-CLI community fork (`code`/`coder` binary),
//! `~/.code/config.toml` with `CODE_HOME` relocation, `[mcp_servers.<name>]`
//! tables; reads foreign `~/.codex` but never writes it. Research:
//! docs/harness-configs/every-code.md.

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
pub const HARNESS_ID_STR: &str = "every-code";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Every Code";

/// Unambiguous binary alias: npm installs `code`, but that name collides
/// with the VS Code binary, so detection probes `coder` only.
pub const EXECUTABLE: &str = "coder";

/// Env var overriding the config directory location.
pub const CONFIG_ENV_VAR: &str = "CODE_HOME";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/every-code.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside `config.toml`.
pub const OWNED_SELECTORS: &[&str] = &[
    "model",
    "model_provider",
    "approval_policy",
    "model_reasoning_effort",
    "sandbox_mode",
    "tui.theme",
    "profiles",
    "mcp_servers",
];

/// Constrained note: relocation scope unverified; foreign ~/.codex reads.
pub const CONSTRAINED_NOTE: &str = "CODE_HOME documents only that it overrides the config directory; whether it also moves auth.json and sessions is unverified, and a stale ~/.codex read-compat layer can break codex-cli itself (every-code.md sections 2, 7)";

/// Concrete adapter for Every Code.
#[derive(Debug, Clone)]
pub struct EveryCodeAdapter {
    id: HarnessId,
}

impl EveryCodeAdapter {
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
        super::home_dir().map(|home| home.join(".code"))
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
                    evidence.push(format!("config.toml missing at {}", config.display()));
                }
                if root.join("auth.json").exists() {
                    evidence.push("auth.json found under the root".to_owned());
                }
            }
            None => evidence.push("could not resolve config root (no HOME)".to_owned()),
        }
        if let Some(home) = super::home_dir()
            && home.join(".codex").exists()
        {
            evidence.push(
                "foreign ~/.codex present: every-code reads it and a stale config.toml there can break codex-cli (every-code.md section 2)"
                    .to_owned(),
            );
        }
    }
}

impl Default for EveryCodeAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for EveryCodeAdapter {
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
                    "found binary `{EXECUTABLE}` at {} (unambiguous alias; bare `code` collides with VS Code)",
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
                "binary `{EXECUTABLE}` not found in PATH (npm @just-every/code installs it alongside VS Code's code)"
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
        let mut config = ConfigSurface::new(
            "config.toml",
            PathResolver::new(
                Some("$CODE_HOME/config.toml"),
                Some("$CODE_HOME/config.toml"),
                Some("%USERPROFILE%\\.code\\config.toml"),
                "~/.code/config.toml",
            ),
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        config.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        config.restart_behavior = RestartBehavior::Restart;

        let auth = ConfigSurface::new(
            "auth.json",
            PathResolver::fallback_only(
                "$CODE_HOME/auth.json (ChatGPT login or OPENAI_API_KEY state)",
            ),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::ExternalSecretStore,
        );
        vec![config, auth]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["sessions/*".to_owned(), "auth.json".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("CODE_HOME config-dir relocation");
        plan.description = format!(
            " every-code overrides its config directory with {CONFIG_ENV_VAR}; config_root {} \
             carries config.toml (and, scope unverified, auth.json and sessions); every-code \
             READS foreign ~/.codex but only ever writes ~/.code, so keep ~/.codex read-only \
             foreign state (every-code.md sections 2, 7)",
            instance.config_root
        );
        plan.env_vars
            .push((CONFIG_ENV_VAR.to_owned(), instance.config_root.to_string()));
        plan.state_paths
            .push(format!("{CONFIG_ENV_VAR}={}", instance.config_root));
        plan.shared_state_warnings.push(
            "a stale ~/.codex/config.toml can break codex-cli itself after an every-code round \
             trip; never mirror into it"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.code".to_owned(), "$CODE_HOME".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::RelocatedRoot | Isolation::EnvOnly | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("every-code requires isolation relocated_root, got {other}"),
            }),
        }
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "config.toml",
            "mcp_servers",
            DocumentKind::Toml,
            ConfigScope::User,
            RestartBehavior::Restart,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism is documented; orchestration targets other CLIs via /plan, /solve, /code, and /auto commands (every-code.md section 5)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_ENV_VAR, EXECUTABLE, EveryCodeAdapter, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> EveryCodeAdapter {
        EveryCodeAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-every-code-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".code-work")).unwrap(),
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
    fn detection_probes_only_the_unambiguous_alias() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains(EXECUTABLE) && e.contains("VS Code")),
            "the code-name collision must be visible in evidence: {:?}",
            result.evidence
        );
    }

    #[test]
    fn wrapper_sets_only_code_home() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, CONFIG_ENV_VAR);
    }

    #[test]
    fn wrapper_rejects_mismatched_harness() {
        let mut inst = sample_instance();
        inst.harness = HarnessId::new("codex-cli").unwrap();
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
    fn mcp_decl_targets_config_toml_tables() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "config.toml");
        assert_eq!(decl.dest_key, "mcp_servers");
        assert_eq!(decl.kind, crate::adapter::DocumentKind::Toml);
        assert!(a.plugin_absence_reason().is_some());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
