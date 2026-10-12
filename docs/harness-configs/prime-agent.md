# Prime Agent — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/PrimeIntellect-ai/prime-agent](https://github.com/PrimeIntellect-ai/prime-agent) (MIT, Rust + Python), the README, docs/decision-api.md, and the config source files in crates[1][2][3][4]. Prime Agent is Prime Intellect's self-improving RLM harness: a persistent Python REPL is the model's built-in tool, subagents spawn as real child agents (`rlm.spawn(...)`), and a Continual Harness stores prompts, memories, and skill descriptions as durable state that `/refine` can update in small, evidence-backed steps (local to the session by default, snapshots rollback-able). Latest release v0.10.0 (nix pins v0.9.8); pushed 2026-10-12: alive.

---

## 1. Install, executable, version

```bash
curl -fsSL https://app.primeintellect.ai/prime-agent/install.sh | sh
# Windows: irm https://app.primeintellect.ai/prime-agent/install.ps1 | iex
prime-agent                    # run inside the project directory
```

First launch: `/login` picks a subscription or API-key provider. Useful commands[1]:

```bash
prime-agent agents             # browse running, idle, and saved sessions
prime-agent attach <agent>     # reattach to a running session
prime-agent --resume [path|id]
prime-agent status             # background service state
prime-agent doctor [--fix]     # inspect or repair services
prime-agent update [--force]
prime-agent shutdown [--force]
prime-agent schedule           # scheduled session re-entry
```

## 2. Config files and paths

From the config constants[3][4]:

| Setting | Value |
|---|---|
| Config dir name | `.prime/agent` (i.e. `~/.prime/agent`) |
| Dir override | `PRIME_AGENT_CODING_AGENT_DIR` |
| Session dir override | `PRIME_AGENT_SESSION_DIR` (legacy alias `PRIME_AGENT_CODING_AGENT_SESSION_DIR`) |
| Daemon socket override | `PRIME_AGENT_DAEMON_SOCKET` or `--daemon-socket` flag |

Settings live in `settings.json` inside that dir: docs/decision-api.md shows the canonical example `{"decisionApi": {"systemOneModel": "prime-inference/cloudflare/clef"}}` for the experimental Decision API, resolved through the same model registry as `imageModel`[2].

## 3. Runtime model

Daemon-backed sessions survive terminal disconnects and reattach; heartbeats (`/heartbeat`, `rlm_heartbeat`) and `/goal` keep long tasks moving; `/autonomous` runs within configured turn, token, and time budgets with optional quality gates. Agent-to-agent messaging lets running agents orchestrate one another without routing through the user[1]. Skills are importable Python packages, with a built-in skill creator for recurring workflows[1].

A security note the README states plainly: Prime Agent executes model-generated Python and project commands with user permissions; its worker/kernel process isolation improves lifecycle handling but is not a security sandbox[1].

## 4. MCP

Not covered in the README or the fetched docs; the nix packaging pulls the MCP Python SDK as a dependency[5], but no MCP configuration surface is documented upstream.

## 5. Not verified upstream

- The settings.json schema beyond `decisionApi.systemOneModel` (and `imageModel` referenced alongside it).
- MCP configuration keys (SDK is vendored; surface unpublished).
- Whether `PRIME_AGENT_CODING_AGENT_DIR` moves auth storage as well as settings.
- Exit codes; a headless/one-shot mode (sessions are daemon-attached; no `-p` equivalent is documented).

## Sources

- [1] https://raw.githubusercontent.com/PrimeIntellect-ai/prime-agent/main/README.md: install, /login, command set, RLM/Continual Harness model, autonomy features, sandbox warning
- [2] https://raw.githubusercontent.com/PrimeIntellect-ai/prime-agent/main/docs/decision-api.md: settings.json decisionApi key, model-registry resolution
- [3] https://raw.githubusercontent.com/PrimeIntellect-ai/prime-agent/main/crates/pa-cli/src/config.rs: CONFIG_DIR_NAME ".prime/agent", PRIME_AGENT_* env var names
- [4] https://raw.githubusercontent.com/PrimeIntellect-ai/prime-agent/main/crates/pa-daemon/src/paths.rs: agent/session dir env overrides, daemon socket resolution
- [5] https://github.com/Qumulo/llm-agents/tree/main/packages/prime-agent: nix index entry (v0.9.8 pin, MCP SDK dependency)
