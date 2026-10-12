//! Registered harness catalog: 91 product surfaces — the 51 plan rows plus
//! 40 research-verified additions (see each entry's source doc).

use serde::{Deserialize, Serialize};

use crate::adapter::{Adapter, GenericAdapter, ProductStatus};
use crate::adapters::agentty::AgenttyAdapter;
use crate::adapters::aider::AiderAdapter;
use crate::adapters::amazon_q::AmazonQAdapter;
use crate::adapters::amp::AmpAdapter;
use crate::adapters::antigravity::AntigravityAdapter;
use crate::adapters::auggie::AuggieAdapter;
use crate::adapters::avante_nvim::AvanteNvimAdapter;
use crate::adapters::bb_app::BbAppAdapter;
use crate::adapters::chatgpt_desktop::ChatGptDesktopAdapter;
use crate::adapters::claude_code::ClaudeCodeAdapter;
use crate::adapters::claude_desktop::ClaudeDesktopAdapter;
use crate::adapters::claw_code::ClawCodeAdapter;
use crate::adapters::cline::ClineAdapter;
use crate::adapters::coco::CocoAdapter;
use crate::adapters::codecompanion_nvim::CodecompanionNvimAdapter;
use crate::adapters::codegeex::CodegeexAdapter;
use crate::adapters::codex_cli::CodexCliAdapter;
use crate::adapters::command_code::CommandCodeAdapter;
use crate::adapters::conductor::ConductorAdapter;
use crate::adapters::continue_dev::ContinueDevAdapter;
use crate::adapters::copilot_cli::CopilotCliAdapter;
use crate::adapters::copilot_coding_agent::CopilotCodingAgentAdapter;
use crate::adapters::copilot_vscode::CopilotVscodeAdapter;
use crate::adapters::crush::CrushAdapter;
use crate::adapters::cursor::CursorAdapter;
use crate::adapters::cursor_agent::CursorAgentAdapter;
use crate::adapters::deepseek::DeepSeekAdapter;
use crate::adapters::devin::DevinAdapter;
use crate::adapters::eca::EcaAdapter;
use crate::adapters::every_code::EveryCodeAdapter;
use crate::adapters::factory_droid::FactoryDroidAdapter;
use crate::adapters::forge::ForgeAdapter;
use crate::adapters::freebuff::FreebuffAdapter;
use crate::adapters::fx::FxAdapter;
use crate::adapters::gemini_cli::GeminiCliAdapter;
use crate::adapters::gitclaw::GitclawAdapter;
use crate::adapters::goose::GooseAdapter;
use crate::adapters::gptme::GptmeAdapter;
use crate::adapters::grok_build::GrokBuildAdapter;
use crate::adapters::hax::HaxAdapter;
use crate::adapters::hermes::HermesAdapter;
use crate::adapters::iflow::IflowAdapter;
use crate::adapters::jcode::JcodeAdapter;
use crate::adapters::jules::JulesAdapter;
use crate::adapters::junie::JunieAdapter;
use crate::adapters::kilo::KiloAdapter;
use crate::adapters::kilocode_cli::KilocodeCliAdapter;
use crate::adapters::kimi_code::KimiCodeAdapter;
use crate::adapters::kiro::KiroAdapter;
use crate::adapters::kode::KodeAdapter;
use crate::adapters::legacy_kimi::LegacyKimiAdapter;
use crate::adapters::letta::LettaAdapter;
use crate::adapters::mimo::MimoAdapter;
use crate::adapters::minimax_code::MiniMaxCodeAdapter;
use crate::adapters::mistral_vibe::MistralVibeAdapter;
use crate::adapters::muse_code::MuseCodeAdapter;
use crate::adapters::nanocoder::NanocoderAdapter;
use crate::adapters::oh_my_codex::OhMyCodexAdapter;
use crate::adapters::oh_my_opencode::OhMyOpencodeAdapter;
use crate::adapters::omo_ai::OmoAiAdapter;
use crate::adapters::omp::OmpAdapter;
use crate::adapters::openclaw::OpenClawAdapter;
use crate::adapters::opencode::OpenCodeAdapter;
use crate::adapters::openfang::OpenfangAdapter;
use crate::adapters::openhands::OpenHandsAdapter;
use crate::adapters::orca::OrcaAdapter;
use crate::adapters::ori::OriAdapter;
use crate::adapters::pi::PiAdapter;
use crate::adapters::plandex::PlandexAdapter;
use crate::adapters::prime_agent::PrimeAgentAdapter;
use crate::adapters::qoder_cli::QoderCliAdapter;
use crate::adapters::qoder_ide::QoderIdeAdapter;
use crate::adapters::qwen_code::QwenCodeAdapter;
use crate::adapters::reasonix::ReasonixAdapter;
use crate::adapters::replit_agent::ReplitAgentAdapter;
use crate::adapters::roo_code::RooCodeAdapter;
use crate::adapters::sculptor::SculptorAdapter;
use crate::adapters::swe_agent::SweAgentAdapter;
use crate::adapters::t3code::T3codeAdapter;
use crate::adapters::terminus::TerminusAdapter;
use crate::adapters::trae_agent::TraeAgentAdapter;
use crate::adapters::unreal_agent::UnrealAgentAdapter;
use crate::adapters::vibe_kanban::VibeKanbanAdapter;
use crate::adapters::vix::VixAdapter;
use crate::adapters::warp::WarpAdapter;
use crate::adapters::windsurf::WindsurfAdapter;
use crate::adapters::workbuddy::WorkBuddyAdapter;
use crate::adapters::zaly::ZalyAdapter;
use crate::adapters::zcode::ZcodeAdapter;
use crate::adapters::zed_acp::ZedAcpAdapter;
use crate::adapters::zulu_cli::ZuluCliAdapter;
use crate::error::CoreError;
use crate::ids::HarnessId;
use crate::state::{AdapterSupport, Isolation};

/// One provisional ledger row, registered in code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Stable harness identifier.
    pub id: &'static str,
    /// Human display name.
    pub display_name: &'static str,
    /// Source research document.
    pub source: &'static str,
    /// Provisional support state from the master plan ledger.
    pub support: AdapterSupport,
    /// Reason or entry-gate note.
    pub reason: &'static str,
    /// Isolation class for the harness.
    pub isolation: Isolation,
    /// Product lifecycle status.
    pub product_status: ProductStatus,
    /// Link to the research document (same as source, kept for adapter).
    pub research_doc: &'static str,
    /// Last verified date, `YYYY-MM-DD`.
    pub last_verified: &'static str,
}

impl CatalogEntry {
    /// Validate that the id is a legal [`HarnessId`].
    pub fn validate_id(&self) -> Result<HarnessId, CoreError> {
        HarnessId::new(self.id)
    }
}

