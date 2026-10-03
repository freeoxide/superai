//! `OpenClaw` adapter: long-running daemon (ports/gateway), config
//! `~/.openclaw/openclaw.json` + `.env`; research-blocked pending gateway.

use std::path::PathBuf;

use crate::adapter::{
    ADAPTER_REVISION, Adapter, ConfigScope, ConfigSurface, DetectionResult, DocumentKind,
    PathResolver, Platform, ProductStatus, RestartBehavior, RootShape, SurfaceOwnership,
    SurfaceSchema, VersionResolution, WrapperPlan,
};
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::instance::Instance;
use crate::state::AdapterSupport;
use superai_config::document::ValueType;

/// Catalog id superai registers this harness under.
pub const HARNESS_ID_STR: &str = "openclaw";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "OpenClaw";

/// Binary name resolved on PATH during detection.
pub const EXECUTABLE: &str = "openclaw";

/// Env var that overrides home.
pub const HOME_ENV_VAR: &str = "OPENCLAW_HOME";

/// Env var that overrides state dir.
pub const STATE_DIR_ENV_VAR: &str = "OPENCLAW_STATE_DIR";

/// Env var that overrides config path.
pub const CONFIG_PATH_ENV_VAR: &str = "OPENCLAW_CONFIG_PATH";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/openclaw.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-08-25";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Why provider writes stay blocked until the research lands.
pub const BLOCKED_REASON: &str = "daemon state, gateway/schema incomplete: ports, gateway security, multi-agent, plugin/skill paths unverified; long-running service not per-invocation CLI";

/// Daemon facts the corpus honestly permits: no verified port/bind, so
/// superai invents no port range and drives no start/stop for openclaw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConstraints {
    /// Whether the corpus verifies a port range/bind for the gateway.
    pub ports_verified: bool,
    /// Why port/bind facts remain unverified.
    pub ports_note: &'static str,
    /// Whether superai lifecycle may start/stop this daemon today.
    pub lifecycle_control: AdapterSupport,
}

/// The `OpenClaw` daemon constraints as the research state allows (declaration
/// only; see `docs/harness-configs/openclaw.md`).
#[must_use]
pub fn daemon_constraints() -> DaemonConstraints {
    DaemonConstraints {
        ports_verified: false,
        ports_note: "gateway ports/bind addresses are not verified in the corpus (openclaw.md)",
        lifecycle_control: AdapterSupport::ResearchBlocked,
    }
}

/// Concrete adapter for `OpenClaw` (`ResearchBlocked`).
#[derive(Debug, Clone)]
pub struct OpenClawAdapter {
    id: HarnessId,
}

impl OpenClawAdapter {
    /// Create a new adapter.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_state_dir() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(STATE_DIR_ENV_VAR)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir));
        }
        if let Ok(dir) = std::env::var(HOME_ENV_VAR)
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir));
        }
        let home = super::home_dir()?;
        Some(home.join(".openclaw"))
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "detection branches are explicit for evidence"
    )]
    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("research blocked: {BLOCKED_REASON}"));
        evidence.push(format!(
            "relocation via {HOME_ENV_VAR}/{STATE_DIR_ENV_VAR}/{CONFIG_PATH_ENV_VAR} (precedence explicit > HOME)"
        ));
        match Self::default_state_dir() {
            Some(dir) => {
                if dir.exists() {
                    evidence.push(format!("state dir exists at {}", dir.display()));
                    let config = dir.join("openclaw.json");
                    if config.exists() {
                        evidence.push(format!("openclaw.json found at {}", config.display()));
                    } else {
                        evidence.push(format!("openclaw.json missing at {}", config.display()));
                    }
                    let env = dir.join(".env");
                    if env.exists() {
                        evidence.push(format!(".env found at {}", env.display()));
                    }
                } else {
                    evidence.push(format!("state dir missing at {}", dir.display()));
                }
            }
            None => {
                evidence.push("could not resolve state dir (no HOME)".to_owned());
            }
        }
        if let Ok(cfg_path) = std::env::var(CONFIG_PATH_ENV_VAR)
            && !cfg_path.trim().is_empty()
        {
            evidence.push(format!("{CONFIG_PATH_ENV_VAR} override set to {cfg_path}"));
        }
    }
}

