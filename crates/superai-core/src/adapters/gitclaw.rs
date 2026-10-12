//! Gitagent adapter (ledger id `gitclaw`): the agent dir IS a git repo
//! (`agent.yaml`, `SOUL.md`, `RULES.md`, memory, tools, skills, plugins);
//! isolation is a separate `--dir`; `mcp_servers` in agent.yaml. Research:
//! docs/harness-configs/gitclaw.md.

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
pub const HARNESS_ID_STR: &str = "gitclaw";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Gitagent";

/// Upstream executable name (npm gitagent / @open-gitagent/gitagent).
pub const EXECUTABLE: &str = "gitagent";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/gitclaw.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside agent.yaml.
pub const OWNED_SELECTORS: &[&str] = &["model", "env", "mcp_servers", "config"];

/// Constrained note: agent.yaml schema only partially published.
pub const CONSTRAINED_NOTE: &str = "the README documents the agent-dir tree and a subset of agent.yaml keys, not the full schema; memory/ is git history and plugins resolve from bundled, ~/.gitagent/plugins, and <agent-dir>/.gitagent/plugins (gitclaw.md sections 2, 7)";

/// Concrete adapter for Gitagent.
#[derive(Debug, Clone)]
pub struct GitclawAdapter {
    id: HarnessId,
}

impl GitclawAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        let cwd_agent = std::path::Path::new("agent.yaml");
        if cwd_agent.exists() {
            evidence.push("agent.yaml present in cwd (the cwd is an agent dir)".to_owned());
        }
        if let Some(home) = super::home_dir()
            && home.join(".gitagent").join("plugins").exists()
        {
            evidence.push("global plugin cache present at ~/.gitagent/plugins".to_owned());
        }
    }
}

impl Default for GitclawAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for GitclawAdapter {
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
                        "version probe failed; --version is implied by npm packaging but undocumented (gitclaw.md section 7)"
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
        let mut agent = ConfigSurface::new(
            "agent.yaml",
            PathResolver::fallback_only("<agent-dir>/agent.yaml (model, tools, runtime config)"),
            DocumentKind::Yaml,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        agent.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        agent.restart_behavior = RestartBehavior::Restart;

        let identity = ConfigSurface::new(
            "SOUL.md",
            PathResolver::fallback_only(
                "<agent-dir>/SOUL.md and RULES.md (git-committed identity)",
            ),
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        vec![agent, identity]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec!["memory/*".to_owned(), ".git/*".to_owned()]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("one agent dir per instance via --dir");
        plan.description = format!(
            " gitagent isolation is the agent dir: run `gitagent --dir {}` so the instance owns \
             its own agent.yaml, memory git history, skills, and MCP; nothing is user-global \
             except the ~/.gitagent/plugins cache; provider keys stay per-process env vars \
             (gitclaw.md sections 3, 6)",
            instance.config_root
        );
        plan.args.push("--dir".to_owned());
        plan.args.push(instance.config_root.to_string());
        plan.env_unset.push("GITHUB_TOKEN".to_owned());
        plan.env_unset.push("GIT_TOKEN".to_owned());
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec!["~/.gitagent".to_owned(), "agent.yaml".to_owned()]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ExplicitConfig
            | Isolation::ProjectScope
            | Isolation::RelocatedRoot
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("gitclaw requires isolation explicit_config, got {other}"),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        super::skill_modes_link_first()
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(
            crate::adapter::McpAdapterDecl::new(
                "agent.yaml",
                "mcp_servers",
                DocumentKind::Yaml,
                ConfigScope::ProjectWorkspace,
                RestartBehavior::Restart,
            )
            .with_read_only(
                "yaml writes refuse (LossyWrite) until a preserving codec exists; inspect/diff only; agent.yaml is the agent's git-committed identity",
            ),
        )
    }

    fn plugin_decl(&self) -> Option<crate::adapter::PluginAdapterDecl> {
        Some(crate::adapter::PluginAdapterDecl {
            source_hint: "git marketplace or local <agent-dir>/plugins".to_owned(),
            dest_file: ".gitagent/plugins".to_owned(),
            dest_key: None,
            dest_dir: Some(".gitagent/plugins".to_owned()),
            discovery_manifest: None,
            kind: crate::adapter::PluginKind::DirectoryBundle,
            requires_execution: false,
            enable_semantics: "presence of the directory under the agent dir's plugin root"
                .to_owned(),
            disable_semantics: "remove the plugin directory or remove it from agent.yaml config"
                .to_owned(),
            remove_semantics: "delete <agent-dir>/.gitagent/plugins/<name>".to_owned(),
            dependency_effect: "plugins resolve bundled, then ~/.gitagent/plugins, then the \
                                agent dir's own plugin root"
                .to_owned(),
            restart: RestartBehavior::Restart,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{GitclawAdapter, HARNESS_ID_STR};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> GitclawAdapter {
        GitclawAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-gitclaw-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".gitagent-work"))
                .unwrap(),
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
    fn detection_explains_the_agent_dir_model() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("constrained") && e.contains("plugins"))
        );
    }

    #[test]
    fn wrapper_points_the_dir_flag_at_the_instance_root() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert!(plan.args.contains(&"--dir".to_owned()));
        assert!(
            plan.env_unset.contains(&"GITHUB_TOKEN".to_owned()),
            "repo-mode tokens must not leak across instances"
        );
        assert!(plan.env_vars.is_empty());
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
    fn validate_instance_accepts_explicit_config() {
        adapter().validate_instance(&sample_instance()).unwrap();
        let mut inst = sample_instance();
        inst.isolation = Isolation::Unsupported;
        assert!(adapter().validate_instance(&inst).is_err());
    }

    #[test]
    fn mcp_decl_targets_agent_yaml_and_plugins_stage_as_directories() {
        let a = adapter();
        let decl = a.mcp_decl().unwrap();
        assert_eq!(decl.dest_file, "agent.yaml");
        assert_eq!(decl.dest_key, "mcp_servers");
        assert_eq!(decl.kind, crate::adapter::DocumentKind::Yaml);
        assert!(decl.read_only.is_some(), "yaml dests are inspect-only");
        let plugin = a.plugin_decl().unwrap();
        assert_eq!(plugin.kind.to_string(), "directory_bundle");
        assert!(!plugin.requires_execution);
        assert_eq!(plugin.dest_dir.as_deref(), Some(".gitagent/plugins"));
        assert!(
            a.supported_operations()
                .iter()
                .all(|(_, s)| *s == AdapterSupport::Constrained)
        );
    }
}