/// All 91 provisional ledger rows.
/// Order follows the plan table; every surface has a source and reason.
pub const ENTRIES: &[CatalogEntry] = &[
    CatalogEntry {
        id: "agentty",
        display_name: "agentty",
        source: "docs/harness-configs/agentty.md",
        support: AdapterSupport::Full,
        reason: "AGENTTY_HOME plus per-dir AGENTTY_*_DIR overrides, JSON settings/MCP/hooks: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/agentty.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "aider",
        display_name: "Aider",
        source: "docs/harness-configs/aider.md",
        support: AdapterSupport::Full,
        reason: "YAML/env/JSON, explicit config: full candidate",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/aider.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "amazon-q-cli",
        display_name: "Amazon Q Developer CLI",
        source: "docs/harness-configs/amazon-q-cli.md",
        support: AdapterSupport::MigrationOnly,
        reason: "sunsetting 2026-05-15, EOS 2027-04-30: migration only",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Sunset,
        research_doc: "docs/harness-configs/amazon-q-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "amp",
        display_name: "Amp",
        source: "docs/harness-configs/amp.md",
        support: AdapterSupport::Constrained,
        reason: "explicit settings file, account constrained",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/amp.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "antigravity-cli",
        display_name: "Antigravity CLI",
        source: "docs/harness-configs/antigravity-cli.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "settings/harness-owned auth incomplete, HOME workaround",
        isolation: Isolation::OsBound,
        product_status: ProductStatus::Preview,
        research_doc: "docs/harness-configs/antigravity-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "auggie",
        display_name: "Auggie",
        source: "docs/harness-configs/auggie.md",
        support: AdapterSupport::Constrained,
        reason: "account/workspace constrained",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/auggie.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "avante-nvim",
        display_name: "avante.nvim",
        source: "docs/harness-configs/avante-nvim.md",
        support: AdapterSupport::Constrained,
        reason: "Lua-only setup(), no config files; MCP via mcphub.nvim; Neovim config-root isolation",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/avante-nvim.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "bb-app",
        display_name: "bb (get-bb)",
        source: "docs/harness-configs/bb-app.md",
        support: AdapterSupport::Constrained,
        reason: "BB_DATA_DIR whole-root relocation, but MCP unverified and plugin state lives in SQLite",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/bb-app.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "chatgpt-desktop",
        display_name: "ChatGPT Desktop (Codex)",
        source: "docs/harness-configs/chatgpt-desktop.md",
        support: AdapterSupport::ReadOnly,
        reason: "read-only on ~/.codex, which belongs to codex-cli (shared store)",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/chatgpt-desktop.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "claude-code",
        display_name: "Claude Code",
        source: "docs/harness-configs/claude-code.md",
        support: AdapterSupport::Full,
        reason: "relocated-root verified, full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/claude-code.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "claude-desktop",
        display_name: "Claude Desktop",
        source: "docs/harness-configs/claude-desktop.md",
        support: AdapterSupport::Constrained,
        reason: "writable mcpServers at the fixed default root; relocation verified-absent",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/claude-desktop.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "claw-code",
        display_name: "Claw Code",
        source: "docs/harness-configs/claw-code.md",
        support: AdapterSupport::Constrained,
        reason: "five-layer JSON settings chain with CLAW_CONFIG_HOME, but museum-exhibit status, no release channel",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/claw-code.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "cline",
        display_name: "Cline",
        source: "docs/harness-configs/cline.md",
        support: AdapterSupport::Full,
        reason: "IDE user-data, full candidate after OS verification",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/cline.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "coco",
        display_name: "Cortex Code (CoCo)",
        source: "docs/harness-configs/coco.md",
        support: AdapterSupport::Constrained,
        reason: "SNOWFLAKE_HOME root relocation but paid-account-tied, no BYO endpoint",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/coco.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "codecompanion-nvim",
        display_name: "codecompanion.nvim",
        source: "docs/harness-configs/codecompanion-nvim.md",
        support: AdapterSupport::Constrained,
        reason: "Lua setup() table is the only surface; native stdio MCP under the mcp key",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/codecompanion-nvim.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "codegeex",
        display_name: "CodeGeeX",
        source: "docs/harness-configs/codegeex.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "host-IDE settings with no published key list; no MCP, env vars, or local files",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/codegeex.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "codex-cli",
        display_name: "Codex CLI",
        source: "docs/harness-configs/codex-cli.md",
        support: AdapterSupport::Full,
        reason: "TOML, relocated-root/profile: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/codex-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "command-code",
        display_name: "Command Code (cmd)",
        source: "docs/harness-configs/command-code.md",
        support: AdapterSupport::Constrained,
        reason: "fixed ~/.commandcode root with 3-scope MCP and BYOK; no relocation var",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/command-code.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "continue-dev",
        display_name: "Continue",
        source: "docs/harness-configs/continue-dev.md",
        support: AdapterSupport::Constrained,
        reason: "YAML/env, project/explicit, hosted features excluded",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Acquired,
        research_doc: "docs/harness-configs/continue-dev.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "copilot-cli",
        display_name: "GitHub Copilot CLI",
        source: "docs/harness-configs/copilot-cli.md",
        support: AdapterSupport::Full,
        reason: "JSONC/MCP, relocated-root: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/copilot-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "copilot-coding-agent",
        display_name: "Copilot Coding Agent",
        source: "docs/harness-configs/copilot-cli.md",
        support: AdapterSupport::Unsupported,
        reason: "cloud-owned repo/org settings, no local mutation",
        isolation: Isolation::Unsupported,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/copilot-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "copilot-vscode",
        display_name: "VS Code Copilot Agent Mode",
        source: "docs/harness-configs/copilot-vscode.md",
        support: AdapterSupport::Constrained,
        reason: "IDE settings and workspace MCP JSON writable; shares ~/.copilot read-only with copilot-cli",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/copilot-vscode.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "crush",
        display_name: "Crush",
        source: "docs/harness-configs/crush.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "executable crushrc, deprecated JSON: research blocked for writes",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/crush.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "cursor",
        display_name: "Cursor IDE and Agent CLI",
        source: "docs/harness-configs/cursor.md",
        support: AdapterSupport::Constrained,
        reason: "CLI root plus IDE user-data: constrained",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/cursor.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "cursor-agent",
        display_name: "Cursor Agent CLI",
        source: "docs/harness-configs/cursor-agent.md",
        support: AdapterSupport::Constrained,
        reason: "CURSOR_CONFIG_DIR relocates cli-config.json only; Cursor-backend-only auth",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/cursor-agent.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "deepseek-harness",
        display_name: "DeepSeek Harness",
        source: "docs/harness-configs/deepseek-harness.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "provider catalog incomplete, developer preview",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Preview,
        research_doc: "docs/harness-configs/deepseek-harness.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "devin",
        display_name: "Devin CLI",
        source: "docs/harness-configs/devin.md",
        support: AdapterSupport::Constrained,
        reason: "JSONC config plus dedicated MCP files, but no config-dir var and org settings outrank user config",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/devin.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "eca",
        display_name: "ECA (Editor Code Assistant)",
        source: "docs/harness-configs/eca.md",
        support: AdapterSupport::Constrained,
        reason: "layered JSON with --config-file and ECA_CONFIG inline, but key schema unpublished",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/eca.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "every-code",
        display_name: "Every Code",
        source: "docs/harness-configs/every-code.md",
        support: AdapterSupport::Constrained,
        reason: "CODE_HOME relocation but auth/session scope unverified; reads foreign ~/.codex",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/every-code.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "factory-droid",
        display_name: "Factory Droid",
        source: "docs/harness-configs/factory-droid.md",
        support: AdapterSupport::Constrained,
        reason: "project/HOME constrained, layered JSON",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/factory-droid.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "forge",
        display_name: "Forge",
        source: "docs/harness-configs/forge.md",
        support: AdapterSupport::Full,
        reason: "relocated config: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/forge.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "freebuff",
        display_name: "Freebuff",
        source: "docs/harness-configs/freebuff.md",
        support: AdapterSupport::Constrained,
        reason: "FREEBUFF_CONFIG_DIR root (absolute-only), but settings schema and MCP unpublished",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/freebuff.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "fx",
        display_name: "fx (Vercel Labs)",
        source: "docs/harness-configs/fx.md",
        support: AdapterSupport::Full,
        reason: "writable JSON at fixed ~/.fx root, any OpenAI-compatible endpoint",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Preview,
        research_doc: "docs/harness-configs/fx.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "gemini-cli",
        display_name: "Gemini CLI",
        source: "docs/harness-configs/gemini-cli.md",
        support: AdapterSupport::MigrationOnly,
        reason: "retired consumer tiers 2026-06-18, successor Antigravity",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Retired,
        research_doc: "docs/harness-configs/gemini-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "gitclaw",
        display_name: "Gitagent",
        source: "docs/harness-configs/gitclaw.md",
        support: AdapterSupport::Constrained,
        reason: "agent dir is a git repo isolated via --dir; agent.yaml schema only partly documented",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/gitclaw.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "goose",
        display_name: "Goose",
        source: "docs/harness-configs/goose.md",
        support: AdapterSupport::Full,
        reason: "YAML config/recipes, relocated-root: full candidate after unverified keys close",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/goose.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "gptme",
        display_name: "gptme",
        source: "docs/harness-configs/gptme.md",
        support: AdapterSupport::Constrained,
        reason: "workspace plus explicit state: constrained",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/gptme.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "grok-build",
        display_name: "Grok Build",
        source: "docs/harness-configs/grok-build.md",
        support: AdapterSupport::Full,
        reason: "TOML + JSON overlay, relocated-root: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/grok-build.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "hax",
        display_name: "hax",
        source: "docs/harness-configs/hax.md",
        support: AdapterSupport::Full,
        reason: "XDG-relocatable config/state/cache, env-only credentials, clean headless stdout",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/hax.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "hermes-agent",
        display_name: "Hermes Agent",
        source: "docs/harness-configs/hermes-agent.md",
        support: AdapterSupport::Full,
        reason: "YAML/env, relocated-root/profile: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/hermes-agent.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "iflow-cli",
        display_name: "iFlow CLI",
        source: "docs/harness-configs/iflow-cli.md",
        support: AdapterSupport::MigrationOnly,
        reason: "shutdown 2026-04-17: migration only",
        isolation: Isolation::EnvOnly,
        product_status: ProductStatus::Sunset,
        research_doc: "docs/harness-configs/iflow-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "jcode",
        display_name: "jcode",
        source: "docs/harness-configs/jcode.md",
        support: AdapterSupport::Constrained,
        reason: "fixed ~/.jcode TOML and MCP; cross-reads ~/.claude.json and imports Codex MCP; no relocation var",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/jcode.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "jules",
        display_name: "Jules CLI",
        source: "docs/harness-configs/jules.md",
        support: AdapterSupport::Constrained,
        reason: "thin cloud-session client; no local config or MCP surface, server-side work",
        isolation: Isolation::DaemonService,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/jules.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "junie-cli",
        display_name: "Junie CLI",
        source: "docs/harness-configs/junie-cli.md",
        support: AdapterSupport::Full,
        reason: "relocated-root, EAP gated: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Eap,
        research_doc: "docs/harness-configs/junie-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "kilo-code",
        display_name: "Kilo Code extension and CLI",
        source: "docs/harness-configs/kilo-code.md",
        support: AdapterSupport::Constrained,
        reason: "layered JSONC, inline/HOME plus IDE: constrained until root verified",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/kilo-code.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "kilocode-cli",
        display_name: "Kilo CLI",
        source: "docs/harness-configs/kilocode-cli.md",
        support: AdapterSupport::Full,
        reason: "OpenCode-fork JSONC, XDG/HOME root relocation: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/kilocode-cli.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "kimi-code-cli",
        display_name: "Kimi Code CLI",
        source: "docs/harness-configs/kimi-cli.md",
        support: AdapterSupport::Full,
        reason: "TOML plus MCP JSON, relocated-root: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/kimi-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "legacy-kimi-cli",
        display_name: "Legacy Kimi CLI",
        source: "docs/harness-configs/kimi-cli.md",
        support: AdapterSupport::MigrationOnly,
        reason: "legacy root, wound down: migration only",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Retired,
        research_doc: "docs/harness-configs/kimi-cli.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "kiro",
        display_name: "Kiro CLI/IDE",
        source: "docs/harness-configs/kiro.md",
        support: AdapterSupport::ReadOnly,
        reason: "read-only until research gaps close, BYO limitations",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/kiro.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "kode",
        display_name: "Kode CLI",
        source: "docs/harness-configs/kode.md",
        support: AdapterSupport::Full,
        reason: "JSON/MCP/agents, relocated-root: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/kode.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "letta-code",
        display_name: "Letta Code",
        source: "docs/harness-configs/letta-code.md",
        support: AdapterSupport::Constrained,
        reason: "client plus server/provider state, separate server",
        isolation: Isolation::DaemonService,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/letta-code.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "mimo-code",
        display_name: "MiMo Code",
        source: "docs/harness-configs/mimo-code.md",
        support: AdapterSupport::Full,
        reason: "JSON/JSONC, relocated-root: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/mimo-code.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "minimax-code",
        display_name: "MiniMax Code",
        source: "docs/harness-configs/minimax-code.md",
        support: AdapterSupport::Full,
        reason: "MINIMAX_DATA_DIR relocation, config.yaml BYOK: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/minimax-code.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "mistral-vibe",
        display_name: "Mistral Vibe",
        source: "docs/harness-configs/mistral-vibe.md",
        support: AdapterSupport::Full,
        reason: "TOML, relocated-root: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/mistral-vibe.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "muse-code",
        display_name: "Muse Code",
        source: "docs/harness-configs/muse-code.md",
        support: AdapterSupport::Constrained,
        reason: "writable settings.json but Meta-backend-only auth, no relocation var",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Preview,
        research_doc: "docs/harness-configs/muse-code.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "nanocoder",
        display_name: "Nanocoder",
        source: "docs/harness-configs/nanocoder.md",
        support: AdapterSupport::Full,
        reason: "JSON provider/MCP, relocated/explicit: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/nanocoder.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "oh-my-codex",
        display_name: "oh-my-codex (omx)",
        source: "docs/harness-configs/oh-my-codex.md",
        support: AdapterSupport::Constrained,
        reason: "workflow layer over codex; writes .codex/config.toml, state under .omx/ and CODEX_HOME cache",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/oh-my-codex.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "oh-my-opencode",
        display_name: "Oh My OpenCode (plugin)",
        source: "docs/harness-configs/oh-my-opencode.md",
        support: AdapterSupport::MigrationOnly,
        reason: "upstream renamed to oh-my-openagent; plugin lineage a dormant fork: migration only",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Retired,
        research_doc: "docs/harness-configs/oh-my-opencode.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "omo-ai",
        display_name: "OmO",
        source: "docs/harness-configs/omo-ai.md",
        support: AdapterSupport::Constrained,
        reason: "~/.omo/omo.jsonc layer walk with OMO_PROFILE; the [codex] block writes Codex settings",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/omo-ai.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "omp",
        display_name: "omp (oh-my-pi)",
        source: "docs/harness-configs/omp.md",
        support: AdapterSupport::Constrained,
        reason: "config/models.yml plus --config overlay; first-run inheritance reads .claude/.codex/.cursor",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/omp.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "openclaw",
        display_name: "OpenClaw",
        source: "docs/harness-configs/openclaw.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "daemon state, gateway/schema incomplete",
        isolation: Isolation::DaemonService,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/openclaw.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "opencode",
        display_name: "OpenCode",
        source: "docs/harness-configs/opencode.md",
        support: AdapterSupport::Full,
        reason: "layered JSONC, relocated/inline: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/opencode.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "openfang",
        display_name: "OpenFang",
        source: "docs/harness-configs/openfang.md",
        support: AdapterSupport::Constrained,
        reason: "OPENFANG_HOME-relocatable TOML, env-name secrets, stdio/sse MCP; pre-1.0 daemon",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/openfang.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "openhands",
        display_name: "OpenHands",
        source: "docs/harness-configs/openhands.md",
        support: AdapterSupport::Constrained,
        reason: "V1 JSON/env plus V0 TOML, Docker: version split required",
        isolation: Isolation::OsBound,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/openhands.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "orca",
        display_name: "Orca (Stably AI)",
        source: "docs/harness-configs/orca.md",
        support: AdapterSupport::Constrained,
        reason: "ADE over BYO agent fleets; per-agent command/env overrides, settings storage path undocumented",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/orca.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "ori",
        display_name: "Ori (OpenRouter)",
        source: "docs/harness-configs/ori.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "binary-only distribution with private source; only the install surface is documented",
        isolation: Isolation::Unknown,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/ori.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "pi",
        display_name: "Pi",
        source: "docs/harness-configs/pi.md",
        support: AdapterSupport::Full,
        reason: "JSON settings/auth, relocated-root, MCP absent by design",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/pi.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "plandex",
        display_name: "Plandex",
        source: "docs/harness-configs/plandex.md",
        support: AdapterSupport::Constrained,
        reason: "env + server/model-pack, provider/server scoped",
        isolation: Isolation::EnvOnly,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/plandex.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "prime-agent",
        display_name: "Prime Agent",
        source: "docs/harness-configs/prime-agent.md",
        support: AdapterSupport::Constrained,
        reason: "PRIME_AGENT_CODING_AGENT_DIR-relocatable settings; daemon sessions, MCP undocumented",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/prime-agent.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "qoder-cli",
        display_name: "Qoder CLI",
        source: "docs/harness-configs/qoder-cli.md",
        support: AdapterSupport::Full,
        reason: "QODER_CONFIG_DIR relocation, layered JSON settings: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/qoder-cli.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "qoder-ide",
        display_name: "Qoder IDE and Desktop",
        source: "docs/harness-configs/qoder-ide.md",
        support: AdapterSupport::Constrained,
        reason: "UI-driven settings with undocumented storage paths; only the MCP JSON editor is file-backed",
        isolation: Isolation::Unknown,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/qoder-ide.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "qwen-code",
        display_name: "Qwen Code",
        source: "docs/harness-configs/qwen-code.md",
        support: AdapterSupport::Full,
        reason: "layered JSON/env/MCP, relocated settings: full candidate",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/qwen-code.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "reasonix",
        display_name: "Reasonix",
        source: "docs/harness-configs/reasonix.md",
        support: AdapterSupport::Constrained,
        reason: "DeepSeek-only by design; fixed ~/.reasonix root with project overrides; reads .claude/skills",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/reasonix.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "replit-agent",
        display_name: "Replit Agent",
        source: "docs/harness-configs/replit-agent.md",
        support: AdapterSupport::Unsupported,
        reason: "cloud workspace agent; no executable, local files, or env surface",
        isolation: Isolation::Unsupported,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/replit-agent.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "roo-code",
        display_name: "Roo Code",
        source: "docs/harness-configs/roo-code.md",
        support: AdapterSupport::MigrationOnly,
        reason: "VS Code storage, archived 2026-05: migration only",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Archived,
        research_doc: "docs/harness-configs/roo-code.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "swe-agent",
        display_name: "SWE-agent",
        source: "docs/harness-configs/swe-agent.md",
        support: AdapterSupport::Full,
        reason: "composed YAML, explicit config/batch: full candidate",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/swe-agent.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "t3code",
        display_name: "T3 Code",
        source: "docs/harness-configs/t3code.md",
        support: AdapterSupport::Constrained,
        reason: "control surface over external CLIs; --base-dir server root, settings store path undocumented",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/t3code.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "terminus",
        display_name: "Terminus (Terminal-Bench)",
        source: "docs/harness-configs/terminus.md",
        support: AdapterSupport::ResearchBlocked,
        reason: "benchmark agent pre-integrated into Harbor; no standalone install or user config files",
        isolation: Isolation::OsBound,
        product_status: ProductStatus::Preview,
        research_doc: "docs/harness-configs/terminus.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "trae-agent",
        display_name: "Trae Agent",
        source: "docs/harness-configs/trae-agent.md",
        support: AdapterSupport::Full,
        reason: "YAML + deprecated JSON, explicit config/env: full candidate",
        isolation: Isolation::ExplicitConfig,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/trae-agent.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "unreal-agent",
        display_name: "Unreal Agent",
        source: "docs/harness-configs/unreal-agent.md",
        support: AdapterSupport::Constrained,
        reason: "thin surface: one XDG preferences.json and per-launch UNREAL_HARNESS_LLM_*; reads Codex login",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/unreal-agent.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "vix",
        display_name: "vix",
        source: "docs/harness-configs/vix.md",
        support: AdapterSupport::Constrained,
        reason: "vixd daemon plus isolated instances; providers.json merge, workflow schema deferred upstream",
        isolation: Isolation::DaemonService,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/vix.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "warp",
        display_name: "Warp Agent CLI/app",
        source: "docs/harness-configs/warp.md",
        support: AdapterSupport::Constrained,
        reason: "CLI TOML + MCP JSON, Linux XDG/profile constrained",
        isolation: Isolation::OsBound,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/warp.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "windsurf",
        display_name: "Windsurf/Devin Desktop",
        source: "docs/harness-configs/windsurf.md",
        support: AdapterSupport::Constrained,
        reason: "MCP JSON + rules/skills, IDE user-data: constrained",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/windsurf.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "workbuddy",
        display_name: "WorkBuddy / CodeBuddy CLI (cbc)",
        source: "docs/harness-configs/workbuddy.md",
        support: AdapterSupport::Constrained,
        reason: "CLI-only config surface under ~/.codebuddy; desktop app GUI-only, documented not mutated",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/workbuddy.md",
        last_verified: "2026-09-01",
    },
    CatalogEntry {
        id: "zaly",
        display_name: "zaly",
        source: "docs/harness-configs/zaly.md",
        support: AdapterSupport::Constrained,
        reason: "ZALY_ROOT whole-root override but alpha: settings schema unpublished, no MCP yet",
        isolation: Isolation::RelocatedRoot,
        product_status: ProductStatus::Preview,
        research_doc: "docs/harness-configs/zaly.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "zcode",
        display_name: "ZCode",
        source: "docs/harness-configs/zcode.md",
        support: AdapterSupport::SingleInstance,
        reason: "versioned-path JSON, fixed path: single instance",
        isolation: Isolation::FixedPathSingle,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/zcode.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "zed-acp",
        display_name: "Zed AI/ACP",
        source: "docs/harness-configs/zed-acp.md",
        support: AdapterSupport::Constrained,
        reason: "JSON settings, ACP wrappers, wrapper registrations: version gates",
        isolation: Isolation::IdeUserData,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/zed-acp.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "zulu-cli",
        display_name: "Zulu CLI (Baidu Comate)",
        source: "docs/harness-configs/zulu-cli.md",
        support: AdapterSupport::Constrained,
        reason: "license-key SaaS, no relocation var; .comate project files writable",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/zulu-cli.md",
        last_verified: "2026-09-18",
    },
    CatalogEntry {
        id: "vibe-kanban",
        display_name: "Vibe Kanban",
        source: "docs/harness-configs/orchestrators.md",
        support: AdapterSupport::MigrationOnly,
        reason: "orchestrator profiles/env/MCP/worktrees, community-maintained",
        isolation: Isolation::ProjectScope,
        product_status: ProductStatus::Sunset,
        research_doc: "docs/harness-configs/orchestrators.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "conductor",
        display_name: "Conductor",
        source: "docs/harness-configs/orchestrators.md",
        support: AdapterSupport::Constrained,
        reason: "user/repo TOML, macOS worktrees/profiles",
        isolation: Isolation::OsBound,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/orchestrators.md",
        last_verified: "2026-08-25",
    },
    CatalogEntry {
        id: "sculptor",
        display_name: "Sculptor",
        source: "docs/harness-configs/orchestrators.md",
        support: AdapterSupport::Constrained,
        reason: "env + harness settings, workspace/container",
        isolation: Isolation::OsBound,
        product_status: ProductStatus::Active,
        research_doc: "docs/harness-configs/orchestrators.md",
        last_verified: "2026-08-25",
    },
];

