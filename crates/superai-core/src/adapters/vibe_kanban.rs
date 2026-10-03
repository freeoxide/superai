//! Vibe Kanban adapter: orchestrator GUI over ten harnesses with profiles,
//! env injection, MCP passthrough, and git worktrees; `MigrationOnly`.

use std::path::{Path, PathBuf};
use std::time::Duration;

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
pub const HARNESS_ID_STR: &str = "vibe-kanban";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Vibe Kanban";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "vibe-kanban";

/// Alternative binary name (via npx).
pub const EXECUTABLE_ALT: &str = "vk";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/orchestrators.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Migration tip: sunsetting, now community maintained.
pub const MIGRATION_TIP: &str = "Vibe Kanban sunsetting as company product, continuing as community-maintained OSS (Apache-2.0, v0.1.44, github.com/BloopAI/vibe-kanban): orchestrator profiles/env/MCP/worktrees; export agent profiles (env ANTHROPIC_BASE_URL/ANTHROPIC_AUTH_TOKEN overrides), MCP JSON written into harness global configs, .vibe-kanban-workspaces/ worktrees, migrate to conductor/sculptor or direct harness usage";

/// Community maintained flag.
pub const COMMUNITY_MAINTAINED: &str = "community-maintained OSS (Apache-2.0)";

/// 5s probe budget: the npx entrypoint (bash + npx + node) can exceed the 2s native-binary budget on cold start.
pub const VERSION_PROBE_BUDGET: Duration = Duration::from_secs(5);

/// Concrete adapter for Vibe Kanban (`MigrationOnly`, `project_scope`): detect/inspect/backup/export only, no new instances.
#[derive(Debug, Clone)]
pub struct VibeKanbanAdapter {
    id: HarnessId,
}

impl VibeKanbanAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn probe_version(binary: &Path) -> Option<String> {
        // Output may carry a git describe suffix (`1.2.3/abcdef`), which the
        // shared parser's prefix scan reduces to the semver core.
        super::parse_version_output(&super::run_capturing(
            binary,
            &["--version"],
            VERSION_PROBE_BUDGET,
        )?)
    }

    fn workspaces_dir() -> Option<PathBuf> {
        let cwd_ws = Path::new(".vibe-kanban-workspaces");
        if cwd_ws.exists() {
            return Some(cwd_ws.to_path_buf());
        }
        let home = super::home_dir()?;
        Some(home.join(".vibe-kanban-workspaces"))
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "detection branches are explicit for evidence"
    )]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("sunset → {COMMUNITY_MAINTAINED}, MigrationOnly"));
        evidence.push(MIGRATION_TIP.to_owned());
        match Self::workspaces_dir() {
            Some(dir) => {
                if dir.exists() {
                    evidence.push(format!("workspaces dir exists at {}", dir.display()));
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        let count = entries.count();
                        evidence.push(format!("workspaces dir contains {count} entries"));
                    }
                } else {
                    evidence.push(format!("workspaces dir missing at {}", dir.display()));
                }
            }
            None => evidence.push("could not resolve workspaces dir (no HOME/cwd)".to_owned()),
        }
        if Path::new(".vibe-kanban").exists() || Path::new(".vibe-kanban-workspaces").exists() {
            evidence.push("repo-local .vibe-kanban* present".to_owned());
        }
        // Profiles are documented orchestrator behaviour, not a file.
        evidence.push("agent profiles: claude-code/codex/gemini-cli etc with env ANTHROPIC_BASE_URL/AUTH_TOKEN, mcpServers written into harness global configs".to_owned());
        for var in ["PORT", "HOST", "MCP_HOST", "MCP_PORT", "VK_ALLOWED_ORIGINS"] {
            if let Ok(val) = std::env::var(var)
                && !val.trim().is_empty()
            {
                evidence.push(format!("{var} set to {val}"));
            } else {
                evidence.push(format!("{var} not set"));
            }
        }
        evidence.push("ten harnesses: claude-code, codex, copilot-cli, gemini-cli, amp, cursor, opencode, droid, ccr, qwen-code; all must be pre-installed on PATH".to_owned());
    }
}

impl Default for VibeKanbanAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for VibeKanbanAdapter {
    fn id(&self) -> HarnessId {
        self.id.clone()
    }

    fn display_name(&self) -> &str {
        DISPLAY_NAME
    }

    fn product_status(&self) -> ProductStatus {
        ProductStatus::Sunset
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

        if let Some(path) = super::find_in_path(&[EXECUTABLE, EXECUTABLE_ALT]) {
            evidence.push(format!(
                "found binary `{}` at {}",
                EXECUTABLE,
                path.display()
            ));
            match Self::probe_version(&path) {
                Some(v) => {
                    evidence.push(format!("version `{v}` via `{EXECUTABLE} --version`"));
                    version = Some(v);
                }
                None => {
                    evidence.push(format!(
                        "version probe failed for `{EXECUTABLE} --version` (timeout or non-zero)"
                    ));
                }
            }
            binary_path = Some(path);
        } else {
            evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            evidence.push("try `npx vibe-kanban --version` for npx entrypoint".to_owned());
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), version.is_some());

