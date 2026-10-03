//! Claude Desktop adapter: platform-gated default root, no relocation.
//! Research source: `docs/harness-configs/claude-desktop.md` (verified 2026-09-18).

use std::path::{Path, PathBuf};
use std::time::Duration;

use superai_config::document::ValueType;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionConfidence, DetectionResult,
    DocumentKind, PathResolver, Platform, ProductStatus, RestartBehavior, RootShape,
    SurfaceOwnership, SurfaceSchema, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::{AdapterSupport, InstallPresence, Isolation};

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "claude-desktop";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Claude Desktop";

/// Linux .deb package/binary name; the GUI binary name is otherwise unverified.
pub const EXECUTABLE: &str = "claude-desktop";

/// Consumer config file name inside the per-OS app-support root.
pub const CONFIG_FILE: &str = "claude_desktop_config.json";

/// macOS config root (official, modelcontextprotocol.io quickstart).
pub const MACOS_CONFIG_ROOT: &str = "~/Library/Application Support/Claude";

/// Windows config root; MSIX installs may read the packaged Packages path instead (github #26073).
pub const WINDOWS_CONFIG_ROOT: &str = "%APPDATA%\\Claude";

/// Linux config root, community-corroborated; not yet in official consumer docs.
pub const LINUX_CONFIG_ROOT: &str = "~/.config/Claude";

/// Personal skills loaded by desktop/Cowork sessions; SHARED with the claude-code harness.
pub const PERSONAL_SKILLS_PATH: &str = "~/.claude/skills";

/// Verified-absent relocation: no env var, no portable mode, no `--config-dir`.
pub const NO_RELOCATION_NOTE: &str = "no config-relocation mechanism (verified-absent): \
no env var, no portable mode, no --config-dir; hardcoded app-support paths";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/claude-desktop.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Owned selectors inside `claude_desktop_config.json`.
pub const OWNED_SELECTORS: &[&str] = &["mcpServers"];

/// Concrete adapter for Claude Desktop.
#[derive(Debug, Clone)]
pub struct ClaudeDesktopAdapter {
    id: HarnessId,
}

impl ClaudeDesktopAdapter {
    /// Create a new adapter instance.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn find_binary_in_path() -> Option<PathBuf> {
        super::find_in_path(&[EXECUTABLE])
    }

    fn probe_version(binary: &Path) -> Option<String> {
        // Claude Desktop prints a date-stamped build string, not semver;
        // harvest_version_token stays local for that scheme.
        harvest_version_token(&super::run_capturing(
            binary,
            &["--version"],
            Duration::from_secs(2),
        )?)
    }

    fn default_config_root() -> Option<PathBuf> {
        let home = super::home_dir()?;
        if cfg!(target_os = "macos") {
            Some(
                PathBuf::from(&home)
                    .join("Library")
                    .join("Application Support")
                    .join("Claude"),
            )
        } else if cfg!(windows) {
            if let Ok(appdata) = std::env::var("APPDATA")
                && !appdata.trim().is_empty()
            {
                return Some(PathBuf::from(appdata).join("Claude"));
            }
            Some(
                PathBuf::from(&home)
                    .join("AppData")
                    .join("Roaming")
                    .join("Claude"),
            )
        } else {
            Some(PathBuf::from(&home).join(".config").join("Claude"))
        }
    }

    fn personal_skills_root() -> Option<PathBuf> {
        let home = super::home_dir()?;
        Some(home.join(".claude").join("skills"))
    }

    #[expect(clippy::excessive_nesting, reason = "evidence branches explicit")]
    fn collect_config_evidence(evidence: &mut Vec<String>) {
        match Self::default_config_root() {
            Some(root) => {
                let config = root.join(CONFIG_FILE);
                if config.exists() {
                    evidence.push(format!("config exists at {}", config.display()));
                    match std::fs::read_to_string(&config) {
                        Ok(text) if text.contains("\"mcpServers\"") => {
                            evidence.push("config contains mcpServers".to_owned());
                        }
                        Ok(_) => {}
                        Err(err) => evidence
                            .push(format!("config unreadable at {}: {err}", config.display())),
                    }
                } else if root.exists() {
                    evidence.push(format!(
                        "app-support root exists without {} at {}",
                        CONFIG_FILE,
                        root.display()
                    ));
                } else {
                    evidence.push(format!("config root missing at {}", root.display()));
                }
                let logs = root.join("logs");
                if logs.exists() {
                    evidence.push(format!("logs dir exists at {}", logs.display()));
                }
            }
            None => evidence.push("could not resolve home for config root".to_owned()),
        }
        if let Some(skills) = Self::personal_skills_root()
            && skills.exists()
        {
            evidence.push(format!(
                "personal skills shared with claude-code at {}",
                skills.display()
            ));
        }
        evidence.push(NO_RELOCATION_NOTE.to_owned());
    }
}