/// Return the full catalog slice.
pub fn all_entries() -> &'static [CatalogEntry] {
    ENTRIES
}

/// Find an entry by harness id string.
pub fn find_by_id(id: &str) -> Option<&'static CatalogEntry> {
    ENTRIES.iter().find(|entry| entry.id == id)
}

/// Build the concrete adapter for `id`, if one exists; otherwise callers
/// fall back to a [`GenericAdapter`] built from the catalog row.
#[expect(
    clippy::too_many_lines,
    reason = "one branch per concrete adapter; the generic fallback covers the rest"
)]
pub(crate) fn concrete_adapter_for(id: &str) -> Option<Box<dyn Adapter>> {
    if id == crate::adapters::claude_code::HARNESS_ID_STR
        && let Ok(adapter) = ClaudeCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::codex_cli::HARNESS_ID_STR
        && let Ok(adapter) = CodexCliAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::claude_desktop::HARNESS_ID_STR
        && let Ok(adapter) = ClaudeDesktopAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::chatgpt_desktop::HARNESS_ID_STR
        && let Ok(adapter) = ChatGptDesktopAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::aider::HARNESS_ID_STR
        && let Ok(adapter) = AiderAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::opencode::HARNESS_ID_STR
        && let Ok(adapter) = OpenCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::cline::HARNESS_ID_STR
        && let Ok(adapter) = ClineAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::copilot_cli::HARNESS_ID_STR
        && let Ok(adapter) = CopilotCliAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::goose::HARNESS_ID_STR
        && let Ok(adapter) = GooseAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::qwen_code::HARNESS_ID_STR
        && let Ok(adapter) = QwenCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::kimi_code::HARNESS_ID_STR
        && let Ok(adapter) = KimiCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::grok_build::HARNESS_ID_STR
        && let Ok(adapter) = GrokBuildAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::mistral_vibe::HARNESS_ID_STR
        && let Ok(adapter) = MistralVibeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::forge::HARNESS_ID_STR
        && let Ok(adapter) = ForgeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::kode::HARNESS_ID_STR
        && let Ok(adapter) = KodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::pi::HARNESS_ID_STR
        && let Ok(adapter) = PiAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::nanocoder::HARNESS_ID_STR
        && let Ok(adapter) = NanocoderAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::hermes::HARNESS_ID_STR
        && let Ok(adapter) = HermesAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::mimo::HARNESS_ID_STR
        && let Ok(adapter) = MimoAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::junie::HARNESS_ID_STR
        && let Ok(adapter) = JunieAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::amp::HARNESS_ID_STR
        && let Ok(adapter) = AmpAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::continue_dev::HARNESS_ID_STR
        && let Ok(adapter) = ContinueDevAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::factory_droid::HARNESS_ID_STR
        && let Ok(adapter) = FactoryDroidAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::gptme::HARNESS_ID_STR
        && let Ok(adapter) = GptmeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::cursor::HARNESS_ID_STR
        && let Ok(adapter) = CursorAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::kilo::HARNESS_ID_STR
        && let Ok(adapter) = KiloAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::windsurf::HARNESS_ID_STR
        && let Ok(adapter) = WindsurfAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::workbuddy::HARNESS_ID_STR
        && let Ok(adapter) = WorkBuddyAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::zed_acp::HARNESS_ID_STR
        && let Ok(adapter) = ZedAcpAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::auggie::HARNESS_ID_STR
        && let Ok(adapter) = AuggieAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::trae_agent::HARNESS_ID_STR
        && let Ok(adapter) = TraeAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::swe_agent::HARNESS_ID_STR
        && let Ok(adapter) = SweAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::gemini_cli::HARNESS_ID_STR
        && let Ok(adapter) = GeminiCliAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::amazon_q::HARNESS_ID_STR
        && let Ok(adapter) = AmazonQAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::roo_code::HARNESS_ID_STR
        && let Ok(adapter) = RooCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::legacy_kimi::HARNESS_ID_STR
        && let Ok(adapter) = LegacyKimiAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::antigravity::HARNESS_ID_STR
        && let Ok(adapter) = AntigravityAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::kiro::HARNESS_ID_STR
        && let Ok(adapter) = KiroAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::openclaw::HARNESS_ID_STR
        && let Ok(adapter) = OpenClawAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::zcode::HARNESS_ID_STR
        && let Ok(adapter) = ZcodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::crush::HARNESS_ID_STR
        && let Ok(adapter) = CrushAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::deepseek::HARNESS_ID_STR
        && let Ok(adapter) = DeepSeekAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::iflow::HARNESS_ID_STR
        && let Ok(adapter) = IflowAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::letta::HARNESS_ID_STR
        && let Ok(adapter) = LettaAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::openhands::HARNESS_ID_STR
        && let Ok(adapter) = OpenHandsAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::plandex::HARNESS_ID_STR
        && let Ok(adapter) = PlandexAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::warp::HARNESS_ID_STR
        && let Ok(adapter) = WarpAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::vibe_kanban::HARNESS_ID_STR
        && let Ok(adapter) = VibeKanbanAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::conductor::HARNESS_ID_STR
        && let Ok(adapter) = ConductorAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::sculptor::HARNESS_ID_STR
        && let Ok(adapter) = SculptorAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::copilot_coding_agent::HARNESS_ID_STR
        && let Ok(adapter) = CopilotCodingAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::muse_code::HARNESS_ID_STR
        && let Ok(adapter) = MuseCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::qoder_cli::HARNESS_ID_STR
        && let Ok(adapter) = QoderCliAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::zulu_cli::HARNESS_ID_STR
        && let Ok(adapter) = ZuluCliAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::minimax_code::HARNESS_ID_STR
        && let Ok(adapter) = MiniMaxCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::coco::HARNESS_ID_STR
        && let Ok(adapter) = CocoAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::fx::HARNESS_ID_STR
        && let Ok(adapter) = FxAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::jules::HARNESS_ID_STR
        && let Ok(adapter) = JulesAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::cursor_agent::HARNESS_ID_STR
        && let Ok(adapter) = CursorAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::kilocode_cli::HARNESS_ID_STR
        && let Ok(adapter) = KilocodeCliAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    if id == crate::adapters::agentty::HARNESS_ID_STR
        && let Ok(adapter) = AgenttyAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::avante_nvim::HARNESS_ID_STR
        && let Ok(adapter) = AvanteNvimAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::bb_app::HARNESS_ID_STR
        && let Ok(adapter) = BbAppAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::claw_code::HARNESS_ID_STR
        && let Ok(adapter) = ClawCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::codecompanion_nvim::HARNESS_ID_STR
        && let Ok(adapter) = CodecompanionNvimAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::codegeex::HARNESS_ID_STR
        && let Ok(adapter) = CodegeexAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::command_code::HARNESS_ID_STR
        && let Ok(adapter) = CommandCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::copilot_vscode::HARNESS_ID_STR
        && let Ok(adapter) = CopilotVscodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::devin::HARNESS_ID_STR
        && let Ok(adapter) = DevinAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::eca::HARNESS_ID_STR
        && let Ok(adapter) = EcaAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::every_code::HARNESS_ID_STR
        && let Ok(adapter) = EveryCodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::freebuff::HARNESS_ID_STR
        && let Ok(adapter) = FreebuffAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::gitclaw::HARNESS_ID_STR
        && let Ok(adapter) = GitclawAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::hax::HARNESS_ID_STR
        && let Ok(adapter) = HaxAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::jcode::HARNESS_ID_STR
        && let Ok(adapter) = JcodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::oh_my_codex::HARNESS_ID_STR
        && let Ok(adapter) = OhMyCodexAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::oh_my_opencode::HARNESS_ID_STR
        && let Ok(adapter) = OhMyOpencodeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::omo_ai::HARNESS_ID_STR
        && let Ok(adapter) = OmoAiAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::omp::HARNESS_ID_STR
        && let Ok(adapter) = OmpAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::openfang::HARNESS_ID_STR
        && let Ok(adapter) = OpenfangAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::orca::HARNESS_ID_STR
        && let Ok(adapter) = OrcaAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::ori::HARNESS_ID_STR
        && let Ok(adapter) = OriAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::prime_agent::HARNESS_ID_STR
        && let Ok(adapter) = PrimeAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::qoder_ide::HARNESS_ID_STR
        && let Ok(adapter) = QoderIdeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::reasonix::HARNESS_ID_STR
        && let Ok(adapter) = ReasonixAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::replit_agent::HARNESS_ID_STR
        && let Ok(adapter) = ReplitAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::t3code::HARNESS_ID_STR
        && let Ok(adapter) = T3codeAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::terminus::HARNESS_ID_STR
        && let Ok(adapter) = TerminusAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::unreal_agent::HARNESS_ID_STR
        && let Ok(adapter) = UnrealAgentAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::vix::HARNESS_ID_STR
        && let Ok(adapter) = VixAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }

    if id == crate::adapters::zaly::HARNESS_ID_STR
        && let Ok(adapter) = ZalyAdapter::new()
    {
        return Some(Box::new(adapter) as Box<dyn Adapter>);
    }
    // Ledger alias: the catalog row is kimi-code-cli; the adapter is kimi-code.
    if id == crate::adapters::kimi_code::HARNESS_ID_LEDGER_ALIAS
        && let Ok(ledger_id) = HarnessId::new(id)
    {
        return Some(Box::new(KimiCodeAdapter::from_ledger_id(ledger_id)) as Box<dyn Adapter>);
    }
    None
}