        let confidence =
            super::detection_confidence(binary_path.is_some(), version.is_some(), false);

        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        if let Some(v) = detection.version.clone() {
            let mut notes = Vec::new();
            notes.push(format!("detected vibe-kanban version {v}"));
            notes.push(format!("mapped to schema version {SCHEMA_VERSION_STR}"));
            notes.push(format!("sunset → {COMMUNITY_MAINTAINED}"));
            let mut res =
                VersionResolution::new(Some(v), Some(SCHEMA_VERSION_STR.to_owned()), true);
            res.notes = notes;
            res
        } else {
            let mut res = VersionResolution::unknown();
            res.notes.clone_from(&detection.evidence);
            res.notes.push(format!("migration tip: {MIGRATION_TIP}"));
            res
        }
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let workspaces_resolver = PathResolver::new(
            Some(".vibe-kanban-workspaces/<vk-*>/ (git worktree per workspace, configurable)"),
            Some(".vibe-kanban-workspaces/<vk-*>/ (git worktree)"),
            Some(".vibe-kanban-workspaces\\<vk-*>\\ (worktree)"),
            ".vibe-kanban-workspaces/ (configurable via Settings → General → Workspace Directory, branch vk/*)",
        );
        let mut workspaces = ConfigSurface::new(
            "worktrees (.vibe-kanban-workspaces)",
            workspaces_resolver,
            DocumentKind::Opaque,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::HarnessManaged,
        );
        workspaces.precedence = 12;
        workspaces.backup_required = false;
        workspaces.restart_behavior = RestartBehavior::None;
        surfaces.push(workspaces);

