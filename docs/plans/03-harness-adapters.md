# Subplan 03 — version-aware harness adapters

Parent: [master plan](master-plan.md)  
Task prefix: HAD  

Status: complete. Run 3 closed every exit-gate item over the 48 registered catalog rows.

What shipped: fixture-backed adapters for every surface documented under harness-configs,
resolving paths, versions, precedence, writable fields, isolation, and discovery.

## Provisional support ledger

State below is implementation entry gate, not a claim that code already supports it. Research
gaps from source docs keep a surface read-only or research-blocked until closed.

| Product surface | Source | Main writable formats/surfaces | Isolation class | Entry gate |
|---|---|---|---|---|
| Aider | [aider.md](../harness-configs/aider.md) | YAML, env, JSON metadata, explicit CLI paths | explicit-config / HOME | Full candidate |
| Amazon Q Developer CLI | [amazon-q-cli.md](../harness-configs/amazon-q-cli.md) | JSON settings/agents/MCP; shared SSO cache | project/account constrained | MigrationOnly |
| Amp | [amp.md](../harness-configs/amp.md) | JSON/JSONC settings; explicit settings file | explicit-config, account constrained | Constrained |
| Antigravity CLI | [antigravity-cli.md](../harness-configs/antigravity-cli.md) | settings plus harness-owned auth; incomplete paths | HOME workaround | ResearchBlocked |
| Auggie | [auggie.md](../harness-configs/auggie.md) | JSON settings, .augment rules/commands | account/workspace constrained | Constrained |
| ChatGPT Desktop (Codex) | [chatgpt-desktop.md](../harness-configs/chatgpt-desktop.md) | read-only on `~/.codex` (owned by codex-cli); no app-level relocation | fixed-path shared store | ReadOnly; alias via codex-cli |
| Claude Code | [claude-code.md](../harness-configs/claude-code.md) | JSON/JSONC settings, env, MCP, skills/plugins | relocated-root | Full candidate |
| Claude Desktop | [claude-desktop.md](../harness-configs/claude-desktop.md) | writable `mcpServers` JSON at the per-OS default root | fixed-path (relocation verified-absent) | Constrained; aliasing refused |
| Cline | [cline.md](../harness-configs/cline.md) | JSON settings/MCP/rules plus VS Code storage | IDE user-data | Full candidate after OS verification |
| Codex CLI | [codex-cli.md](../harness-configs/codex-cli.md) | TOML, rules, skills, MCP, profile files | relocated-root/profile | Full candidate |
| Continue | [continue-dev.md](../harness-configs/continue-dev.md) | YAML, env, rules/prompts/MCP; legacy JSON | project/explicit CLI | Constrained; hosted features excluded |
| GitHub Copilot CLI | [copilot-cli.md](../harness-configs/copilot-cli.md) | JSONC, MCP/LSP JSON, skills/plugins | relocated-root | Full candidate |
| Copilot Coding Agent | [copilot-cli.md](../harness-configs/copilot-cli.md) | repository/org cloud settings | cloud-owned | Unsupported for mutation; document only |
| Crush | [crush.md](../harness-configs/crush.md) | executable crushrc; deprecated JSON | project/XDG | ResearchBlocked for writes |
| Cursor IDE and Agent CLI | [cursor.md](../harness-configs/cursor.md) | IDE JSON/storage, CLI config, MCP/rules | CLI root plus IDE user-data | Constrained |
| DeepSeek Harness | [deepseek-harness.md](../harness-configs/deepseek-harness.md) | provider catalog; plugin config incomplete | relocated-root | ResearchBlocked, developer preview |
| Factory Droid | [factory-droid.md](../harness-configs/factory-droid.md) | layered JSON, MCP/skills/agents | project/HOME constrained | Constrained |
| Forge | [forge.md](../harness-configs/forge.md) | config directory, provider profiles | relocated config | Full candidate |
| Gemini CLI | [gemini-cli.md](../harness-configs/gemini-cli.md) | JSON settings, env, extensions | relocated-root | MigrationOnly; retired consumer tiers |
| Goose | [goose.md](../harness-configs/goose.md) | YAML config/recipes/extensions | relocated-root | Full candidate after unverified keys closed |
| gptme | [gptme.md](../harness-configs/gptme.md) | TOML/YAML/env/workspaces/log roots | workspace plus explicit state | Constrained |
| Grok Build | [grok-build.md](../harness-configs/grok-build.md) | TOML, JSON overlay, skills/plugins/MCP | relocated-root | Full candidate |
| Hermes Agent | [hermes-agent.md](../harness-configs/hermes-agent.md) | YAML/env, profiles, skills/MCP | relocated-root/profile | Full candidate |
| iFlow CLI | [iflow-cli.md](../harness-configs/iflow-cli.md) | JSON/env/MCP/agents | env/explicit system file | MigrationOnly; shutdown |
| Junie CLI | [junie-cli.md](../harness-configs/junie-cli.md) | JSON, models/MCP, skills/agents | relocated-root | Full candidate; EAP gated |
| Kilo Code extension and CLI | [kilo-code.md](../harness-configs/kilo-code.md) | layered JSONC, agents/MCP | inline/HOME plus IDE user-data | Constrained until full root verified |
| Kimi Code CLI | [kimi-cli.md](../harness-configs/kimi-cli.md) | TOML plus MCP JSON | relocated-root | Full candidate |
| Legacy Kimi CLI | [kimi-cli.md](../harness-configs/kimi-cli.md) | TOML/MCP legacy root | legacy root | MigrationOnly |
| Kiro CLI/IDE | [kiro.md](../harness-configs/kiro.md) | settings/rules/MCP under KIRO_HOME | relocated-root | ReadOnly until research gaps close |
| Kode CLI | [kode.md](../harness-configs/kode.md) | JSON, MCP, agents/skills | relocated-root | Full candidate |
| Letta Code | [letta-code.md](../harness-configs/letta-code.md) | client config plus server/provider state | separate server/state | Constrained |
| MiMo Code | [mimo-code.md](../harness-configs/mimo-code.md) | JSON/JSONC, inline config, plugins/MCP | relocated-root | Full candidate |
| Mistral Vibe | [mistral-vibe.md](../harness-configs/mistral-vibe.md) | TOML, skills/tools/MCP | relocated-root | Full candidate |
| Nanocoder | [nanocoder.md](../harness-configs/nanocoder.md) | JSON provider/MCP/preferences | relocated/explicit files | Full candidate |
| OpenClaw | [openclaw.md](../harness-configs/openclaw.md) | config/provider JSON plus daemon state | relocated-root/daemon | ResearchBlocked until gateway/schema complete |
| OpenCode | [opencode.md](../harness-configs/opencode.md) | layered JSONC, agents/plugins/MCP | relocated/inline config | Full candidate |
| OpenHands | [openhands.md](../harness-configs/openhands.md) | V1 JSON/env; V0 TOML; Docker persistence | persistence root/container | Constrained, version split required |
| Pi | [pi.md](../harness-configs/pi.md) | JSON settings/auth/models, extensions | relocated-root | Full candidate; MCP absent by design |
| Plandex | [plandex.md](../harness-configs/plandex.md) | env and server/model-pack config | provider/server scoped | Constrained |
| Qwen Code | [qwen-code.md](../harness-configs/qwen-code.md) | layered JSON/env/MCP | relocated settings/runtime | Full candidate |
| Roo Code | [roo-code.md](../harness-configs/roo-code.md) | VS Code storage, YAML modes, JSON MCP | IDE user-data | MigrationOnly; archived |
| SWE-agent | [swe-agent.md](../harness-configs/swe-agent.md) | composed YAML | explicit config/batch | Full candidate for config-run instances |
| Trae Agent | [trae-agent.md](../harness-configs/trae-agent.md) | YAML; deprecated JSON | explicit config/env | Full candidate |
| Warp Agent CLI/app | [warp.md](../harness-configs/warp.md) | CLI TOML, MCP JSON, workflows YAML | Linux XDG/profile constrained | Constrained |
| Windsurf/Devin Desktop | [windsurf.md](../harness-configs/windsurf.md) | MCP JSON, rules/skills; IDE storage | IDE user-data | Constrained |
| WorkBuddy / CodeBuddy CLI (cbc) | [workbuddy.md](../harness-configs/workbuddy.md) | JSON models/settings/MCP under `~/.codebuddy`; env auth | relocated-root (`CODEBUDDY_CONFIG_DIR`) | Constrained; desktop app GUI-only, documented not mutated |
| ZCode | [zcode.md](../harness-configs/zcode.md) | versioned-path JSON | fixed path | SingleInstance; schema research gate |
| Zed AI/ACP | [zed-acp.md](../harness-configs/zed-acp.md) | JSON settings, ACP wrappers, MCP | wrapper registrations | Constrained; version gates required |
| Vibe Kanban | [orchestrators.md](../harness-configs/orchestrators.md) | app profiles/env/MCP/worktrees | orchestrator profiles | MigrationOnly/community-maintained |
| Conductor | [orchestrators.md](../harness-configs/orchestrators.md) | user/repo TOML, env, scripts | macOS worktrees/profiles | Constrained |
| Sculptor | [orchestrators.md](../harness-configs/orchestrators.md) | env, harness settings, containers | workspace/container | Constrained |


