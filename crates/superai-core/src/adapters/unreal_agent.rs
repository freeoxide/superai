//! Unreal Agent adapter: async-first Go harness with a thin user surface:
//! XDG `preferences.json` written by the setup dialog, `UNREAL_HARNESS_LLM_*`
//! env overrides; discovers and reuses an existing Codex login and settings
//! catalog. Research: docs/harness-configs/unreal-agent.md.

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
pub const HARNESS_ID_STR: &str = "unreal-agent";

/// Name shown for this harness in listings and errors.
pub const DISPLAY_NAME: &str = "Unreal Agent";

/// TUI binary name (brew installs `unreal-agent`, same TUI as uat).
pub const EXECUTABLE: &str = "unreal-agent";

/// Companion TUI binary name.
pub const EXECUTABLE_ALT: &str = "uat";

/// Research doc whose findings the constants in this file encode.
pub const RESEARCH_DOC: &str = "docs/harness-configs/unreal-agent.md";

/// Date the research doc was last checked against the harness.
pub const LAST_VERIFIED: &str = "2026-09-18";

/// Schema version this adapter maps detected versions to.
pub const SCHEMA_VERSION_STR: &str = "1";

/// Constrained note: everything below the TUI is undocumented.
pub const CONSTRAINED_NOTE: &str = "the harness library's configuration, the cloud runner's configuration, and operation-manager backends are undocumented outside the source; the TUI surface is one preferences file plus three env vars (unreal-agent.md sections 2-3, 5)";

/// Concrete adapter for Unreal Agent.
#[derive(Debug, Clone)]
pub struct UnrealAgentAdapter {
    id: HarnessId,
}

impl UnrealAgentAdapter {
    /// Create a new adapter instance, validating the static harness id.
    pub fn new() -> Result<Self, CoreError> {
        let id = HarnessId::new(HARNESS_ID_STR)?;
        Ok(Self { id })
    }

    fn default_config_root() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME")
            && !dir.trim().is_empty()
        {
            return Some(PathBuf::from(dir).join("unreal-agent"));
        }
        super::home_dir().map(|home| home.join(".config").join("unreal-agent"))
    }

    #[expect(clippy::unused_self, reason = "uses adapter constants via Self")]
    fn collect_config_evidence(&self, evidence: &mut Vec<String>) {
        evidence.push(format!("constrained: {CONSTRAINED_NOTE}"));
        match Self::default_config_root() {
            Some(root) => {
                let prefs = root.join("preferences.json");
                if prefs.exists() {
                    evidence.push(format!("preferences.json found at {}", prefs.display()));
                } else {
                    evidence.push(format!(
                        "preferences.json missing at {} (written by the first-launch setup dialog)",
                        prefs.display()
                    ));
                }
            }
            None => evidence.push("could not resolve config root (no HOME)".to_owned()),
        }
        if let Some(home) = super::home_dir()
            && home.join(".codex").exists()
        {
            evidence.push(
                "~/.codex present: the TUI can discover and reuse a Codex subscription login and settings.json model catalog (unreal-agent.md section 4)"
                    .to_owned(),
            );
        }
    }
}

impl Default for UnrealAgentAdapter {
    fn default() -> Self {
        let id = HarnessId::from_validated_const(HARNESS_ID_STR);
        Self { id }
    }
}

impl Adapter for UnrealAgentAdapter {
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