        let profiles_resolver = PathResolver::fallback_only(
            "agent profiles JSON (per-agent reusable: plan/sandbox/model/provider, env ANTHROPIC_BASE_URL overrides)",
        );
        let mut profiles = ConfigSurface::new(
            "agent profiles",
            profiles_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        profiles.precedence = 10;
        profiles.owned_selectors = vec![
            "agent".to_owned(),
            "environmentVariables".to_owned(),
            "model".to_owned(),
            "sandbox".to_owned(),
        ];
        profiles.backup_required = true;
        surfaces.push(profiles);

        let mcp_resolver = PathResolver::fallback_only(
            "MCP `{\"mcpServers\":{…}}` per-agent, written into harness global config (persists outside VK)",
        );
        let mut mcp = ConfigSurface::new(
            "mcpServers (per-agent, harness-global)",
            mcp_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::HarnessManaged,
        );
        mcp.precedence = 11;
        mcp.owned_selectors = vec!["mcpServers".to_owned()];
        mcp.backup_required = true;
        surfaces.push(mcp);

        let project_resolver = PathResolver::fallback_only(
            ".vibe-kanban / project settings (dev-server/setup/cleanup scripts, parallel setup toggle)",
        );
        let mut project = ConfigSurface::new(
            "project settings",
            project_resolver,
            DocumentKind::TextFragment,
            ConfigScope::ProjectWorkspace,
            SurfaceOwnership::UserEditable,
        );
        project.precedence = 14;
        project.backup_required = false;
        surfaces.push(project);

        let remote_resolver = PathResolver::fallback_only(
            "remote-access via cloud.vibekanban.com pairing, env VIBEKANBAN_REMOTE_JWT_SECRET / VK_ALLOWED_ORIGINS",
        );
        let mut remote = ConfigSurface::new(
            "remote access (cloud pairing)",
            remote_resolver,
            DocumentKind::Opaque,
            ConfigScope::Internal,
            SurfaceOwnership::HarnessManaged,
        );
        remote.precedence = 0;
        remote.backup_required = false;
        surfaces.push(remote);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        vec![
            ("detect".to_owned(), AdapterSupport::MigrationOnly),
            ("read_config".to_owned(), AdapterSupport::MigrationOnly),
            ("write_config".to_owned(), AdapterSupport::Unsupported),
            ("manage_skills".to_owned(), AdapterSupport::Unsupported),
            ("manage_mcp".to_owned(), AdapterSupport::Unsupported),
            ("manage_plugins".to_owned(), AdapterSupport::Unsupported),
            ("configure_provider".to_owned(), AdapterSupport::Unsupported),
            ("plan_mirror".to_owned(), AdapterSupport::MigrationOnly),
            ("plan_wrapper".to_owned(), AdapterSupport::Unsupported),
            ("scan_candidates".to_owned(), AdapterSupport::MigrationOnly),
            (
                "validate_instance".to_owned(),
                AdapterSupport::MigrationOnly,
            ),
            ("backup".to_owned(), AdapterSupport::MigrationOnly),
            ("export".to_owned(), AdapterSupport::MigrationOnly),
        ]
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            ".vibe-kanban-workspaces/*".to_owned(),
            "cache/*".to_owned(),
            "logs/*".to_owned(),
            "*.log".to_owned(),
            "*.tmp".to_owned(),
            ".tmp/*".to_owned(),
            "tmp/*".to_owned(),
            "*.lock".to_owned(),
            "sessions/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        Err(CoreError::UnsupportedOperation {
            harness: self.id.to_string(),
            operation: "plan_wrapper".to_owned(),
            reason: format!("MigrationOnly: {MIGRATION_TIP}: no new instances; export/backup only"),
        })
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            ".vibe-kanban-workspaces".to_owned(),
            ".vibe-kanban-workspaces/vk-* (worktree branch)".to_owned(),
            ".vibe-kanban (project)".to_owned(),
            "~/.vibe-kanban-workspaces (global workspaces)".to_owned(),
            "$VK_ALLOWED_ORIGINS / $VIBEKANBAN_REMOTE_JWT_SECRET (remote)".to_owned(),
            "npx vibe-kanban entrypoint".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::ProjectScope | Isolation::Unknown | Isolation::RelocatedRoot => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!(
                    "vibe-kanban (MigrationOnly) expects isolation project_scope (git worktree), got {other}: {MIGRATION_TIP}"
                ),
            }),
        }
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        Vec::new()
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "per-agent mcpServers are written into each agent own global config by VK itself (harness-managed); VK exposes an MCP server but has no VK-owned MCP dest (orchestrators.md)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("no plugin mechanism documented (orchestrators.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{HARNESS_ID_STR, VERSION_PROBE_BUDGET, VibeKanbanAdapter};

    use crate::adapter::{Adapter, ConfigScope, DocumentKind, ProductStatus};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> VibeKanbanAdapter {
        VibeKanbanAdapter::new().unwrap()
    }

    /// The npx entrypoint (observed 0.55-0.99s) exceeds the uniform 2s budget;
    /// the 5s pin is deliberate, so a regression to 2s fails here.
    #[test]
    fn version_probe_budget_covers_npx_cold_start() {
        assert_eq!(VERSION_PROBE_BUDGET, std::time::Duration::from_secs(5));
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-vibe-kanban-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::ProjectScope,
            origin: InstanceOrigin::Created,
            ownership: Ownership::SuperaiCreated,
            template: None,
            created_at: "2026-08-26T00:00:00Z".to_owned(),
            adapter_revision: crate::adapter::ADAPTER_REVISION.to_owned(),
        }
    }

    #[test]
    fn adapter_identity() {
        let a = adapter();
        // Constructor wiring plus catalog registration: an id the catalog
        // does not know can never reconcile with detection or instances.
        assert_eq!(a.id().as_str(), HARNESS_ID_STR);
        assert!(crate::harness_catalog::find_by_id(HARNESS_ID_STR).is_some());
        assert_eq!(a.product_status(), ProductStatus::Sunset);
    }

    #[test]
    fn supported_platforms_covers_all() {
        let a = adapter();
        let platforms = a.supported_platforms();
        assert!(platforms.len() >= 3);
        let os_set: HashSet<String> = platforms.iter().map(|p| p.os.to_string()).collect();
        assert!(os_set.contains("linux"));
        assert!(os_set.contains("macos"));
        assert!(os_set.contains("windows"));
    }

    #[test]
    fn detection_returns_evidence_with_migration_tip() {
        let a = adapter();
        let result = a.detection();
        assert!(!result.evidence.is_empty());
        assert!(result.evidence.iter().any(|e| e.contains("sunset")));
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("MigrationOnly") || e.contains("community"))
        );
        match result.present {
            InstallPresence::Absent => assert!(result.version.is_none()),
            InstallPresence::Present => assert!(result.version.is_some()),
            InstallPresence::UnknownVersion => {
                assert!(result.evidence.iter().any(|e| e.contains("found binary")));
            }
            InstallPresence::Broken => assert!(!result.evidence.is_empty()),
        }
    }

    #[test]
    fn version_resolution_maps_detected() {
        let a = adapter();
        let res = a.version_resolution();
        if res.detected_version.is_some() {
            assert_eq!(
                res.schema_version.as_deref(),
                Some(super::SCHEMA_VERSION_STR)
            );
            assert!(res.compatible);
            assert!(res.notes.iter().any(|n| n.contains("vibe-kanban")));
        } else {
            assert!(!res.compatible);
            assert!(res.schema_version.is_none());
            assert!(
                res.notes
                    .iter()
                    .any(|n| n.contains("MigrationOnly") || n.contains("migration tip"))
            );
        }
        assert!(!res.notes.is_empty());
    }

    #[test]
    fn config_surfaces_include_worktrees_and_profiles() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(surfaces.len() >= 4);
        let worktrees = surfaces
            .iter()
            .find(|s| s.id == "worktrees (.vibe-kanban-workspaces)")
            .expect("worktrees must exist");
        assert_eq!(worktrees.kind, DocumentKind::Opaque);
        assert_eq!(worktrees.scope, ConfigScope::ProjectWorkspace);
        let profiles = surfaces
            .iter()
            .find(|s| s.id == "agent profiles")
            .expect("agent profiles must exist");
        assert_eq!(profiles.kind, DocumentKind::Json);
        let mcp = surfaces
            .iter()
            .find(|s| s.id == "mcpServers (per-agent, harness-global)")
            .expect("mcp must exist");
        assert!(mcp.owned_selectors.contains(&"mcpServers".to_owned()));
    }

    #[test]
    fn supported_operations_are_migration_only() {
        let a = adapter();
        let ops = a.supported_operations();
        assert!(!ops.is_empty());
        let map: std::collections::HashMap<String, AdapterSupport> = ops.into_iter().collect();
        assert_eq!(map.get("detect"), Some(&AdapterSupport::MigrationOnly));
        assert_eq!(map.get("plan_wrapper"), Some(&AdapterSupport::Unsupported));
        assert_eq!(map.get("write_config"), Some(&AdapterSupport::Unsupported));
        assert_eq!(map.get("backup"), Some(&AdapterSupport::MigrationOnly));
    }

    #[test]
    fn plan_mirror_exclusions_cover_workspaces_and_logs() {
        let a = adapter();
        let exclusions = a.plan_mirror_exclusions();
        assert!(!exclusions.is_empty());
        for pat in [".vibe-kanban-workspaces/*", "cache/*", "*.lock"] {
            assert!(
                exclusions.contains(&pat.to_owned()),
                "exclusions must contain {pat}"
            );
        }
        assert!(!exclusions.contains(&"agent profiles".to_owned()));
    }

    #[test]
    fn plan_wrapper_is_blocked_migration_only() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".vibe-kanban-work"));
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::UnsupportedOperation {
                harness,
                operation,
                reason,
            } => {
                assert_eq!(harness, HARNESS_ID_STR);
                assert_eq!(operation, "plan_wrapper");
                assert!(reason.contains("MigrationOnly"));
                assert!(reason.contains("community"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness_but_blocked_first() {
        let a = adapter();
        let mut inst =
            sample_instance_with_root(&crate::test_util::tmp_abs_str(".vibe-kanban-work"));
        inst.harness = HarnessId::new("claude-code").unwrap();
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_workspaces_and_npx() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(!candidates.is_empty());
        assert!(
            candidates
                .iter()
                .any(|c| c.contains(".vibe-kanban-workspaces"))
        );
        assert!(candidates.iter().any(|c| c.contains("npx")));
        assert!(
            candidates
                .iter()
                .any(|c| c.contains("VK_ALLOWED_ORIGINS") || c.contains("VIBEKANBAN"))
        );
    }

    #[test]
    fn validate_instance_accepts_project_scope() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".vibe-kanban-work"));
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst =
            sample_instance_with_root(&crate::test_util::tmp_abs_str(".vibe-kanban-work"));
        inst.isolation = Isolation::OsBound;
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::Validation { field, .. } => assert_eq!(field, "isolation"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn supported_skill_modes_is_empty_migration_only() {
        let a = adapter();
        assert!(a.supported_skill_modes().is_empty());
    }

    #[test]
    fn fixture_corpus_validates_secret_free_and_flags_malformed() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/vibe_kanban");
        let report = crate::verification::fixture_report(&dir);
        assert!(!report.outcomes.is_empty(), "vibe_kanban corpus must load");
        assert!(
            report.validity_pass,
            "validity: {:?}",
            report
                .outcomes
                .iter()
                .filter(|o| o.exists && o.is_valid != o.expected_valid)
                .map(|o| o.path.display().to_string())
                .collect::<Vec<_>>()
        );
        assert!(
            report.secret_free_pass,
            "vibe_kanban fixtures must be secret-free"
        );
        assert!(report.malformed_count >= 2);
    }

    #[test]
    fn fixture_profiles_load_with_harness_entries() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/vibe_kanban");
        let profiles = superai_config::json::load(&dir.join("profiles.populated.json")).unwrap();
        assert!(profiles.contains_key("profiles"));
        assert!(
            superai_config::json::load(&dir.join("profiles.malformed.json")).is_err(),
            "malformed profiles must fail to parse"
        );
    }
}