/// Build one adapter per catalog entry.
/// The returned count always equals `ENTRIES.len()`.
pub fn all_adapters() -> Vec<Box<dyn Adapter>> {
    let mut out = Vec::with_capacity(ENTRIES.len());
    for entry in ENTRIES {
        if let Some(adapter) = adapter_for_entry(entry) {
            out.push(adapter);
        }
    }
    out
}

pub(crate) fn adapter_for_id_fold(id: &str) -> Option<Box<dyn Adapter>> {
    let lowered = id.to_lowercase();
    let entry = ENTRIES
        .iter()
        .find(|entry| entry.id.to_lowercase() == lowered)?;
    adapter_for_entry(entry)
}

fn adapter_for_entry(entry: &CatalogEntry) -> Option<Box<dyn Adapter>> {
    if let Some(adapter) = concrete_adapter_for(entry.id) {
        return Some(adapter);
    }
    let harness_id = HarnessId::new(entry.id).ok()?;
    let adapter = GenericAdapter::new(
        harness_id,
        entry.display_name,
        entry.product_status,
        entry.research_doc,
        entry.last_verified,
        entry.support,
        entry.reason,
        entry.source,
    );
    Some(Box::new(adapter) as Box<dyn Adapter>)
}

/// Convenience: adapter ids as strings.
pub fn all_ids() -> Vec<&'static str> {
    ENTRIES.iter().map(|entry| entry.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Ledger tripwire: a row added or dropped without touching the ledger
    /// constants fails the build (compare against the derived values below).
    const LEDGER_ENTRY_COUNT: usize = 91;
    const LEDGER_SUPPORT_COUNTS: &[(AdapterSupport, usize)] = &[
        (AdapterSupport::Full, 26),
        (AdapterSupport::Constrained, 46),
        (AdapterSupport::SingleInstance, 1),
        (AdapterSupport::MigrationOnly, 7),
        (AdapterSupport::ResearchBlocked, 7),
        (AdapterSupport::Unsupported, 2),
        (AdapterSupport::ReadOnly, 2),
    ];

    fn count_support(want: AdapterSupport) -> usize {
        ENTRIES.iter().filter(|entry| entry.support == want).count()
    }

    #[test]
    fn catalog_size_matches_ledger() {
        assert_eq!(
            all_entries().len(),
            LEDGER_ENTRY_COUNT,
            "catalog drifted from the ledger; add the row's doc, entry, and bump the ledger constants"
        );
    }

    #[test]
    fn all_ids_are_valid_harness_ids() {
        for entry in ENTRIES {
            let parsed = HarnessId::new(entry.id);
            assert!(
                parsed.is_ok(),
                "catalog id `{}` must be a valid HarnessId: {:?}",
                entry.id,
                parsed.err()
            );
            let harness_id = parsed.unwrap();
            assert_eq!(harness_id.as_str(), entry.id);
        }
    }

    #[test]
    fn ids_are_unique() {
        let mut seen = HashSet::new();
        for entry in ENTRIES {
            let lower = entry.id.to_lowercase();
            assert!(
                seen.insert(lower.clone()),
                "duplicate id `{}` in catalog",
                entry.id
            );
        }
    }

    #[test]
    fn every_source_doc_exists() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for entry in ENTRIES {
            let doc = repo_root.join(entry.source);
            assert!(
                doc.is_file(),
                "entry `{}` sources `{}` but that doc is missing",
                entry.id,
                doc.display()
            );
        }
    }

    #[test]
    fn every_entry_has_source_and_reason() {
        use std::path::Path;
        for entry in ENTRIES {
            assert!(
                !entry.source.is_empty(),
                "entry `{}` must have a source link",
                entry.id
            );
            assert!(
                Path::new(entry.source)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md")),
                "entry `{}` source must be a markdown file, got `{}`",
                entry.id,
                entry.source
            );
            assert!(
                entry.source.starts_with("docs/harness-configs/"),
                "entry `{}` source must be under docs/harness-configs/, got `{}`",
                entry.id,
                entry.source
            );
            assert!(
                !entry.reason.is_empty(),
                "entry `{}` must have a non-empty reason",
                entry.id
            );
            assert!(
                !entry.display_name.is_empty(),
                "entry `{}` must have a display name",
                entry.id
            );
            assert!(
                !entry.research_doc.is_empty(),
                "entry `{}` must have research_doc",
                entry.id
            );
            assert!(
                !entry.last_verified.is_empty(),
                "entry `{}` must have last_verified date",
                entry.id
            );
        }
    }

    #[test]
    fn every_entry_has_support_state() {
        let mut has_full = false;
        let mut has_constrained = false;
        let mut has_single = false;
        let mut has_migration = false;
        let mut has_research = false;
        let mut has_unsupported = false;
        let mut has_read_only = false;
        for entry in ENTRIES {
            let support_str = entry.support.to_string();
            assert!(!support_str.is_empty());
            let json = serde_json::to_string(&entry.support).unwrap();
            let back: AdapterSupport = serde_json::from_str(&json).unwrap();
            assert_eq!(back, entry.support);
            match entry.support {
                AdapterSupport::Full => has_full = true,
                AdapterSupport::Constrained => has_constrained = true,
                AdapterSupport::SingleInstance => has_single = true,
                AdapterSupport::MigrationOnly => has_migration = true,
                AdapterSupport::ResearchBlocked => has_research = true,
                AdapterSupport::Unsupported => has_unsupported = true,
                AdapterSupport::ReadOnly => has_read_only = true,
            }
        }
        assert!(has_full, "must have at least one Full");
        assert!(has_constrained, "must have at least one Constrained");
        assert!(has_single, "must have at least one SingleInstance");
        assert!(has_migration, "must have at least one MigrationOnly");
        assert!(has_research, "must have at least one ResearchBlocked");
        assert!(has_unsupported, "must have at least one Unsupported");
        assert!(has_read_only, "must have at least one ReadOnly");
    }

    #[test]
    fn find_by_id_works() {
        let found = find_by_id("claude-code").unwrap();
        assert_eq!(found.display_name, "Claude Code");
        assert_eq!(found.support, AdapterSupport::Full);
        assert!(find_by_id("does-not-exist").is_none());
        // orchestrator entries are discoverable too
        let sculptor = find_by_id("sculptor").unwrap();
        assert_eq!(sculptor.display_name, "Sculptor");
    }

    #[test]
    fn all_adapters_span_catalog() {
        let adapters = all_adapters();
        let mut ids: Vec<String> = adapters
            .iter()
            .map(|adapter| adapter.id().to_string())
            .collect();
        ids.sort();
        let mut expected: Vec<String> = ENTRIES.iter().map(|entry| entry.id.to_owned()).collect();
        expected.sort();
        assert_eq!(
            ids.len(),
            expected.len(),
            "all_adapters must return an adapter for each catalog row"
        );
        assert_eq!(ids, expected, "adapter ids must match catalog ids");

        // Check that each adapter's research link matches catalog source.
        for adapter in &adapters {
            let id = adapter.id().to_string();
            let entry = find_by_id(&id).unwrap();
            assert_eq!(
                adapter.research_doc_link(),
                entry.research_doc,
                "adapter `{id}` research link must match catalog"
            );
            assert_eq!(
                adapter.last_verified_date(),
                entry.last_verified,
                "adapter `{id}` verified date must match catalog"
            );
            assert_eq!(
                adapter.display_name(),
                entry.display_name,
                "adapter `{id}` display name must match catalog"
            );
        }
    }

    #[test]
    fn support_partition_covers_every_entry_and_matches_ledger() {
        let derived_total: usize = LEDGER_SUPPORT_COUNTS
            .iter()
            .map(|(variant, _)| count_support(*variant))
            .sum();
        assert_eq!(
            derived_total,
            ENTRIES.len(),
            "the support ledger table must partition every entry; a support \
             variant missing from the table leaves rows uncounted"
        );
        for (variant, expected) in LEDGER_SUPPORT_COUNTS {
            assert_eq!(
                count_support(*variant),
                *expected,
                "ledger support count drifted for {variant}"
            );
        }
        let ledger_total: usize = LEDGER_SUPPORT_COUNTS.iter().map(|(_, n)| n).sum();
        assert_eq!(
            ledger_total, LEDGER_ENTRY_COUNT,
            "the support ledger must sum to the catalog ledger total"
        );
    }

    #[test]
    #[expect(
        clippy::excessive_nesting,
        reason = "test branching for support states"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "test exercises every trait method for 60 adapters"
    )]
    fn adapters_are_object_safe_and_usable_as_trait_objects() {
        let adapters = all_adapters();
        for adapter in adapters {
            // Exercise every trait method to ensure no panic.
            let _ = adapter.id();
            let _ = adapter.display_name();
            let _ = adapter.product_status();
            let _ = adapter.supported_platforms();
            let _ = adapter.adapter_revision();
            let _ = adapter.research_doc_link();
            let _ = adapter.last_verified_date();
            let _ = adapter.detection();
            let _ = adapter.version_resolution();
            let _ = adapter.config_surfaces();
            let _ = adapter.supported_operations();
            let _ = adapter.plan_mirror_exclusions();
            let _ = adapter.scan_candidates();
            // validate_instance on a synthetic instance for that adapter
            let harness_id = adapter.id();
            let inst = crate::instance::Instance {
                id: crate::ids::InstanceId::new("test-id-catalog-1").unwrap(),
                name: crate::ids::InstanceName::new("work").unwrap(),
                harness: harness_id.clone(),
                config_root: crate::paths::AbsolutePath::new(&crate::test_util::tmp_abs_str(
                    ".test-catalog-work",
                ))
                .unwrap(),
                binary: None,
                wrapper: None,
                isolation: Isolation::RelocatedRoot,
                origin: crate::state::InstanceOrigin::Created,
                ownership: crate::state::Ownership::SuperaiCreated,
                template: None,
                created_at: "2026-08-26T00:00:00Z".to_owned(),
                adapter_revision: crate::adapter::ADAPTER_REVISION.to_owned(),
            };
            let res = adapter.validate_instance(&inst);
            let entry = find_by_id(harness_id.as_str());
            let is_research_blocked =
                entry.is_some_and(|e| e.support == AdapterSupport::ResearchBlocked);
            let is_unsupported = entry.is_some_and(|e| e.support == AdapterSupport::Unsupported);
            // Only strictly enforce ResearchBlocked for concrete adapters (antigravity, openclaw);
            // generic ResearchBlocked adapters (crush, deepseek) currently return Ok and are allowed.
            let is_concrete_research_blocked = matches!(
                harness_id.as_str(),
                crate::adapters::antigravity::HARNESS_ID_STR
                    | crate::adapters::openclaw::HARNESS_ID_STR
                    | crate::adapters::crush::HARNESS_ID_STR
                    | crate::adapters::deepseek::HARNESS_ID_STR
            );
            let is_concrete_unsupported = matches!(
                harness_id.as_str(),
                crate::adapters::copilot_coding_agent::HARNESS_ID_STR
            );
            if (is_research_blocked && is_concrete_research_blocked)
                || (is_unsupported && is_concrete_unsupported)
            {
                assert!(
                    res.is_err(),
                    "validate_instance should be blocked for `{harness_id}` (research/unsupported)"
                );
                if let Err(err) = &res {
                    let dbg = format!("{err:?}");
                    assert!(
                        dbg.contains("ResearchBlocked")
                            || dbg.contains("research")
                            || dbg.contains("Unsupported")
                            || dbg.contains("unsupported")
                            || dbg.contains("cloud-owned"),
                        "expected blocked error for `{harness_id}`: {dbg}"
                    );
                }
            } else {
                // For generic ResearchBlocked (crush, deepseek) allow Ok; for others require Ok.
                assert!(
                    res.is_ok(),
                    "validate_instance should succeed for matching harness `{harness_id}`: {:?}",
                    res.err()
                );
            }
            let plan_res = adapter.plan_wrapper(&inst);
            let is_migration_only =
                entry.is_some_and(|e| e.support == AdapterSupport::MigrationOnly);
            // Only strictly enforce blocking for concrete migration/research/unsupported adapters;
            // generic adapters still return Ok and are allowed until they get concrete impls.
            let is_concrete_blocked = matches!(
                harness_id.as_str(),
                crate::adapters::gemini_cli::HARNESS_ID_STR
                    | crate::adapters::amazon_q::HARNESS_ID_STR
                    | crate::adapters::roo_code::HARNESS_ID_STR
                    | crate::adapters::legacy_kimi::HARNESS_ID_STR
                    | crate::adapters::antigravity::HARNESS_ID_STR
                    | crate::adapters::openclaw::HARNESS_ID_STR
                    | crate::adapters::crush::HARNESS_ID_STR
                    | crate::adapters::deepseek::HARNESS_ID_STR
                    | crate::adapters::iflow::HARNESS_ID_STR
                    | crate::adapters::copilot_coding_agent::HARNESS_ID_STR
                    | crate::adapters::vibe_kanban::HARNESS_ID_STR
            );
            if (is_research_blocked || is_migration_only || is_unsupported) && is_concrete_blocked {
                assert!(
                    plan_res.is_err(),
                    "plan_wrapper should be blocked for `{harness_id}` (support {support:?})",
                    support = entry.map(|e| e.support)
                );
            } else {
                let plan = match plan_res {
                    Ok(p) => p,
                    Err(err) => panic!("plan_wrapper should succeed for `{harness_id}`: {err:?}"),
                };
                assert!(!plan.description.is_empty());
            }
        }
    }

    #[test]
    fn isolation_coverage_is_not_all_one_value() {
        let mut distinct: Vec<Isolation> = Vec::new();
        for entry in ENTRIES {
            if !distinct.contains(&entry.isolation) {
                distinct.push(entry.isolation);
            }
        }
        // We expect several isolation classes to be represented.
        assert!(
            distinct.len() >= 5,
            "expected at least 5 distinct isolation classes, got {distinct:?}"
        );
    }

    #[test]
    fn product_status_is_set_and_serializable() {
        for entry in ENTRIES {
            let json = serde_json::to_string(&entry.product_status).unwrap();
            let back: ProductStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, entry.product_status);
            assert!(!entry.product_status.to_string().is_empty());
        }
    }
}
