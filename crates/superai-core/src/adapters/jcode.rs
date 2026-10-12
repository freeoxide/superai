//! jcode adapter: Rust TUI harness with daemon-shared headless sessions,
//! `~/.jcode/config.toml` provider profiles plus `mcp.json`; cross-reads
//! Claude Code and Codex MCP config; no relocation var. Research:
//! docs/harness-configs/jcode.md.

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
pub const HARNESS_ID_STR: &str = "jcode";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "jcode";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "jcode";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/jcode.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside config.toml.
pub const OWNED_SELECTORS: &[&str] = &["provider", "providers", "display"];

/// Owned selectors inside mcp.json.
pub const MCP_OWNED_SELECTORS: &[&str] = &["mcpServers", "servers"];

/// Constrained note: fixed root plus foreign cross-reads.
pub const CONSTRAINED_NOTE: &str = "the root is fixed at ~/.jcode with no relocation var documented, and jcode cross-reads ~/.claude.json, .mcp.json, and ~/.codex/config.toml MCP servers, so mutating those files changes jcode's view (jcode.md sections 2, 4, 6)";

/// Concrete adapter for jcode.
#[derive(Debug, Clone)]
pub struct JcodeAdapter {
    id: HarnessId,
}

impl JcodeAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        super::home_dir().map(|home| home.join(".jcode"))
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
        for file in ["config.toml", "mcp.json"] {
            if root.join(file).exists() {
                evidence.push(format!("{file} found under the root"));
            }
        }
    }
}

impl Default for JcodeAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for JcodeAdapter {
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
                        "version probe failed; `jcode update` is the documented version surface (jcode.md section 6)"
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
        let mut config = ConfigSurface::new(
            "config.toml",
            PathResolver::fallback_only(
                "~/.jcode/config.toml (provider profiles; api_key_env references hold secrets)",
            ),
            DocumentKind::Toml,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        config.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        config.restart_behavior = RestartBehavior::Restart;

        let mut mcp = ConfigSurface::new(
            "mcp.json",
            PathResolver::fallback_only("~/.jcode/mcp.json and <project>/.jcode/mcp.json"),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        mcp.owned_selectors = MCP_OWNED_SELECTORS
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        mcp.restart_behavior = RestartBehavior::Restart;
        vec![config, mcp]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["sessions/*".to_owned(), "daemon/*".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("fixed root; profile and provider-profile isolation");
        plan.description = format!(
            " jcode documents no config-dir relocation: the root stays at ~/.jcode and \
             config_root {} is informative only; `--provider-profile <name>` selects a profile \
             per run and `jcode login --headless` keeps per-identity auth; jcode also imports \
             Codex MCP servers and reads ~/.claude.json, so treat those trees as shared foreign \
             state (jcode.md sections 2, 4-5)",
            instance.config_root
        );
        plan.shared_state_warnings.push(
            "jcode reads ~/.claude.json, .mcp.json, and ~/.codex/config.toml; a second writer on \
             those files changes this instance's MCP view"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.jcode".to_owned(), ".jcode".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::FixedPathSingle
            | Isolation::ExplicitConfig
            | Isolation::RelocatedRoot
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("jcode requires isolation fixed_path_single, got {other}"),
            }),
        }
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            "mcp.json",
            "mcpServers",
            DocumentKind::Json,
            ConfigScope::User,
            RestartBehavior::Restart,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism is documented; swarm roles are configured per session via /agents (jcode.md section 5)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, JcodeAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> JcodeAdapter {
        JcodeAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-jcode-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".jcode-work")).unwrap(),
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
    fn detection_names_the_foreign_cross_reads() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("~/.claude.json") && e.contains("~/.codex"))
        );
    }

    #[test]
    fn wrapper_sets_no_env_vars_for_a_fixed_root() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no relocation var is documented: {:?}",
            plan.env_vars
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
    fn validate_instance_accepts_fixed_root() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::Unsupported;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_decl_targets_mcp_json() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "mcp.json");
        assert_eq!(decl.dest_key, "mcpServers");
        assert!(a.plugin_absence_reason().is_some());
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
