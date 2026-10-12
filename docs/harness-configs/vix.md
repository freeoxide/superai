# vix — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/get-vix/vix](https://github.com/get-vix/vix) (v0.6.1, pushed 2026-09-23) and getvix.dev[1]. vix is a token-efficient coding agent: stem agents share one cacheable conversation across explore/plan/execute phases, and a Tree-sitter virtual filesystem lets the model read and edit minified code (claimed 20 to 50 percent token reduction). Daemon plus N isolated instances is the documented concurrency model. macOS and Linux only[1].

---

## 1. Install, executable, version

```bash
curl -fsSL https://getvix.dev/install.sh | bash
brew tap get-vix/vix && brew install vix
```

`ANTHROPIC_API_KEY` must be in the environment (v0.x's documented auth path). Start the daemon, then spawn instances[1]:

```bash
vixd          # daemon
vix           # each instance is isolated
```

## 2. Config files

Providers are extended or overridden by dropping a `providers.json` into `~/.vix/` (user) or `./.vix/` (project); the file is merged on top of embedded defaults: new providers append, existing ones field-patch. The full field reference, recipes, model discovery, and OAuth configuration are in the repo's PROVIDERS.md[1]. Workflows (multi-phase agent pipelines: agent/bash/tool steps with templating, branching, parallelism, history forking) are defined in `settings.json`; the README points at coming documentation for that schema[1].

## 3. Providers

Built-in: Anthropic, OpenAI, OpenRouter, OrcaRouter, AWS Bedrock, Ollama, llama.cpp, "and more"[1].

## 4. Features that touch the rest of the system

- Skills, MCP servers, subagents, LSP-backed code intelligence, and sandboxed execution are all listed capabilities[1]; their configuration files are not spelled out in the README.
- Self-evolving behavior: vix writes its own scheduled jobs, watchers, and alerts[1], so a wrapper should expect background work started by the agent itself.

## 5. Not verified upstream

- The providers.json field schema (PROVIDERS.md was linked, not fetched).
- The settings.json workflow schema (README defers to future documentation).
- MCP server configuration path and format.
- A config-dir relocation variable (none documented; `~/.vix` looks fixed).
- Version probe and exit codes.

## Sources

- [1] https://raw.githubusercontent.com/get-vix/vix/main/README.md: install, vixd/instance model, providers.json merge semantics, settings.json workflows, feature list, benchmarks
- [2] https://github.com/get-vix/vix/releases/tag/v0.6.1: latest release; repo pushed 2026-09-23
- [3] https://github.com/Qumulo/llm-agents/tree/main/packages/vix: nix index entry (owner get-vix, v0.6.1)
