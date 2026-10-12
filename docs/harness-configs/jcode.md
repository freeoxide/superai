# jcode — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/1jehuang/jcode](https://github.com/1jehuang/jcode) (Rust TUI, docs at jcode.sh) and the Qumulo llm-agents nix index entry `packages/jcode`[1][2]. jcode is a performance-focused coding agent harness: RAM-frugal headless sessions that share one daemon, an embedded-memory system, and a swarm mode where the main agent spawns worker agents in parallel. Latest release v0.95.0 (nix pins v0.94.0); pushed 2026-10-12: alive.

---

## 1. Install, executable, version

```bash
curl -fsSL https://jcode.sh/install | bash      # macOS / Linux
irm https://jcode.sh/install.ps1 | iex           # Windows
jcode update
```

Headless-session benchmarks in the README measured ~10 MB per session with sessions sharing one daemon[1].

## 2. Config files

Main config: `~/.jcode/config.toml`[1]. Documented shapes[1]:

```toml
[provider]
default_provider = "my-api"
default_model = "my-model-id"

[providers.my-api]
type = "openai-compatible"          # or "anthropic-compatible"
base_url = "https://llm.example.com/v1"
api_key_env = "JCODE_PROVIDER_MY_API_API_KEY"   # reference, not the secret
env_file = "provider-my-api.env"
disable_reasoning_heuristics = true  # stop gpt-5-* names auto-enabling reasoning_effort

[[providers.my-api.models]]
id = "my-model-id"
context_window = 128000             # used when /v1/models has no usable response
reasoning = true
reasoning_effort = "high"

[display]
emoji = false

swarm_root_effort = "low"           # /effort swarm root level
swarm_deep_root_effort = "high"
```

Anthropic-compatible gateways take the same profile shape with `auth = "bearer"` (or custom-header / none) and a `[providers.<name>.headers]` table[1]. `jcode provider add <name> --base-url ... --api-key-stdin --set-default [--model-catalog|--no-api-key|--context-window N]` writes profiles without hand-editing, stores secrets in jcode's private app config directory when requested, and prints validation commands[1].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `JCODE_SWARM_ROOT_EFFORT`, `JCODE_SWARM_DEEP_ROOT_EFFORT` | Swarm root effort overrides | [1] |
| `JCODE_STREAM_IDLE_TIMEOUT_SECS` | Streaming idle timeout (default 180s; scaled 2x/3x/4x at high/xhigh/max effort); also `[provider] stream_idle_timeout_secs` | [1] |
| `JCODE_NO_EMOJI=1` | Disable emoji globally (same as `[display] emoji = false`) | [1] |
| `<PROFILE>_API_KEY` style vars | Referenced from profiles via `api_key_env` | [1] |

## 4. Login flows and MCP

`jcode login --provider <id>` covers subscription OAuth (claude, openai, gemini, copilot, azure, alibaba-coding-plan, fireworks, novita, minimax, meta-muse, lmstudio, ollama, openai-compatible) plus built-in OpenAI-compatible profiles: `openrouter`, `orcarouter`, `deepseek`, `zai`, `kimi`, `moonshotai`, `opencode` (Zen), `opencode-go`, `yolo-auto`, `302ai`, `baseten`, `cortecs`, `huggingface`, `nebius`, `scaleway`, `stackit`, `firmware`. `--no-browser`/`--headless` prints the auth URL for SSH sessions[1].

MCP is separate from config.toml[1]: global `~/.jcode/mcp.json` and project `.jcode/mcp.json`, both accepting the `mcpServers` key or jcode's historical `servers` key. jcode also reads Claude Code's config (`~/.claude.json` top-level and per-project `projects.<abs_path>.mcpServers`, `.mcp.json` at repo root, legacy `.claude/mcp.json`) and imports `~/.codex/config.toml` MCP servers into `~/.jcode/mcp.json` when it does not exist. stdio servers only; HTTP/SSE entries are recognized and skipped with a log line. Per-server `timeout_secs` (default 30)[1]. These cross-reads overlap the claude-code and codex-cli harnesses; a wrapper mutating those files will change jcode's view.

## 5. Swarm, sessions, headless

`/agents` sets worker models per session (swarm/review/judge/memory/ambient roles) with `/agents global` for saved defaults; agents can spawn their own swarms via a swarm tool, headless or headed[1]. `--provider-profile <name>` selects a profile; `jcode --provider-profile my-api run 'hello'` and `auth-test` are the documented scripting pair[1].

## 6. Not verified upstream

- A config-dir relocation env var (none documented; the root is fixed at `~/.jcode` in the fetched material).
- Exit codes and a JSON output mode for `run` (README shows streaming usage only).
- Skills/AGENTS.md guidance handling (memory system is documented; skill-file locations are not in the README).
- Version flag convention (`jcode update` is documented; `--version` was not).

## Sources

- [1] https://raw.githubusercontent.com/1jehuang/jcode/master/README.md: install, config.toml provider shapes, provider add, login profiles, MCP cross-reads, swarm config, env vars, daemon/headless benchmarks
- [2] https://github.com/Qumulo/llm-agents/tree/main/packages/jcode: nix index entry (owner 1jehuang, v0.94.0 pin)
- [3] https://github.com/1jehuang/jcode/releases/tag/v0.95.0: latest release at compile time
