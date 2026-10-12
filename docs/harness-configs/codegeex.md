# CodeGeeX (Zhipu AI) — Configuration Reference

**Compiled:** 2026-10-12 against the VS Code Marketplace listing (publisher Zhipu AI, ~1.37M installs), codegeex.cn / codegeex.ai site metadata, and the THUDM CodeGeeX model repositories on GitHub. CodeGeeX is Zhipu AI's in-editor coding assistant: code completion (accept with Tab), comment-to-code generation, auto comments in Chinese or English, unit-test generation, an "Ask CodeGeeX" sidebar with `/explain`, `/comment`, `/fixbug`, `/tests`, inline chat (Cmd+i / Ctrl+i), `@Repo` RAG over open-source repos, and a code-interpreter toolbox for file, data, chart, and image processing[1]. It runs as a plugin in VS Code, JetBrains IDEs, HBuilderX, and DeepIn-IDE, across 100+ languages[1][2]. The model lineage is open source (THUDM CodeGeeX through CodeGeeX4, a 13B-parameter multilingual code model line); the plugin service is Zhipu-hosted[1][3].

This is a completion-and-chat assistant, not an agent harness: no agentic file-editing loop, no MCP surface, and no documented local config files were found. Zhipu's agent products (ZCode, Claude Code endpoints on bigmodel.cn) are separate surfaces; ZCode has its own entry at [zcode.md](zcode.md).

---

## 1. Install, executable, version

Install is per-IDE through the host's extension mechanism: VS Code Extensions pane search "CodeGeeX" (publisher Zhipu AI), the JetBrains Marketplace plugin, or the equivalent for HBuilderX and DeepIn-IDE[1][2]. There is no standalone executable and no CLI. No version-probe command is documented; the extension's version lives in the host IDE's extension list.

## 2. Config files

None that CodeGeeX owns. All settings are contributed through the host IDE's settings system, so on VS Code they live in `settings.json` (user or workspace scope) under the extension's contributed keys, and on JetBrains in the IDE's settings. The official settings reference did not render on any fetched page: the marketplace listing carries no settings table, and codegeex.ai is a JavaScript-rendered page whose fetch returned only metadata[1][5]. Third-party guides describe a setting that triggers completion only on Alt/Option + `\` and cite a VS Code 1.68.0+ requirement[4], but no official page publishes the key list, so a config-writing adapter has no verified key surface to target.

## 3. Environment variables, MCP, BYO endpoint

- Environment variables: none documented on fetched pages.
- MCP: not mentioned anywhere on the fetched listing, site, or model repos; treat as absent until Zhipu says otherwise.
- Custom endpoints: the plugin is a hosted Zhipu service; no BYO-gateway option was found on fetched pages. Unverified either way.

## 4. Wrapper relevance

Realistic adapter surface: none beyond install detection (the extension present in a given IDE profile). Isolation follows the host IDE (`code --user-data-dir`, JetBrains config directories). Anything more would need a published settings reference, which does not exist on the reachable pages as of this compile.

## 5. Not verified upstream

- The contributed settings keys and their defaults (no official list rendered; VS Code shows them in its Settings editor once installed).
- Account requirements: free use, login, or plan gating (marketplace text does not state it; Chinese guides describe Zhipu-account login, but no official page fetched confirms it).
- The VS Code 1.68.0+ requirement and the Alt/Option + `\` trigger setting (third-party FAQ claims only).
- Whether any 2026 release added agent or MCP features (searches surfaced nothing; Zhipu's agent attention appears to have gone to ZCode and its Claude Code integration).

## Sources

- [1] https://marketplace.visualstudio.com/items?itemName=aminer.codegeex: publisher, features, IDE and language support, THUDM repo links
- [2] https://codegeex.cn: official site (Chinese edition)
- [3] https://github.com/THUDM/CodeGeeX4: open-source model lineage
- [4] Third-party setup guides surfaced by search (CSDN comparison article Feb 2025; Zhihu install guide Jun 2026): extension-install flow, trigger setting, VS Code version claim
- [5] https://codegeex.ai: international site (JavaScript-rendered; only metadata retrievable at compile time)