impl Default for OpenClawAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for OpenClawAdapter {
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
                    "found binary `{}` at {}",
                    EXECUTABLE,
                    path.display()
                ));
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
            None => {
                evidence.push(format!("binary `{EXECUTABLE}` not found in PATH"));
            }
        }

        self.collect_config_evidence(&mut evidence);

        let present = super::install_presence(binary_path.is_some(), version.is_some());

        let confidence = super::detection_confidence(
            binary_path.is_some(),
            version.is_some(),
            evidence.iter().any(|e| e.contains("state dir exists")),
        );

        DetectionResult::new(present, version, evidence, confidence)
    }

    fn version_resolution(&self) -> VersionResolution {
        self.version_resolution_from(&self.detection())
    }

    fn version_resolution_from(&self, detection: &DetectionResult) -> VersionResolution {
        if let Some(v) = detection.version.clone() {
            let mut notes = Vec::new();
            notes.push(format!("detected openclaw version {v}"));
            notes.push(format!("research blocked: {BLOCKED_REASON}"));
            let mut res = VersionResolution::new(Some(v), None, false);
            res.notes = notes;
            res
        } else {
            let mut res = VersionResolution::unknown();
            res.notes.clone_from(&detection.evidence);
            res.notes
                .push(format!("research blocked: {BLOCKED_REASON}"));
            res
        }
    }

    fn config_surfaces(&self) -> Vec<ConfigSurface> {
        let mut surfaces = Vec::new();

        let config_resolver = PathResolver::new(
            Some(
                "~/.openclaw/openclaw.json ($OPENCLAW_STATE_DIR/openclaw.json or $OPENCLAW_CONFIG_PATH)",
            ),
            Some("~/.openclaw/openclaw.json"),
            Some("%USERPROFILE%\\.openclaw\\openclaw.json"),
            "~/.openclaw/openclaw.json",
        );
        let mut config_surface = ConfigSurface::new(
            "openclaw.json",
            config_resolver,
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        config_surface.precedence = 10;
        config_surface.owned_selectors = vec![
            "agents.defaults.model".to_owned(),
            "models.providers".to_owned(),
        ];
        config_surface.backup_required = true;
        config_surface.restart_behavior = RestartBehavior::Restart;
        surfaces.push(config_surface);

        let env_resolver = PathResolver::new(
            Some("~/.openclaw/.env ($OPENCLAW_STATE_DIR/.env)"),
            Some("~/.openclaw/.env"),
            Some("%USERPROFILE%\\.openclaw\\.env"),
            "~/.openclaw/.env",
        );
        let mut env_surface = ConfigSurface::new(
            ".env",
            env_resolver,
            DocumentKind::Env,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        );
        env_surface.precedence = 8;
        env_surface.backup_required = true;
        surfaces.push(env_surface);

        let state_resolver = PathResolver::new(
            Some("~/.openclaw (daemon state, not writable)"),
            Some("~/.openclaw (daemon state)"),
            Some("%USERPROFILE%\\.openclaw (daemon state)"),
            "~/.openclaw (daemon state, gateway)",
        );
        let mut state_surface = ConfigSurface::new(
            "daemon-state",
            state_resolver,
            DocumentKind::Opaque,
            ConfigScope::Internal,
            SurfaceOwnership::HarnessManaged,
        );
        state_surface.precedence = 0;
        state_surface.backup_required = false;
        state_surface.restart_behavior = RestartBehavior::Restart;
        surfaces.push(state_surface);

        surfaces
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        vec![
            ("detect".to_owned(), AdapterSupport::ResearchBlocked),
            ("read_config".to_owned(), AdapterSupport::ResearchBlocked),
            ("write_config".to_owned(), AdapterSupport::ResearchBlocked),
            ("manage_skills".to_owned(), AdapterSupport::ResearchBlocked),
            ("manage_mcp".to_owned(), AdapterSupport::ResearchBlocked),
            ("manage_plugins".to_owned(), AdapterSupport::ResearchBlocked),
            (
                "configure_provider".to_owned(),
                AdapterSupport::ResearchBlocked,
            ),
            ("plan_mirror".to_owned(), AdapterSupport::ResearchBlocked),
            ("plan_wrapper".to_owned(), AdapterSupport::ResearchBlocked),
            (
                "scan_candidates".to_owned(),
                AdapterSupport::ResearchBlocked,
            ),
            (
                "validate_instance".to_owned(),
                AdapterSupport::ResearchBlocked,
            ),
        ]
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        vec![
            "sessions/*".to_owned(),
            "cache/*".to_owned(),
            "*.log".to_owned(),
            "daemon/*".to_owned(),
            "gateway/*".to_owned(),
        ]
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        Err(CoreError::ResearchBlocked {
            harness: self.id.to_string(),
            surface: "wrapper".to_owned(),
            reason: format!(
                "ResearchBlocked: {BLOCKED_REASON}: two instances means two daemons, ports/gateway not verified"
            ),
        })
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.openclaw/openclaw.json".to_owned(),
            "~/.openclaw/.env".to_owned(),
            "~/.openclaw (state dir)".to_owned(),
            "$OPENCLAW_STATE_DIR/openclaw.json via OPENCLAW_STATE_DIR".to_owned(),
            "$OPENCLAW_HOME/openclaw.json via OPENCLAW_HOME".to_owned(),
            "$OPENCLAW_CONFIG_PATH via OPENCLAW_CONFIG_PATH".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        Err(CoreError::ResearchBlocked {
            harness: self.id.to_string(),
            surface: "validate_instance".to_owned(),
            reason: format!(
                "ResearchBlocked: {BLOCKED_REASON}: validate blocked until gateway/schema complete"
            ),
        })
    }

    fn supported_skill_modes(&self) -> Vec<crate::adapter::SkillMode> {
        Vec::new()
    }

    fn surface_schema(&self, surface_id: &str) -> Option<SurfaceSchema> {
        // Read side only; writes stay ResearchBlocked. Shapes follow
        // openclaw.md §3: agents.defaults.model, models.providers.
        match surface_id {
            "openclaw.json" => Some(
                SurfaceSchema::new()
                    .with_root_shape(RootShape::Object)
                    .with_owned_key("agents", ValueType::Object)
                    .with_owned_key("models", ValueType::Object),
            ),
            ".env" => Some(SurfaceSchema::new().with_root_shape(RootShape::EnvEntries)),
            _ => None,
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "research-blocked; the openclaw.json schema is not walked end to end in the corpus (openclaw.md)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some("skills/plugin layout not walked end to end in the corpus (openclaw.md)")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{BLOCKED_REASON, HARNESS_ID_STR, OpenClawAdapter};
    use crate::adapter::{Adapter, ConfigScope, DocumentKind, SurfaceOwnership};
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> OpenClawAdapter {
        OpenClawAdapter::new().unwrap()
    }

    fn sample_instance_with_root(root: &str) -> Instance {
        Instance {
            id: InstanceId::new("test-openclaw-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(root).unwrap(),
            binary: None,
            wrapper: None,
            isolation: Isolation::DaemonService,
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
        let entry = crate::harness_catalog::find_by_id(HARNESS_ID_STR).unwrap();
        assert_eq!(a.id().as_str(), HARNESS_ID_STR);
        assert_eq!(a.product_status(), entry.product_status);
    }

    #[test]
    fn daemon_constraints_report_unverified_ports_and_blocked_lifecycle() {
        // The daemon machinery exists, but research verifies no ports and
        // permits no lifecycle control.
        let c = super::daemon_constraints();
        assert!(!c.ports_verified);
        assert!(
            c.ports_note.contains("not verified"),
            "note: {}",
            c.ports_note
        );
        assert_eq!(c.lifecycle_control, AdapterSupport::ResearchBlocked);
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
    fn detection_returns_evidence_with_blocked_reason() {
        let a = adapter();
        let result = a.detection();
        assert!(!result.evidence.is_empty());
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("research blocked"))
        );
        assert!(result.evidence.iter().any(|e| e.contains("OPENCLAW")));
    }

    #[test]
    fn version_resolution_is_not_compatible() {
        let a = adapter();
        let res = a.version_resolution();
        assert!(!res.compatible);
        assert!(res.schema_version.is_none());
        assert!(!res.notes.is_empty());
        assert!(
            res.notes
                .iter()
                .any(|n| n.contains("research blocked") || n.contains("gateway"))
        );
    }

    #[test]
    fn config_surfaces_exist() {
        let a = adapter();
        let surfaces = a.config_surfaces();
        assert!(!surfaces.is_empty());
        let cfg = surfaces
            .iter()
            .find(|s| s.id == "openclaw.json")
            .expect("openclaw.json must exist");
        assert_eq!(cfg.kind, DocumentKind::Json);
        assert_eq!(cfg.ownership, SurfaceOwnership::UserEditable);
        assert_eq!(cfg.scope, ConfigScope::User);
        assert!(cfg.backup_required);
    }

    #[test]
    fn supported_operations_are_research_blocked() {
        let a = adapter();
        let ops = a.supported_operations();
        for (_, support) in ops {
            assert_eq!(support, AdapterSupport::ResearchBlocked);
        }
        assert!(!BLOCKED_REASON.is_empty());
    }

    #[test]
    fn plan_wrapper_is_research_blocked() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".openclaw-work"));
        let err = a.plan_wrapper(&inst).unwrap_err();
        match err {
            CoreError::ResearchBlocked { reason, .. } => {
                assert!(
                    reason.contains("daemon")
                        || reason.contains("gateway")
                        || reason.contains("ResearchBlocked")
                );
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn validate_instance_is_research_blocked() {
        let a = adapter();
        let inst = sample_instance_with_root(&crate::test_util::tmp_abs_str(".openclaw-work"));
        let err = a.validate_instance(&inst).unwrap_err();
        match err {
            CoreError::ResearchBlocked { .. } => {}
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn scan_candidates_include_openclaw_paths() {
        let a = adapter();
        let candidates = a.scan_candidates();
        assert!(candidates.iter().any(|c| c.contains("openclaw.json")));
        assert!(candidates.iter().any(|c| c.contains("OPENCLAW")));
    }

    #[test]
    fn supported_skill_modes_is_empty() {
        let a = adapter();
        assert!(a.supported_skill_modes().is_empty());
    }

    #[test]
    fn surface_schema_declares_config_and_env_shapes() {
        let a = adapter();
        let config = a
            .surface_schema("openclaw.json")
            .expect("openclaw.json schema");
        assert_eq!(config.root_shape, Some(crate::adapter::RootShape::Object));
        assert!(config.owned_key_rules.iter().any(|r| r.path == "models"));
        assert!(config.owned_key_rules.iter().any(|r| r.path == "agents"));
        let env = a.surface_schema(".env").expect("env schema");
        assert_eq!(env.root_shape, Some(crate::adapter::RootShape::EnvEntries));
        assert!(a.surface_schema("daemon-state").is_none());
    }

    #[test]
    fn schema_rejects_non_object_openclaw_root() {
        let diags = crate::adapter::validate_surface_content(
            &adapter(),
            "openclaw.json",
            b"[1]",
            superai_config::document::DocumentKind::StrictJson,
        );
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("root must be of kind object")
                    && d.message.starts_with("[openclaw/openclaw.json]")),
            "diags: {diags:?}"
        );
        // Provider tree per docs §3: agents/models are objects.
        let bad = crate::adapter::validate_surface_content(
            &adapter(),
            "openclaw.json",
            br#"{"models": ["acme"]}"#,
            superai_config::document::DocumentKind::StrictJson,
        );
        assert!(
            bad.iter().any(|d| d
                .message
                .contains("`models` must hold a value of type object")),
            "diags: {bad:?}"
        );
        let ok = crate::adapter::validate_surface_content(
            &adapter(),
            "openclaw.json",
            br#"{"agents": {"defaults": {"model": {"primary": "p/m"}}}, "models": {"providers": {}}}"#,
            superai_config::document::DocumentKind::StrictJson,
        );
        assert!(ok.is_empty(), "{ok:?}");
    }
}