Priority gaps:

- Antigravity config/auth/MCP paths and HOME behavior.
- DeepSeek full schema, plugins, profiles, skills, lifecycle, sandbox.
- Crush command-backed mutation or safe managed include.
- Kiro full schema and BYO limitations.
- OpenClaw schema, gateway, ports, plugins/skills.
- ZCode full schema/MCP/skills and version path history.
- Zed current gateway/ACP/MCP shapes.
- Goose exact unverified keys.
- Warp XDG behavior outside documented Linux path.
- IDE credential/global-storage relocation on each OS.


## Exit gate

- [x] Every ledger row is registered in code/data.
- [x] Every writable row passes conformance suite.
- [x] Every limited row returns typed state and evidence.
- [x] Unknown versions cannot write.
- [x] Auto-managed/keychain/opaque files cannot enter mutation plans.
- [x] All path/version/precedence claims have fixtures or official evidence.
- [x] No harness is dropped because it lacks full isolation or custom endpoints.

## Residuals

> Completion record (run 3): all 48 catalog rows registered and fixture-corpus-covered
> (`all_adapters_span_catalog`, `ledger_coverage_has_known_harnesses` — now enforced over
> every id, plus the 13 formerly corpus-less surfaces adopted, area 3); root-shape/semantic
> validators + era-boundary fixtures on the 8 spine adapters (area 3, commit-time
> `SchemaValidation`/`UnsupportedVersion` gates) and MCP/plugin declarations across all 48
> adapters (area 5: 33 MCP dests + 15 explicit absences, 7 plugin decls + 41 absences,
> exact-partition test-enforced); effective provider/model/key-sink inspection closed via
> PRV-05 (area 4). Unknown-version write lock enforced at the raw-editor boundary with one
> documented in-code exception (zcode fixed-path schema accepts unknown binary version when
> the fixed-path config exists, zcode.rs). Adapter-level external-edit conflict detection is
> exercised at the mutation layer all adapter writes share (per-step-kind §4.2 tests, area
> 2); the `conflict_detection_placeholder_no_panic` adapter tests remain as smoke coverage
> by area-3 decision.
