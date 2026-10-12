# Gitagent (ledger id `gitclaw`) — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/open-gitagent/gitagent](https://github.com/open-gitagent/gitagent) (TypeScript 5.7, Node 18+), npm `gitagent` / `@open-gitagent/gitagent`, and the Qumulo llm-agents nix index entry `packages/gitclaw`[1][2]. Gitagent's pitch is "your agent IS a git repository": identity, rules, memory, tools, and skills are version-controlled files inside the agent dir, so you fork an agent, branch a personality, and `git log` its memory[1]. Latest release v2.2.0 (2026-08-20), pushed 2026-10-09: alive. Ledger id note: the task list and the nix index call this `gitclaw`; the upstream name is gitagent and the executable is `gitagent`.

---

## 1. Install, executable, version

```bash
bash <(curl -fsSL "https://raw.githubusercontent.com/open-gitagent/gitagent/main/install.sh?$(date +%s)")
# or manually:
npm install -g @open-gitagent/gitagent   # slim CLI + SDK
npm install -g @open-gitagent/voice      # optional voice mode + web UI
gitagent "Explain this project"
```

The installer walks through API key setup and launches a voice UI at `http://localhost:3333`; `GITAGENT_SLIM=1` skips voice. Voice split out of the main bundle in 2.0 after supply-chain scanners flagged it; `gitagent --voice` loads it dynamically[1].

## 2. Config files: the agent directory

An agent dir (default: cwd, override with `--dir`) scaffolds on first run as[1]:

```text
agent.yaml          # model, tools, runtime config
SOUL.md             # personality and identity
RULES.md            # behavioral constraints
memory/             # git-committed memory with full history
tools/*.yaml        # declarative tool definitions
skills/<name>/      # composable skill modules
hooks/hooks.yaml    # lifecycle hooks (script or programmatic)
knowledge/index.yaml
config/default.yaml # default environment config
config/<env>.yaml   # environment overrides (-e/--env selects)
plugins/            # local plugins
```

`agent.yaml` keys include `model` (`"provider:model-id"` string) and `env` (environment config name)[1]. Plugin resolution order: bundled, global `~/.gitagent/plugins/<name>/`, installed `<agent-dir>/.gitagent/plugins/<name>/`[1]. Plugin config resolution: `agent.yaml config` > env var > manifest default, with `${VAR}` interpolation[1].

## 3. Environment and credentials

`OPENAI_API_KEY` for the default provider; models select Anthropic and others via the `provider:model` form[1]. Repo mode uses `GITHUB_TOKEN` or `GIT_TOKEN` (or `--pat`)[1].

## 4. MCP

MCP servers are declared in `agent.yaml` under `mcp_servers`[1]:

```yaml
mcp_servers:
  filesystem:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
  remote:
    url: "https://mcp.example.com/mcp"
```

stdio entries take `command`/`args`/`env`/`cwd`; `${VAR}` is interpolated from the environment. v2 supports MCP tools; resources and prompts are not yet exposed. SDK `mcpServers` merge with agent.yaml values (SDK wins on collision)[1].

## 5. Headless / SDK

```bash
gitagent -p "single-shot prompt"                     # skip the REPL
gitagent --repo https://github.com/org/repo --pat ghp_xxx "Fix the login bug"
gitagent --repo ... --session gitagent/session-a1b2c3d4 "Continue"
```

Flags: `--dir/-d`, `--repo/-r`, `--pat`, `--session`, `--model/-m provider:model`, `--sandbox/-s` (sandbox VM), `--prompt/-p`, `--env/-e`[1]. The SDK (`import { query } from "gitagent"`) runs the agent in-process, no subprocesses, mirroring the Claude Agent SDK pattern[1].

## 6. Multi-instance wrappers

Isolation is the agent dir: separate `--dir` trees are separate agents with their own memory, rules, and MCP. Nothing user-global except the plugin cache at `~/.gitagent/plugins/`; a wrapper that wants independent instances gives each one its own agent dir and its own `GITHUB_TOKEN`/provider key.

## 7. Not verified upstream

- The full `agent.yaml` schema (README documents the directory tree and a subset of keys).
- Where `~/.gitagent/` holds anything besides plugins (auth state was not described).
- Sandbox VM mechanics behind `--sandbox`.
- A version-probe command (`gitagent --version` is implied by npm packaging but not documented in the README).

## Sources

- [1] https://raw.githubusercontent.com/open-gitagent/gitagent/main/README.md: install, agent-dir tree, CLI flags, MCP config, plugins, SDK
- [2] https://github.com/open-gitagent/gitagent/releases/tag/v2.2.0: latest release (2026-08-20); repo pushed 2026-10-09
- [3] https://github.com/Qumulo/llm-agents/tree/main/packages/gitclaw: nix index entry (owner open-gitagent, repo gitagent, v2.2.0)