fn harvest_version_token(output: &str) -> Option<String> {
    for token in output.split_whitespace() {
        let candidate = token.strip_prefix('v').unwrap_or(token);
        let has_dot = candidate.contains('.');
        let starts_digit = candidate.chars().next().is_some_and(|c| c.is_ascii_digit());
        let version_like = candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
        if has_dot && starts_digit && version_like {
            return Some(candidate.to_owned());
        }
    }
    None
}

impl Default for ClaudeDesktopAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for ClaudeDesktopAdapter {
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
        match Self::find_binary_in_path() {
            Some(path) => {
                evidence.push(format!(
                    "found binary `{EXECUTABLE}` at {} (GUI binary name otherwise unverified)",
                    path.display()
                ));
                match Self::probe_version(&path) {
                    Some(v) => {
                        evidence.push(format!("version `{v}` via `--version`"));
                        version = Some(v);
                    }
                    None => evidence.push(
                        "version probe failed (no documented --version for the GUI binary)"
                            .to_owned(),
                    ),
                }
                binary_path = Some(path);
            }
            None => {
                evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            }
        }
        Self::collect_config_evidence(&mut evidence);
        // GUI apps often expose no binary on PATH; config-root evidence
        // alone counts as a low-confidence present.
        let config_seen = evidence.iter().any(|e| e.contains("config exists"));
        let present = match (&binary_path, &version) {
            (Some(_), Some(_)) => InstallPresence::Present,
            (Some(_), None) => InstallPresence::UnknownVersion,
            (None, _) if config_seen => InstallPresence::Present,
            (None, _) => InstallPresence::Absent,
        };
        let confidence = match (&binary_path, config_seen) {
            (Some(_), false) => DetectionConfidence::Medium,
            (None, true) => DetectionConfidence::Low,
            (Some(_), true) | (None, false) => DetectionConfidence::High,
        };
        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        if let Some(v) = detection.version.clone() {
            let notes = vec![
                format!("detected claude-desktop version {v} (date-stamped build scheme)"),
                format!("mapped to schema version {SCHEMA_VERSION_STR}"),
            ];
            let mut res =
                VersionResolution::new(Some(v), Some(SCHEMA_VERSION_STR.to_owned()), true);
            res.notes = notes;
            res
        } else if detection.present == InstallPresence::Present {
            let mut res = VersionResolution::new(None, Some(SCHEMA_VERSION_STR.to_owned()), true);
            res.notes.clone_from(&detection.evidence);
            res.notes.push(format!(
                "config-present detection; schema {SCHEMA_VERSION_STR} is the documented mcpServers shape"
            ));
            res
        } else {
            let mut res = VersionResolution::unknown();
            res.notes.clone_from(&detection.evidence);
            res
        }
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let config_resolver = PathResolver::new(
            Some(&format!("{LINUX_CONFIG_ROOT}/{CONFIG_FILE}")),
            Some(&format!("{MACOS_CONFIG_ROOT}/{CONFIG_FILE}")),
            Some(&format!("{WINDOWS_CONFIG_ROOT}\\{CONFIG_FILE}")),
            &format!("{MACOS_CONFIG_ROOT}/{CONFIG_FILE}"),
        );
        let mut config = ConfigSurface::new(
            CONFIG_FILE,
            config_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        config.precedence = 10;
        config.owned_selectors = OWNED_SELECTORS.iter().map(|s| (*s).to_owned()).collect();
        config.backup_required = true;
        config.restart_behavior = RestartBehavior::Restart;
        surfaces.push(config);

        let logs_resolver = PathResolver::new(
            Some("~/.config/Claude/logs (mcp.log, mcp-server-<name>.log)"),
            Some("~/Library/Logs/Claude (mcp.log, mcp-server-<name>.log)"),
            Some("%APPDATA%\\Claude\\logs"),
            "~/Library/Logs/Claude",
        );
        let mut logs = ConfigSurface::new(
            "logs",
            logs_resolver,
            DocumentKind::TextFragment,
            ConfigScope::Internal,
            SurfaceOwnership::HarnessManaged,
        );
        logs.precedence = 0;
        logs.backup_required = false;
        logs.restart_behavior = RestartBehavior::None;
        surfaces.push(logs);

        let skills_resolver = PathResolver::fallback_only(&format!(
            "{PERSONAL_SKILLS_PATH}/<name>/ (SHARED with claude-code)"
        ));
        let mut skills = ConfigSurface::new(
            "personal skills",
            skills_resolver,
            DocumentKind::TextFragment,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        skills.precedence = 5;
        skills.backup_required = false;
        skills.restart_behavior = RestartBehavior::Reload;
        surfaces.push(skills);

        let managed_resolver = PathResolver::new(
            Some("/etc/claude-desktop/managed-settings.json"),
            Some("/Library/Managed Preferences/<user>/com.anthropic.claudefordesktop.plist"),
            Some("HKLM\\SOFTWARE\\Policies\\Claude"),
            "3P/enterprise managed settings (out of alias scope)",
        );
        let mut managed = ConfigSurface::new(
            "managed-settings (3P/enterprise)",
            managed_resolver,
            DocumentKind::Json,
            ConfigScope::SystemManaged,
            SurfaceOwnership::HarnessManaged,
        );
        managed.precedence = 1;
        managed.backup_required = false;
        managed.restart_behavior = RestartBehavior::None;
        surfaces.push(managed);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        vec![
            ("detect".to_owned(), AdapterSupport::Constrained),
            ("read_config".to_owned(), AdapterSupport::Constrained),
            ("write_config".to_owned(), AdapterSupport::Constrained),
            ("manage_skills".to_owned(), AdapterSupport::Constrained),
            ("manage_mcp".to_owned(), AdapterSupport::Constrained),
            ("manage_plugins".to_owned(), AdapterSupport::Constrained),
            ("configure_provider".to_owned(), AdapterSupport::Constrained),
            ("plan_mirror".to_owned(), AdapterSupport::Constrained),
            ("plan_wrapper".to_owned(), AdapterSupport::Constrained),
            ("scan_candidates".to_owned(), AdapterSupport::Constrained),
            ("validate_instance".to_owned(), AdapterSupport::Constrained),
        ]
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "logs/*".to_owned(),
            "*.log".to_owned(),
            "cache/*".to_owned(),
            "tmp/*".to_owned(),
            "Cache".to_owned(),
            "Code Cache".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan =
            WrapperPlan::new("fixed default root: no relocation mechanism (verified-absent)");
        plan.description = format!(
            " claude-desktop reads hardcoded per-OS app-support paths \
             (macOS {MACOS_CONFIG_ROOT}, Windows {WINDOWS_CONFIG_ROOT}, Linux \
             {LINUX_CONFIG_ROOT}); {NO_RELOCATION_NOTE}; config_root {} is \
             informative only: no env vars are set and aliasing is refused \
             upstream (claude-desktop.md sections 4-5)",
            instance.config_root
        );
        plan.shared_state_warnings = vec![
            "extension (.mcpb) secrets live in the OS keychain, shared across every profile"
                .to_owned(),
            "personal skills at ~/.claude/skills are shared with the claude-code harness"
                .to_owned(),
            "on Windows the app's Claude Code sessions run inside a WSL2 distro (per-distro state)"
                .to_owned(),
        ];
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            format!("{MACOS_CONFIG_ROOT}/{CONFIG_FILE}"),
            format!("{WINDOWS_CONFIG_ROOT}\\{CONFIG_FILE}"),
            format!("{LINUX_CONFIG_ROOT}/{CONFIG_FILE}"),
            format!("{PERSONAL_SKILLS_PATH} (shared with claude-code)"),
            "/etc/claude-desktop/managed-settings.json (3P/enterprise)".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::FixedPathSingle | Isolation::RelocatedRoot | Isolation::Unknown => {
                crate::adapter::validate_instance_surfaces(self, instance.config_root.as_path())
            }
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!(
                    "claude-desktop has no relocation mechanism; isolation is \
                     fixed_path_single at the default root, got {other}"
                ),
            }),
        }
    }

    fn surface_schema(&self, surface_id: &str) -> Option<SurfaceSchema> {
        // Only the documented `mcpServers` shape is modeled; the full
        // consumer schema (beyond mcpServers) is unpublished.
        match surface_id {
            CONFIG_FILE => Some(
                SurfaceSchema::new()
                    .with_root_shape(RootShape::Object)
                    .with_owned_key("mcpServers", ValueType::Object),
            ),
            _ => None,
        }
    }

    // LinkAll first: relink_skills takes the first supported mode.
    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        vec![
            crate::adapter::SkillMode::LinkAll,
            crate::adapter::SkillMode::LinkSelected,
            crate::adapter::SkillMode::CopySelected,
        ]
    }

    fn mcp_decl(&self) -> Option<crate::adapter::McpAdapterDecl> {
        Some(crate::adapter::McpAdapterDecl::new(
            CONFIG_FILE,
            "mcpServers",
            DocumentKind::Json,
            ConfigScope::User,
            RestartBehavior::Restart,
        ))
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "desktop extensions (.mcpb) exist but the per-OS install directory is not \
             published (claude-desktop.md section 3); no honest staging target",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CONFIG_FILE, ClaudeDesktopAdapter, HARNESS_ID_STR, LINUX_CONFIG_ROOT, MACOS_CONFIG_ROOT,
        NO_RELOCATION_NOTE, OWNED_SELECTORS, WINDOWS_CONFIG_ROOT,
    };
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, RestartBehavior, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstallPresence, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> ClaudeDesktopAdapter {
        ClaudeDesktopAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-claude-desktop-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::FixedPathSingle,
            origin: InstanceOrigin::Created,
            ownership: Ownership::SuperaiCreated,
            template: None,
            created_at: "2026-09-18T00:00:00Z".to_owned(),
            adapter_revision: crate::adapter::ADAPTER_REVISION.to_owned(),
        }
    }

    #[test]
    fn adapter_identity() {
        let a = adapter();
        // Constructor wiring plus catalog registration: an id the catalog
        // does not know can never reconcile with detection or instances.
        let entry = crate::harness_catalog::find_by_id(HARNESS_ID_STR).unwrap();
        assert_eq!(a.id().as_str(), HARNESS_ID_STR);
        assert_eq!(a.product_status(), entry.product_status);
    }

    #[test]
    fn detection_returns_evidence_and_honesty_lines() {
        let a = adapter();
        let r = a.detection();
        assert!(!r.evidence.is_empty());
        // The verified-absent relocation note is part of the evidence so
        // callers never guess a CLAUDE_DESKTOP_CONFIG_DIR.
        assert!(
            r.evidence
                .iter()
                .any(|e| e.contains("no config-relocation")),
            "evidence must carry the no-relocation note: {:?}",
            r.evidence
        );
        match r.present {
            InstallPresence::Absent => assert!(r.version.is_none()),
            InstallPresence::Present | InstallPresence::UnknownVersion => {
                assert!(!r.evidence.is_empty());
            }
            InstallPresence::Broken => {}
        }
    }

    #[test]
    fn version_tokens_harvest_date_stamped_builds() {
        assert_eq!(
            super::harvest_version_token("Claude Desktop v1.2581.0").as_deref(),
            Some("1.2581.0")
        );
        assert_eq!(
            super::harvest_version_token("1.46388.1").as_deref(),
            Some("1.46388.1")
        );
        assert_eq!(super::harvest_version_token("not a version"), None);
        assert_eq!(super::harvest_version_token(""), None);
    }

    #[test]
    fn config_surfaces_are_platform_gated_without_env_relocation() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        let config = surfaces
            .iter()
            .find(|s| s.id == CONFIG_FILE)
            .expect("claude_desktop_config.json surface");
        assert_eq!(config.kind, DocumentKind::Json);
        assert_eq!(config.scope, ConfigScope::User);
        assert_eq!(config.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(config.restart_behavior, RestartBehavior::Restart);
        assert!(config.backup_required);
        for sel in OWNED_SELECTORS {
            assert!(config.owned_selectors.contains(&(*sel).to_owned()));
        }
        // Platform-gated resolvers name the three documented roots and never
        // invent an env var.
        let hints = [
            config.path_resolver.linux.as_deref(),
            config.path_resolver.macos.as_deref(),
            config.path_resolver.windows.as_deref(),
            Some(config.path_resolver.fallback.as_str()),
        ];
        for hint in hints.into_iter().flatten() {
            assert!(
                !hint.contains("CLAUDE_DESKTOP"),
                "no fabricated env: {hint}"
            );
        }
        assert_eq!(
            config.path_resolver.linux.as_deref(),
            Some(format!("{LINUX_CONFIG_ROOT}/{CONFIG_FILE}").as_str())
        );
        assert_eq!(
            config.path_resolver.macos.as_deref(),
            Some(format!("{MACOS_CONFIG_ROOT}/{CONFIG_FILE}").as_str())
        );
        assert_eq!(
            config.path_resolver.windows.as_deref(),
            Some(format!("{WINDOWS_CONFIG_ROOT}\\{CONFIG_FILE}").as_str())
        );
        assert!(surfaces.iter().any(|s| s.id == "personal skills"));
    }

    #[test]
    fn mcp_decl_is_writable_json_with_restart() {
        let a = adapter();
        let decl = a
            .mcp_decl()
            .expect("claude-desktop models a writable MCP dest");
        assert_eq!(decl.dest_file, CONFIG_FILE);
        assert_eq!(decl.dest_key, "mcpServers");
        assert_eq!(decl.kind, DocumentKind::Json);
        assert!(decl.read_only.is_none(), "consumer config is writable");
        assert_eq!(decl.restart, RestartBehavior::Restart);
        assert!(a.plugin_absence_reason().is_some_and(|r| !r.is_empty()));
        assert!(a.mcp_absence_reason().is_none());
    }

    #[test]
    fn plugin_absence_cites_unpublished_install_dir() {
        let a = adapter();
        let reason = a.plugin_absence_reason().unwrap();
        assert!(
            reason.contains("install directory is not published"),
            "absence must cite the .mcpb gap: {reason}"
        );
    }

    #[test]
    fn supported_operations_constrained() {
        let a = adapter();
        let ops = a.supported_operations();
        assert!(!ops.is_empty());
        for (_, support) in ops {
            assert_eq!(support, AdapterSupport::Constrained);
        }
    }

    /// Alias-contract pin: no relocation env vars exist to declare, so
    /// `alias::create_alias` refuses; the honest outcome, not a missing feature.
    #[test]
    fn plan_wrapper_declares_no_relocation_env_vars() {
        let a = adapter();
        let inst =
            sample_instance_with_root(&crate::test_util::tmp_abs_str(".claude-desktop-work"));
        let plan = a.plan_wrapper(&inst).unwrap();
        assert!(
            plan.env_vars.is_empty(),
            "no relocation env vars may be fabricated: {:?}",
            plan.env_vars
        );
        assert!(!plan.description.is_empty());
        assert!(
            plan.description
                .contains(NO_RELOCATION_NOTE.split(':').next().unwrap_or("")),
            "description must state the verified absence: {}",
            plan.description
        );
        assert!(
            !plan.shared_state_warnings.is_empty(),
            "keychain/skills/WSL shared state must be warned about"
        );
    }

    #[test]
    fn plan_wrapper_rejects_mismatched_harness() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".cd-work"));
        inst.harness = HarnessId::new("claude-code").unwrap();
        match a.plan_wrapper(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "harness"),
            other => panic!("expected harness validation, got {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_cover_all_three_platforms() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(
            candidates
                .iter()
                .any(|c| c.contains("Library/Application Support"))
        );
        assert!(candidates.iter().any(|c| c.contains("%APPDATA%")));
        assert!(candidates.iter().any(|c| c.contains(".config/Claude")));
    }

    #[test]
    fn validate_instance_accepts_fixed_path_and_relocated() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".cd-work"));
        a.validate_instance(&inst).unwrap();
        inst.isolation = Isolation::RelocatedRoot;
        a.validate_instance(&inst).unwrap();
    }

    #[test]
    fn validate_instance_rejects_wrong_isolation() {
        let a = adapter();
        let mut inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".cd-work"));
        inst.isolation = Isolation::EnvOnly;
        match a.validate_instance(&inst).unwrap_err() {
            CoreError::Validation { field, .. } => assert_eq!(field, "isolation"),
            other => panic!("expected isolation validation, got {other:?}"),
        }
    }

    #[test]
    fn adapter_is_object_safe() {
        let a = adapter();
        let boxed: Box<dyn Adapter> = Box::new(a);
        assert_eq!(boxed.id().as_str(), HARNESS_ID_STR);
        assert!(!boxed.config_surfaces().is_empty());
    }

    #[test]
    fn fixture_corpus_validity_and_secret_free() {
        let path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/claude_desktop");
        let report = crate::verification::fixture_report(&path);
        assert!(report.validity_pass, "claude_desktop corpus validity");
        assert!(report.secret_free_pass, "claude_desktop corpus secret-free");
        let populated = path.join("claude_desktop_config.populated.json");
        assert!(
            populated.exists(),
            "fixture missing: {}",
            populated.display()
        );
        let value = superai_config::json::load(&populated).unwrap();
        let servers = value.get("mcpServers").unwrap();
        assert!(servers.as_object().is_some_and(|m| !m.is_empty()));
    }

    /// Corpus satisfies the declared root shape + owned-key rule; a non-object root is rejected.
    #[test]
    fn surface_schema_accepts_corpus_and_rejects_bad_root() {
        let a = adapter();
        let corpus = b"{\"mcpServers\": {\"x\": {\"command\": \"c\"}}}";
        let diags = crate::adapter::validate_surface_content(
            &a,
            CONFIG_FILE,
            corpus,
            superai_config::document::DocumentKind::StrictJson,
        );
        assert!(diags.is_empty(), "corpus shape must validate: {diags:?}");
        let bad_root = b"[1, 2, 3]";
        let diags = crate::adapter::validate_surface_content(
            &a,
            CONFIG_FILE,
            bad_root,
            superai_config::document::DocumentKind::StrictJson,
        );
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("root must be") && d.message.contains("object")),
            "non-object root must be rejected: {diags:?}"
        );
    }
}
