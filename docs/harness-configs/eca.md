# ECA (Editor Code Assistant) — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/editor-code-assistant/eca](https://github.com/editor-code-assistant/eca) (Clojure server, latest release 0.163.1 of 2026-10-09, pushed 2026-10-09) and the eca.dev configuration introduction[1][2]. ECA is an editor-agnostic AI pair-programming server: editors (Emacs, VS Code, Neovim, IntelliJ, plus an ECA Desktop app) spawn `eca server` and speak an LSP-like protocol over stdin/stdout. One configuration model covers every editor, which is the project's central pitch[1].

---

## 1. Install, executable, version

Editor plugins download and start the server automatically; the install page covers plugins, the Desktop app, and manual installs (script, Homebrew, mise, GitHub Releases)[1]. The nix index wraps the platform-native zips with a JAR fallback for unsupported platforms[3]. Version probe: releases are tagged (0.163.1 at compile time); no dedicated CLI version flag is documented in the README.

## 2. Config files

| Layer | Path / mechanism | Notes | Source |
|---|---|---|---|
| Global | `~/.config/eca/config.json` | User-wide config | [2] |
| Local | `.eca/config.json` | Per workspace root | [2] |
| Editor | `initializationOptions` on the initialize request | Set by the client editor; uncommon | [2] |
| Env | `ECA_CONFIG='{"myConfig": "my_value"}'` | Inline JSON at server spawn | [2] |
| Extra files | `extraConfigs` array in the global file | e.g. `~/.config/eca/config.local.json`, `.eca/config.local.json`; relative paths resolve against the workspace root; missing files are logged and skipped | [2] |

Merge order, lowest to highest: built-in defaults, initializationOptions, `ECA_CONFIG` env, global file, local file, then `extraConfigs` in listed order. Deep merge: nested maps merge recursively, scalars and arrays from a higher source replace[2]. `--config-file` replaces the global/local lookup entirely[2].

The in-chat `/login` flow writes provider auth into the global config.json (choose provider, follow key/auth steps)[1].

## 3. Providers

Documented logins: OpenAI, Anthropic, GitHub Copilot (called out for free models), Ollama local models, "and many more", plus custom providers on the models documentation page[1]. Model config lands in global config.json via `/login` or manual editing; the schema lives at eca.dev/config/models (linked, page not fetched here).

## 4. MCP, agents, context

- MCP resources and prompts are supported as context sources[1] (the OpenTelemetry line and feature bullets); the MCP config key schema was not fetched.
- Agents/subagents: multiple agents configurable with different models, tools, and behaviors[1].
- OpenTelemetry export for tools, prompts, and server usage[1].

## 5. Not verified upstream

- The config.json key schema beyond `defaultAgent` examples (the full reference lives on eca.dev subpages not fetched here).
- MCP configuration keys and file shape.
- Any ECA-specific environment variables beyond `ECA_CONFIG`.
- A config-dir relocation variable (none appears on the introduction page; global path looks fixed to `~/.config/eca`, which XDG_CONFIG_HOME would move wholesale).

## Sources

- [1] https://raw.githubusercontent.com/editor-code-assistant/eca/master/README.md: protocol model, editor plugins, /login writing global config.json, features (agents, MCP context, multi-provider)
- [2] https://eca.dev/config/introduction: config layer table, ECA_CONFIG env var, extraConfigs, merge order, --config-file
- [3] https://github.com/Qumulo/llm-agents/tree/main/packages/eca: nix index entry (native zips + JAR fallback, releases channel)
- [4] https://github.com/editor-code-assistant/eca/releases/tag/0.163.1: latest release at compile time
