# agentty — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/1ay1/agentty](https://github.com/1ay1/agentty) (MIT, C++26), agentty.org's configuration and environment-variables pages, and the Qumulo llm-agents nix index entry `packages/agentty`[1][2][3][5]. agentty is a single static binary (16.7 MB, ~3 ms cold start) coding agent in C++26, current release v0.9.19 (nix pins v0.9.11), pushed 2026-10-12. Two on-disk config locations, JSON settings, and one of the larger documented env-var surfaces in this catalog[2].

---

## 1. Install, executable, version

```bash
curl -fsSL https://raw.githubusercontent.com/1ay1/agentty/master/install.sh | sh
agentty
```

Also packaged: deb/rpm (repo releases), AUR (`agentty-bin`/`agentty-git`), `brew tap 1ay1/tap && brew install agentty`, scoop bucket, winget (`agentty.agentty`), and a Termux build path[1].

## 2. Config files and storage

User level, under `~/.agentty/` (whole root overridable via `AGENTTY_HOME`)[2]:

| Path | Purpose |
|---|---|
| `~/.agentty/settings.json` | Persisted in-app settings (written by the pickers; docs say there is nothing to hand-edit) |
| `~/.agentty/mcp.json` | User MCP servers (hand-edited) |
| `~/.agentty/hooks.json`, `~/.agentty/skills/`, `~/.agentty/AGENTS.md` | Hand-edited config surfaces |
| `~/.agentty/threads/` | Conversations, mode 0700 |
| `~/.agentty/credentials/` | Sign-ins and keys, 0700; plaintext JSON by default, optional AES-256-GCM encryption or OS keystore |
| `~/.agentty/state/`, `cache/`, `logs/`, `memory.jsonl` | Approval hashes, indexes (logs rotate at 32 MB), memory |

Project level: `<project>/.agentty/` with `mcp.json`, `skills/`, `cache/`, `state/`, `memory.jsonl`; project scope shadows user scope for skills, agents, and slash commands[2].

Guidance files merge low to high: `~/.agentty/AGENTS.md`, `~/.agents/AGENTS.md`, `<project>/AGENTS.md`, then the nearest nested `AGENTS.md` via an upward walk that never leaves the workspace. `~/CLAUDE.md`, `<project>/CLAUDE.md`, and `<project>/CLAUDE.local.md` are also read, each capped at 64 KiB[2]. The workspace is the launch directory, override with `agentty --workspace <dir>`[2].

## 3. Environment variables (selection)

Storage and security[3]: `AGENTTY_HOME`, `AGENTTY_THREADS_DIR`, `AGENTTY_CREDENTIALS_DIR`, `AGENTTY_STATE_DIR`, `AGENTTY_CACHE_DIR`, `AGENTTY_LOGS_DIR`, `AGENTTY_PROJECT_DIR`; credential encryption `AGENTTY_USE_KEYSTORE`, `AGENTTY_ENCRYPT_PASSPHRASE`/`AGENTTY_PASSPHRASE`, `AGENTTY_KDF`; `AGENTTY_TLS_PINS`, `AGENTTY_INSECURE`, `AGENTTY_SOCKS_PROXY`.

Providers[3]: `ANTHROPIC_API_KEY`, `CLAUDE_CODE_OAUTH_TOKEN` (reuses a Claude Code OAuth token), `OPENAI_API_KEY`, `GROQ_API_KEY`, `OPENROUTER_API_KEY`, `TOGETHER_API_KEY`, `CEREBRAS_API_KEY`, `DEEPSEEK_API_KEY`, `XAI_API_KEY`, `MISTRAL_API_KEY`, `GEMINI_API_KEY`/`GOOGLE_API_KEY`, `FIREWORKS_API_KEY`; host overrides `AGENTTY_API_HOST`, `AGENTTY_OAUTH_HOST`; `AGENTTY_OLLAMA_HOST` plus Ollama tuning; `--provider` persists; live switch with `^P`[1][3]. Credential priority: `-k` flag > `ANTHROPIC_API_KEY` > `CLAUDE_CODE_OAUTH_TOKEN` > on-disk credentials[2]. Env keys win over stored keys and are never written back to settings.json[2].

MCP and ACP[3]: `AGENTTY_MCP_CONFIG` (path; default lookup `~/.agentty` then `./.agentty`), `AGENTTY_MCP_ALLOW_PROJECT` (project mcp.json trust, off by default), `AGENTTY_MCP_TIMEOUT_MS`, `AGENTTY_MCP_CONNECT_TIMEOUT_MS`, `AGENTTY_MCP_TOOL_BUDGET` (soft cap, default 100), `AGENTTY_ACP_AGENTS`, `AGENTTY_ACP_ALLOW_PROJECT`.

Retrieval/Smart Mode knobs: a large `AGENTTY_RAG_*` family (fusion, BM25/dense weights, MMR, GraphRAG, docs dir via `AGENTTY_DOCS_DIR`) and `AGENTTY_SMART_*` family (router thresholds, floors, orchestration toggles)[1][3].

## 4. MCP

stdio MCP servers in user or project `mcp.json`; project stdio entries need exact-byte approval stored in user `state/` (a repo cannot approve itself); `AGENTTY_MCP_ALLOW_PROJECT` enables blanket trust[2]. Skills drop into `.agentty/skills/` or `~/.agentty/skills/` as `SKILL.md` (Claude Code format compatible); skills that exec or reach the network declare `effects: [exec, net]` frontmatter and install via `agentty skill add`, which pins approval to content[1].

## 5. Headless, ACP, air-gap

```bash
agentty run "fix the failing test" --json --events jsonl
agentty run --read-only --no-network "analyze untrusted code"
agentty acp            # Agent Client Protocol server for Zed
```

`agentty run` prints just the answer; `--timeout`, clear exit codes, and JSONL event streams are documented[1]. Air-gap mode relays traffic over SSH (`agentty airgap --setup user@host`, SOCKS5-over-SSH, TLS pinned)[1].

## 6. Not verified upstream

- The `settings.json` key schema (docs state the pickers own it and publish no key list).
- Hooks schema beyond the file name (`hooks.json`); the docs Hooks page was not fetched.
- MCP transport support beyond stdio in `mcp.json` (HTTP/SSE entries not confirmed).
- Version convention (no `--version` flag documented; `agentty run` exit-code semantics summarized as "clear exit codes" without a table).

## Sources

- [1] https://raw.githubusercontent.com/1ay1/agentty/master/README.md: binary facts, install matrix, providers, skills, run/airgap/ACP, RAG and Smart Mode overview
- [2] https://agentty.org/docs/configuration/: on-disk paths, AGENTTY_HOME, guidance-file tiers, credentials storage, MCP trust model, workspace flag
- [3] https://agentty.org/docs/environment/: full environment-variable listing by group
- [4] https://github.com/1ay1/agentty/releases/tag/v0.9.19: latest release at compile time
- [5] https://github.com/Qumulo/llm-agents/tree/main/packages/agentty: nix index entry (v0.9.11 pin, owner 1ay1)
