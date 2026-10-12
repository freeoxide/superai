# Claw Code (`claw`) — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/ultraworkers/claw-code](https://github.com/ultraworkers/claw-code) (MIT, Rust), its USAGE.md, and the Qumulo llm-agents nix index entry `packages/claw-code`[1][2][4]. Claw Code is a public Rust reimplementation of a Claude Code-shaped CLI (binary `claw`) that its own README describes as an "agent-managed museum exhibit" rather than a supported product: the repo is maintained by other harnesses (LazyCodex, Gajae-Code), and the README steers serious users to those[1]. Tracked here because it is real, installable, and has a documented config surface. Repo activity: pushed 2026-10-11.

---

## 1. Install, executable, version

There is no supported binary release. Build from source only[1]:

```bash
git clone https://github.com/ultraworkers/claw-code
cd claw-code/rust && cargo build --workspace
./target/debug/claw --help
```

Two naming traps the README flags[1]:

- `cargo install claw-code` installs a deprecated crates.io stub that only prints "claw-code has been renamed to agent-code". Do not use it.
- `cargo install agent-code` is a different upstream binary and installs `agent`, not `claw`.

The nix index pins an unstable snapshot (2026-08-16) rather than a release[4]. No version-probe convention is documented beyond `claw --help`.

## 2. Config files

Settings are JSON, loaded in this order, later files overriding earlier ones[2]:

1. `~/.claw.json`
2. `~/.config/claw/settings.json`
3. `<repo>/.claw.json`
4. `<repo>/.claw/settings.json`
5. `<repo>/.claw/settings.local.json`

`claw --output-format json config` reports each file's precedence rank, winning keys, and shadowed keys, so an adapter can verify which file owns a key without reimplementing the merge[2]. Model aliases can be added in any settings file[2]. Per-workspace state lands in `.claw/worker-state.json` (worker id, session reference, model, permission mode)[2]. Subagents scaffold to `.claw/agents/<name>.toml`[2]. `CODEX_HOME` is read as an optional root for user-level skill and command lookups (sessions and Codex config are explicitly not supported)[2], which overlaps the codex-cli harness's home.

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `CLAW_CONFIG_HOME` | Relocates the config home (README's smoke-test example sets it to a temp dir) | [1] |
| `CLAW_OUTPUT_FORMAT` | Default non-interactive output format (`text`/`json`); `--output-format` wins | [2] |
| `CLAW_LOG` | Log level selector surfaced by help and doctor | [2] |
| `CLAW_MODEL`, `ANTHROPIC_MODEL`, `ANTHROPIC_DEFAULT_MODEL` | Environment model slots, checked in that order | [2] |
| `ANTHROPIC_API_KEY` vs `ANTHROPIC_AUTH_TOKEN` | Not interchangeable: `sk-ant-*` keys must go in `ANTHROPIC_API_KEY` (x-api header); the wrong slot is a documented 401 cause | [2] |
| `OPENAI_API_KEY` + `OPENAI_BASE_URL`, `XAI_API_KEY`, `DASHSCOPE_API_KEY`, `OLLAMA_HOST` | Other provider backends; model-name prefixes (`openai/`, `local/`, `qwen/`, `kimi/`) select the backend | [2] |

Auth is API-key only; a Claude subscription login is not a supported path[1]. Proxy: standard `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY` are honored[2].

## 4. MCP

`mcpServers` entries live in the settings JSON files (for example `.claw.json`)[2]. `claw mcp --output-format json` validates entries and keeps loading the valid subset while reporting which field of each malformed entry was rejected; `status` and `doctor` expose the same validation data[2].

## 5. Headless

```bash
claw prompt "say hello"                 # one-shot prompt
claw --permission-mode read-only prompt "summarize Cargo.toml"
claw --output-format json status        # machine-readable envelope
```

Permission modes: `read-only`, `workspace-write` (default), `danger-full-access` (explicit opt-in via flag, `--dangerously-skip-permissions`, env, or config)[2]. `claw doctor` is the documented health check[1].

## 6. Not verified upstream

- ACP support: `claw acp` is currently a status-only alias (exit 0, no daemon); real ACP remains on the roadmap[1].
- Whether `CLAW_CONFIG_HOME` moves every settings layer or only the `~/.config/claw` one.
- The full settings JSON schema (USAGE.md documents the resolution chain and named keys, not a complete key list).
- Release channel: none exists; the nix pin is a dated snapshot.

## Sources

- [1] https://raw.githubusercontent.com/ultraworkers/claw-code/main/README.md: project status, build-from-source, crates.io stub trap, CLAW_CONFIG_HOME smoke example, doctor, ACP status
- [2] https://raw.githubusercontent.com/ultraworkers/claw-code/main/USAGE.md: settings resolution chain, env vars, MCP validation, permission modes, hooks, agents
- [3] https://github.com/ultraworkers/claw-code: repo activity (pushed 2026-10-11)
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/claw-code: nix index entry (unstable snapshot pin, owner ultraworkers)