        match super::find_in_path(&[EXECUTABLE, EXECUTABLE_ALT]) {
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
                        "version probe failed; -help is documented but a --version flag is not (unreal-agent.md section 5)"
                            .to_owned(),
                    ),
                }
                binary_path = Some(path);
            }
            None => evidence.push(format!(
                "binaries `{EXECUTABLE}`/`{EXECUTABLE_ALT}` not found in PATH"
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
        vec![ConfigSurface::new(
            "preferences.json",
            PathResolver::fallback_only("$XDG_CONFIG_HOME/unreal-agent/preferences.json"),
            DocumentKind::Json,
            ConfigScope::User,
            SurfaceOwnership::UserEditable,
        )]
    }

    fn supported_operations(&self) -> Vec<(String, AdapterSupport)> {
        super::operations_at(AdapterSupport::Constrained)
    }

    fn plan_mirror_exclusions(&self) -> Vec<String> {
        Vec::new()
    }

    fn plan_wrapper(&self, instance: &Instance) -> Result<WrapperPlan, CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        let mut plan = WrapperPlan::new("XDG config swap; UNREAL_HARNESS_LLM_* per launch");
        plan.description = format!(
            " unreal-agent keeps one preferences file under the XDG config dir and takes \
             provider/model/effort from UNREAL_HARNESS_LLM_{{PROVIDER,MODEL,REASONING_EFFORT}} \
             per launch (flags > env > saved); point XDG_CONFIG_HOME at config_root {} for \
             per-instance preferences and keep the Codex discovery path out of the env to avoid \
             reusing a shared subscription login (unreal-agent.md sections 2-4)",
            instance.config_root
        );
        plan.env_vars.push((
            "XDG_CONFIG_HOME".to_owned(),
            instance.config_root.to_string(),
        ));
        plan.env_unset = [
            "UNREAL_HARNESS_LLM_PROVIDER",
            "UNREAL_HARNESS_LLM_MODEL",
            "UNREAL_HARNESS_LLM_REASONING_EFFORT",
        ]
        .iter()
        .map(|var| (*var).to_owned())
        .collect();
        plan.shared_state_warnings.push(
            "on first launch the TUI discovers an existing Codex subscription login and reads the \
             Codex settings.json model catalog; that tree is shared foreign state"
                .to_owned(),
        );
        Ok(plan)
    }

    fn scan_candidates(&self) -> Vec<String> {
        vec![
            "~/.config/unreal-agent".to_owned(),
            "$XDG_CONFIG_HOME/unreal-agent".to_owned(),
        ]
    }

    fn validate_instance(&self, instance: &Instance) -> Result<(), CoreError> {
        super::ensure_instance_harness(&self.id, instance)?;
        instance.validate()?;
        match instance.isolation {
            Isolation::FixedPathSingle
            | Isolation::RelocatedRoot
            | Isolation::EnvOnly
            | Isolation::Unknown => Ok(()),
            other => Err(CoreError::Validation {
                field: "isolation".to_owned(),
                reason: format!("unreal-agent requires isolation fixed_path_single, got {other}"),
            }),
        }
    }

    fn mcp_absence_reason(&self) -> Option<&'static str> {
        Some(
            "MCP is not mentioned in either README; nothing to document at v0.3.x (unreal-agent.md section 5)",
        )
    }

    fn plugin_absence_reason(&self) -> Option<&'static str> {
        Some(
            "no plugin mechanism is documented; the user-facing surface is one preferences file and three env vars (unreal-agent.md section 2)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_ID_STR, UnrealAgentAdapter};
    use crate::adapter::Adapter;
    use crate::error::CoreError;
    use crate::ids::{HarnessId, InstanceId, InstanceName};
    use crate::instance::Instance;
    use crate::paths::AbsolutePath;
    use crate::state::{AdapterSupport, InstanceOrigin, Isolation, Ownership};

    fn adapter() -> UnrealAgentAdapter {
        UnrealAgentAdapter::new().unwrap()
    }

    fn sample_instance() -> Instance {
        Instance {
            id: InstanceId::new("test-unreal-1").unwrap(),
            name: InstanceName::new("work").unwrap(),
            harness: HarnessId::new(HARNESS_ID_STR).unwrap(),
            config_root: AbsolutePath::new(&crate::test_util::tmp_abs_str(".unreal-agent-work"))
                .unwrap(),
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
    fn detection_names_the_codex_discovery_read_when_present() {
        let result = adapter().detection();
        assert!(
            result
                .evidence
                .iter()
                .any(|e| e.contains("preferences.json")),
            "the always-on evidence must name the TUI surface: {:?}",
            result.evidence
        );
        let codex_present =
            crate::adapters::home_dir().is_some_and(|home| home.join(".codex").exists());
        let codex_note = result
            .evidence
            .iter()
            .any(|e| e.contains("Codex subscription login"));
        assert_eq!(
            codex_note, codex_present,
            "the Codex discovery note must track the foreign ~/.codex tree"
        );
    }

    #[test]
    fn wrapper_unsets_the_env_model_overrides() {
        let plan = adapter().plan_wrapper(&sample_instance()).unwrap();
        assert_eq!(plan.env_vars.len(), 1, "{:?}", plan.env_vars);
        assert_eq!(plan.env_vars[0].0, "XDG_CONFIG_HOME");
        assert!(
            plan.env_unset
                .contains(&"UNREAL_HARNESS_LLM_PROVIDER".to_owned()),
            "env overrides outrank saved preferences and must not leak across instances"
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
