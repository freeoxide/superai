# Subplan 13 — verification, quality gates, and non-UI release

Parent: [master plan](master-plan.md)  
Task prefix: QAL  

All fifteen QAL items are closed. QAL-03/07/08/10/11/12 were verified at baseline, and areas 2, 8,
and 9b closed the rest, judge-verified round 2. The QAL-09 platform-state tests are cfg-gated and
execute on the windows/macos CI runners, not on this linux host.

SUPERAI_TEST_KEEP test isolation, 48/48 fixture corpora, executor and span-codec fuzz targets plus
the adapter-detection fuzz family, a cargo-mutants CI job, coverage_ledger.rs ledgers,
docs/dependency-review.md, README sections, and platform tests in transaction.rs.

## Exit gate

- [x] Tests cover every supported adapter/platform state.
      > Closed (area 9b): dedicated platform-adversarial tests added in superai-config
      > transaction.rs — cfg(windows) `windows_locked_target_commit_is_typed_error_
      > never_corrupting` (target held open without FILE_SHARE_DELETE through the
      > mutation boundary: typed `ConfigError::Io`, contents never corrupted, no leaked
      > staged temp), `windows_reserved_device_paths_rejected_live` (CON/PRN/AUX/NUL/
      > COM1/LPT1 write targets refused before any disk work, backed by the
      > host-independent `windows_reserved_device_names_are_refused_as_write_targets`
      > helper table wired into path validation), and `windows_long_path_commit_is_
      > verified_or_typed_never_partial` (>MAX_PATH depth: verified read-back or typed
      > refusal, never a partial file); cfg(target_os="macos")
      > `macos_case_insensitive_collision_write_is_typed_never_corrupting` (real
      > case-insensitive APFS volume: case-variant creation refused, exact-name
      > overwrite still verified) and `macos_application_support_paths_commit_
      > through_the_boundary` (`~/Library/Application Support/...` shaped writes).
      > Cross-clippy x86_64-pc-windows-msvc and aarch64-apple-darwin PASS for
      > superai-config (the platform-gated code lives there); execution is by the
      > windows/macos CI runners — reasoned-not-executed on this linux host.
- [x] Fault/crash recovery evidence exists.
- [x] Fuzzing and mutation testing guard parsers/safety branches.
- [x] Secret/path/process/network abuse suites pass.
- [x] Dependency audit and lockfile discipline pass.
- [x] Goal/harness/provider/template ledgers are complete.
- [x] Non-UI release claims match observable behavior.

## Deferred items

> Completion record (run 3): QAL-03/07/08/10/11/12 verified at baseline; QAL-01
> (SUPERAI_TEST_KEEP retain-on-failure), QAL-02 (48/48 fixture corpora; the
> layout.default/isolated fixture kind covers the 3 flagship relocated-root
> surfaces — claude_code, codex_cli, gemini_cli — machine-checked against the
> adapters' own constants; 20 RelocatedRoot surfaces exist in the catalog, the
> other 17 carry the standard minimal/populated/foreign/malformed kinds),
> QAL-04 (executor + span-codec fuzz targets over the new engine surfaces plus
> the adapter version/schema detection family — version-output fixtures,
> PATH-shaped strings, and catalog entries driven through the real detect path
> with injected PATH/home, which found and fixed a byte-vs-char bound bug in
> version extraction; coverage-guided cargo-fuzz stays a documented local
> recipe per the pinned stable-toolchain/dependency rules in fuzz.rs), QAL-05
> (cargo-mutants 27.1.0 installed and run: 610-mutant scoped baseline at 50.2%
> caught, .cargo/mutants.toml + CI continue-on-error job with artifact;
> diff-scoped/sharded runs are recorded follow-up), QAL-06 (real-path fault
> injection — area 2), QAL-13/14 (coverage_ledger.rs: §9/§10 machine-verified
> ledgers + research-file coverage + freshness window), QAL-15 (README
> backup/recovery, data locations, security posture, template-repo sections)
> closed by area 8, judge-verified round 2. docs/dependency-review.md carries
> the per-crate dependency evidence (QAL-12). QAL-09's platform-state families
> (macOS application paths, macOS case-insensitive collisions, Windows
> locked-file, reserved-device-name, and long-path adversarial cases) were
> closed by area 9b with cfg-gated tests executed by the platform CI runners.
