# Ori (OpenRouter) — Configuration Reference

**Compiled:** 2026-10-12 against the OpenRouterLabs ori-releases README, the openrouter.ai/ori product page, and release metadata[1][2][3]. Ori is OpenRouter's CLI. Two sides matter here: it wraps harnesses you already have (`ori claude`, `ori codex`, `ori hermes` launch those CLIs with OpenRouter's model selection wired in, "no extra auth"), and it includes Ori Code, OpenRouter's own terminal coding agent, plus `ori eval` for model recommendations. Distribution is binary-only: the source lives in a private repository, and OpenRouterLabs/ori-releases carries compiled executables published by that repo's release workflow[1]. Ori Intern is in private testing; Desktop is announced[2]. Release repo pushed 2026-10-12: alive.

---

## 1. Install, executable, version

```sh
curl -fsSL https://openrouter.ai/labs/ori/install.sh | bash   # verifies SHA256SUMS, installs to ~/.local/bin
ori update                    # move to the newest release
ori update --alpha            # single alpha build; --stable returns
```

`ORI_INSTALL_DIR` relocates the install target[1]. Releases are immutable, tagged `cli-<version>-<commit>`, and never rewritten; each carries per-platform `ori` executables (darwin/linux x64+arm64, musl variants), `install.sh`, `SHA256SUMS`, `version`, LICENSE and NOTICE. Assets are not signed; the checksums are the verification mechanism. The alpha channel is opt-in per invocation via `ORI_CHANNEL=alpha` on install, never a persisted subscription[1]. The nix index resolves versions from the GitHub latest-release API and normalizes the `+`-spelled tag for `ori --version` comparison[4].

## 2. Config files, env vars

Beyond install/update, the published pages document no config file, no `ORI_*` runtime variables, and no per-harness settings; the landing page's pitch is that auth needs no extra setup because OpenRouter account auth flows through the wrapped harness[2]. What is documented:

| Var | Purpose | Source |
|---|---|---|
| `ORI_INSTALL_DIR` | Install directory for install.sh | [1] |
| `ORI_CHANNEL=alpha` | One-shot alpha channel on install | [1] |

## 3. What the harness-wrapping means for this catalog

`ori claude` / `ori codex` / `ori hermes` are launcher verbs over the claude-code, codex-cli, and hermes-agent harnesses: model selection moves to OpenRouter while those tools keep their own config trees. An adapter that manages those harnesses should expect a second writer (Ori) choosing models through its own account auth. Which env or config Ori injects into the child process is not published; that is the main open question for wrappers.

## 4. Not verified upstream

- The Ori source repository (private; only release artifacts are public).
- Any config file, runtime env var, or per-harness injection mechanism (undocumented on the fetched pages).
- Ori Code's own configuration surface (presented as a native agent on the landing page; no docs fetched).
- `ori --version` output shape beyond the nix index's handling of `+`-spelled versions.

## Sources

- [1] https://raw.githubusercontent.com/OpenRouterLabs/ori-releases/main/README.md: install, ORI_INSTALL_DIR/ORI_CHANNEL, ori update --alpha/--stable, release contents and immutability, checksums-not-signatures
- [2] https://openrouter.ai/ori: product surface (ori claude/codex/hermes, Ori Code, ori eval), no-extra-auth claim, Ori Intern/Desktop status
- [3] https://github.com/OpenRouterLabs/ori-releases: release repo activity (pushed 2026-10-12)
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/ori: nix index entry (release-API version resolution, binary distribution)
