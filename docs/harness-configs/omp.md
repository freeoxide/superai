# omp (oh-my-pi) — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/can1357/oh-my-pi](https://github.com/can1357/oh-my-pi) (omp, by Stencil Labs; a fork of badlogic/pi-mono by Mario Zechner), npm `@oh-my-pi/pi-coding-agent` (latest 18.10.1), and omp.sh[1][2]. omp is a TypeScript/Rust coding agent "with the IDE wired in": 60+ providers, 31 built-in tools, LSP and DAP operations, and the same engine exposed four ways (TUI, one-shot, Node SDK, RPC/ACP). Distinct from the catalogued pi harness: that is upstream pi-mono's CLI with its own doc ([pi.md](pi.md)); omp is the downstream fork with its own config tree. Pushed 2026-10-12: alive.

---

## 1. Install, executable, version

```sh
curl -fsSL https://omp.sh/install | sh          # macOS / Linux (binary fallback via --binary)
brew install can1357/tap/omp
bun install -g @oh-my-pi/pi-coding-agent        # bun >= 1.3.14
nix profile add github:can1357/oh-my-pi         # flake; Home Manager module available
```

Windows: `irm https://omp.sh/install.ps1 | iex`. Shell completions are self-generated (`omp completions bash|zsh|fish`)[1].

## 2. Config files

Two YAML files under `~/.omp/agent/`[1]:

- `~/.omp/agent/models.yml`: custom providers, appended or field-patched onto the built-in catalog.

```yaml
providers:
  spark:
    baseUrl: http://192.168.10.223:8000/v1
    api: openai-completions      # or openai-responses, anthropic-messages, bedrock-converse-stream,
    apiKey: dummy                # google-generative-ai, google-vertex, typesafe, and more
    models:
      - id: minimax-m3
        contextWindow: 100000
        maxTokens: 32000
```

- `~/.omp/agent/config.yml`: routing and defaults (`modelRoles.default: spark/minimax-m3`), per-role or per-model `retry.fallbackChains`, `enabledModels`/`disabledProviders` with `path:`-prefix scoping to pin a model set per repo, and round-robin credential stacks per provider[1].

Per-invocation: `--config <file>` takes a YAML overlay with nested keys (for example `mcp.enableProjectConfig: false`)[1]. A Home Manager module can own `programs.omp.settings` declaratively[1].

## 3. Providers

Direct API providers (Anthropic, OpenAI, Codex, Gemini, Vertex, Antigravity, xAI/SuperGrok, DeepSeek, Mistral, Groq, Cerebras, Fireworks, Together, Bedrock, Azure, OpenRouter, gateways, and local Ollama/LM Studio/llama.cpp/vLLM/LiteLLM) plus subscription-routed plans: Cursor, GitHub Copilot, GitLab Duo, Devin, Kimi Code, Moonshot, MiniMax Coding Plan (+CN), Alibaba Coding Plan, Qwen Portal, Z.AI/GLM Coding Plan, Zhipu, MiMo, Kilo, and more. `/login` attaches subscriptions[1]. Web search is built in across 23 backends, pinned by provider name and key (`PERPLEXITY_API_KEY`, `XAI_API_KEY`, `ZAI_API_KEY`, `EXA_API_KEY`, `TAVILY_API_KEY`, `BRAVE_API_KEY`, ...)[1].

## 4. MCP and inheritance

On first run omp inherits whatever is on disk: rules, skills, and MCP servers from `.claude`, `.cursor`, `.windsurf`, `.gemini`, `.codex`, `.cline`, `.github/copilot`, and `.vscode`, with no migration script[1]. Opening a repository loads its project inputs by design; project `.mcp.json` can be excluded per invocation with the `mcp.enableProjectConfig: false` overlay, extension discovery with `--no-extensions` or an exact `--trusted-extension <path>` allowlist, and built-ins with `--no-tools`[1]. That inheritance makes omp a read-consumer of most other harnesses in this catalog.

## 5. Entry points

`omp` (TUI), `omp -p` (one-shot prompt), `omp --mode rpc` and `omp acp` (stdio control for other programs and ACP editors), and a Node/TypeScript SDK embedding the session (`ModelRegistry`, `SessionManager`, `createAgentSession`, `discoverAuthStorage`)[1]. `omp setup` is the guided default-model flow; `/model` and `/reload-plugins` manage runtime changes[1].

## 6. Not verified upstream

- The full `config.yml` key schema (the README documents the routing knobs; the complete reference lives at omp.sh/docs/providers, not fetched).
- Credential storage location and format (auth storage is SDK-discoverable; paths were not documented in the README).
- A config-dir relocation variable for `~/.omp` itself (Home Manager settings are declarative install-time; no runtime env override was found).
- Exit codes for `omp -p`.

## Sources

- [1] https://raw.githubusercontent.com/can1357/oh-my-pi/main/README.md: install matrix, models.yml/config.yml shapes, provider and subscription lists, inheritance of .claude/.codex/etc, trust flags, entry points, completions
- [2] https://registry.npmjs.org/@oh-my-pi/pi-coding-agent: npm package, latest 18.10.1 (checked 2026-10-12)
- [3] https://github.com/Qumulo/llm-agents/tree/main/packages/omp: nix index entry (owner can1357, repo oh-my-pi)
