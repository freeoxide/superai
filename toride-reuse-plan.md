# toride reuse plan

This report compares superai's crates against toride's to decide which components superai could replace with a toride piece, where it should copy a design instead, and where its own code is the better fit. The repos are /home/axent/fo/superai and /home/axent/fo/toride. This section covers the filesystem layer only: every file in crates/superai-config/src named in the table below was read in full against crates/toride-fs/src (all seven files) and a skim of crates/toride-backup/src. Written 2026-10-08. Publish status was checked the same day: https://crates.io/api/v1/crates/toride-fs returns 200 with max_version 0.1.0, created 2026-10-08, so toride-fs exists on crates.io and an ADOPT verdict would not be blocked by superai's published-deps rule. It went up today with a single 0.x release, which is worth remembering before depending on it. Later sections cover process, install, and update areas.

## Filesystem layer

| superai component | toride counterpart | verdict | reason |
|---|---|---|---|
| atomic.rs atomic write | toride-fs/src/atomic.rs | KEEP | superai adds digest gating, fd-based chmod, read-back verify, Windows rename retry |
| locking (superai has none) | toride-fs/src/lock.rs | KEEP | fd lock on the replaceable inode breaks across rename; see prose |
| permission audit (superai has none) | toride-fs/src/permissions.rs | PORT | two small unix checks superai lacks; secret files keep their loose modes today |
| safe_paths.rs path rules | toride-fs/src/expand.rs | KEEP | refuse-don't-expand is the safer rule for harness config paths |
| backup.rs sibling backups + catalog | toride-backup (restic/borg) | KEEP | wrong shape: repository snapshots vs inline sibling copies |
| journal.rs crash recovery | none in toride | KEEP | no counterpart exists |
| snapshot.rs conflict tokens | none in toride | KEEP | no counterpart exists |
| transaction.rs multi-file tx, copy_tree | none in toride | KEEP | no counterpart exists |
| quarantine.rs recoverable delete | none in toride | KEEP | no counterpart exists |
| read paths | toride-fs/src/read.rs | KEEP | a NotFound-to-None helper adds nothing over superai's typed errors |

### Atomic write

Both sides do the same core: create a temp in the target's directory, write, fsync the temp, rename, fsync the parent. toride does this in `atomic_write_bytes_with_mode` (toride-fs/src/atomic.rs:177-219), creating the temp with the final mode up front so the inode is never visible with looser bits (toride-fs/src/atomic.rs:186-189). superai does it in `atomic_write_expecting` (crates/superai-config/src/atomic.rs:234-366) with an O_EXCL temp whose handle stays open until the bytes are durable, so a pre-planted symlink at the temp name cannot redirect the write (crates/superai-config/src/atomic.rs:258-277).

The differences are in what happens around that core, and they all favor superai:

Durability of the parent fsync. toride treats the directory sync as optional and swallows every error with a trace (toride-fs/src/atomic.rs:80-104). superai surfaces parent-sync failures and tolerates only the error kinds Windows actually produces: Unsupported, PermissionDenied, NotFound (crates/superai-config/src/atomic.rs:147-165). Silence here means a crash can lose the rename with no diagnostic, which sits badly with superai's no-swallowed-Results rule.

Mode preservation. superai derives the replacement's mode from the target when it exists, lands 0o600 for new files, and applies it through the open fd so a name swap cannot redirect the chmod onto a foreign file (crates/superai-config/src/atomic.rs:67-99). toride always lands 0o600 for the plain entry points (toride-fs/src/atomic.rs:32, toride-fs/src/atomic.rs:69-73). A harness config at 0644 that some service reads as another user would silently tighten to 0600 on every edit. Worse, the `atomic_write_with_perms` safety net chmods by path after the rename, and on chmod failure it removes the just-replaced file (toride-fs/src/atomic.rs:133-137). That is a race window where the chmod can land on a swapped-in foreign file, and a failure mode that deletes the config instead of leaving it with a wrong mode. superai removes only its own temp when mode application fails (crates/superai-config/src/atomic.rs:308-311).

Concurrent-modification gating. superai checks a required prior state before creating the temp and rechecks the digest immediately before the rename, aborting untouched on any foreign change (crates/superai-config/src/atomic.rs:190-226 and crates/superai-config/src/atomic.rs:316-327). toride has nothing comparable; its write is last-writer-wins and will clobber a concurrent edit without a signal.

Verification and platform edges. superai reads the file back after the rename and compares bytes (crates/superai-config/src/atomic.rs:354-363), clears the Windows readonly attribute before replacing, and retries a denied rename with growing backoff (crates/superai-config/src/atomic.rs:330-349). toride has neither step; a readonly target on Windows fails the persist outright.

Verdict: KEEP. Nothing in toride-fs/src/atomic.rs is ahead, and the chmod-by-path rollback is the one piece I would not copy under any circumstances.

### Locking

toride-fs offers `with_lock` and `with_lock_path`: open the file, take an fd-lock write guard, run a closure (toride-fs/src/lock.rs:30-51 and toride-fs/src/lock.rs:63-84). superai has no fd-lock dependency anywhere (verified by grep across crates/).

Does superai need one? Its digest gating already catches concurrent writers, including foreign editors that would never take a superai lock; the failure is a typed ConcurrentModification instead of a lost write. A lock would only serialize two superai processes, and toride's implementation would do it badly for this use. The lock is taken on the config file itself, and fd locks bind to the inode. superai replaces files by rename, so the moment the rename lands, the locked inode no longer carries the path and the next process to lock the path locks the new inode. Mutual exclusion silently ends exactly when it matters. The open also creates the file when missing and follows symlinks (toride-fs/src/lock.rs:34-39), so locking a config that does not exist materializes an empty file.

Verdict: KEEP. If process serialization ever becomes a requirement, port the concept with a separate never-renamed .lock file, not this closure over the target itself.

### Permission audits

This is the one genuine gap. toride-fs has `check_not_world_writable` (refuses mode bit 0o002, toride-fs/src/permissions.rs:49-59) and `check_owner_is_root` (toride-fs/src/permissions.rs:70-73), both unix-only. superai has no mode audit; `check_ownership` in executor.rs governs config keys, not file bits (crates/superai-config/src/executor.rs:155-160). superai preserves a target's existing mode across its atomic replace (crates/superai-config/src/atomic.rs:73-81), which is the right default, but the flip side is that a world-readable `.env` full of API keys stays world-readable through every superai edit and nothing ever says so. The snapshot already records permission bits (crates/superai-config/src/snapshot.rs:58-59), so the audit has its data.

Verdict: PORT. Reimplement the two checks, roughly fifteen lines, unix-gated, and warn (or refuse, for `.env`-shaped files) when a secret-bearing config is readable by others. Taking the dependency for this would add tempfile, fd-lock, dirs, and tracing (toride-fs Cargo.toml dependencies) for two boolean functions.

### Path expansion vs safe_paths

The two crates answer the same question oppositely. toride expands: `~` to home, `$HOME` interpolated from the environment (toride-fs/src/expand.rs:29-46 and toride-fs/src/expand.rs:63-75). superai refuses: any `$`, `%`, or `~` component in a path is an unresolved variable and a typed rejection (crates/superai-config/src/safe_paths.rs:20-26), globs likewise (crates/superai-config/src/safe_paths.rs:15-18), and broad roots such as `/etc` or a bare drive are refused outright (crates/superai-config/src/safe_paths.rs:28-46, with the Windows shapes at crates/superai-config/src/transaction.rs:246-287 and reserved device names at crates/superai-config/src/transaction.rs:291-315).

For a tool that writes into other programs' config directories, refusal is the safer rule. Interpolating `$HOME` from the environment moves an attacker-controllable variable into path resolution, and toride's expander has two quiet failure modes superai's refusal avoids: `~someuser` is returned unresolved and flows downstream as a literal directory name (toride-fs/src/expand.rs:40-43), and an unresolvable home falls back to `.` (toride-fs/src/expand.rs:84-86), producing a relative path where superai's `home_dir` returns None and callers refuse (crates/superai-config/src/safe_paths.rs:3-13, with the refusal on use at crates/superai-config/src/quarantine.rs:12-22).

Verdict: KEEP.

### Backup and catalog

superai's backup.rs takes a digest-verified sibling copy beside the file before a mutation, records mode and size in a catalog entry, and restores through the atomic-write discipline with identity checks so a backup of one file can never restore over another (crates/superai-config/src/backup.rs:120-136 for the exclusive create, crates/superai-config/src/backup.rs:404-418 for restore, crates/superai-config/src/backup.rs:522-552 for the relation gate, crates/superai-config/src/backup.rs:668-742 for the verified restore with a redacted diff preview). toride-backup is a different animal: a restic and borg client that shells out to those binaries, dispatching `restic restore` or `borg extract` through a command runner (crates/toride-backup/src/lib.rs:1-7, crates/toride-backup/src/restore.rs:13-19, restore options at crates/toride-backup/src/restore.rs:42-56), plus a scheduler that installs systemd timer units and cron entries under /etc (crates/toride-backup/src/schedule.rs:1-3).

None of that maps onto superai's need. superai wants an inline, synchronous, no-external-binary copy made milliseconds before a config edit, sized for single files, with no root and no daemon. toride-backup assumes repositories, snapshot ids, and host-level scheduling. Even its best detail, carrying passphrases in environment variables with redaction (crates/toride-backup/src/restore.rs:22-26), answers a problem superai does not have.

Verdict: KEEP. toride-backup is not relevant to config-file backup, by shape rather than by quality.

### Components with no toride counterpart

journal.rs writes a phase-tracked crash journal containing paths and backup ids but never contents (crates/superai-config/src/journal.rs:64-85) and recovers at startup by sweeping only its own temp pattern, restoring from recorded backups, and quarantining corrupt journals aside rather than aborting the scan (crates/superai-config/src/journal.rs:179-199, temp sweep at crates/superai-config/src/journal.rs:322-368). snapshot.rs builds conflict tokens from digest, size, and symlink target, deliberately ignoring hint fields for decisions (crates/superai-config/src/snapshot.rs:51-80, comparison at crates/superai-config/src/snapshot.rs:192-209), plus a capped symlink-loop walk (crates/superai-config/src/snapshot.rs:213-265). transaction.rs validates removal targets against traversal, globs, and broad roots (crates/superai-config/src/transaction.rs:123-177), stages temps before commit (crates/superai-config/src/transaction.rs:426-496), handles cross-device renames with a copy fallback (crates/superai-config/src/transaction.rs:551-559), refuses case-fold collisions (crates/superai-config/src/transaction.rs:592-612), backs up foreign files before the first commit (crates/superai-config/src/transaction.rs:1532-1590), and journals each step as about-to-commit before mutating (crates/superai-config/src/transaction.rs:1688-1699). quarantine.rs moves deletion targets into an owner-only 0o700 directory, verifies digests, and restores across filesystems (crates/superai-config/src/quarantine.rs:289-299, crates/superai-config/src/quarantine.rs:336-360, crates/superai-config/src/quarantine.rs:471-520).

toride-fs has nothing in any of these areas, so there is nothing to adopt or port. The read helpers (toride-fs/src/read.rs:21-36 and toride-fs/src/read.rs:44-59) are NotFound-to-None wrappers; superai's reads go through its own typed error path and gain nothing from them.

### Security notes

The security-relevant findings from this comparison: toride's post-rename chmod-by-path with delete-on-failure (toride-fs/src/atomic.rs:133-137) is both a TOCTOU exposure and a data-loss path. toride's lock file open creates missing files and follows symlinks (toride-fs/src/lock.rs:34-39). toride's env interpolation moves environment control into path resolution (toride-fs/src/expand.rs:68-71). On superai's side, the one gap this comparison surfaced is the missing world-readable audit on secret-bearing configs, covered under permission audits above.

## Process execution

This section reads crates/superai-core/src/process.rs in full against crates/toride-runner/src: runner.rs, spec.rs, policy.rs, duct_runner.rs, discovery.rs, fake.rs, redact.rs, output.rs, and output_mode.rs. The async files (async_runner.rs, tokio_runner.rs, streaming.rs) sit behind non-default tokio features and are weighed under adoption readiness. Publish status checked 2026-10-08: https://crates.io/api/v1/crates/toride-runner returns 200 with max_version 0.1.0, created 2026-10-08, 26 downloads. The four semantics conflicts the 2026-10-01 gap plan recorded (toride-apps-gap-plan.md:115) are now per-spec policy knobs in policy.rs, so those blockers are gone. What still argues against adoption is below.

| superai component | toride counterpart | verdict | reason |
|---|---|---|---|
| env composition, compose_child_env | spec.rs env fields + duct_runner.rs apply_env_policy | KEEP | remove-wins is now a knob but per-spec; superai's one composed map has no wrapper-order dependence |
| PATH resolution, resolve_executable | policy.rs PathResolution::ChildEnvNoCwd | KEEP | ChildEnvNoCwd matches superai rule for rule; it is the opt-in, not the default |
| output cap checked after child exit | duct_runner.rs run_duct_command_limited | PORT | in-capture limiting bounds memory under a flooding child; superai's cap refuses after buffering everything |
| redaction, redact_args + scrub_stderr | redact.rs + display.rs value scrubbing | PORT | toride scrubs secret values from both streams; superai blanks the whole stderr field and never touches stdout |
| no process-layer test seam | Runner trait + FakeRunner | PORT | one trait replaces the ad-hoc UpdateCommandRunner and gives the ten caller modules a fake |
| timeout + orphan-pipe handling | duct kill_and_reap | KEEP | toride's default path waits unbounded after kill; superai's grace-bounded abandonment is ahead |
| async_runner / tokio_runner / streaming | not needed | KEEP | behind tokio features superai would never enable; GPUI is not tokio |

### Env composition

superai composes one BTreeMap: inherited, or empty under clear_env, then the env additions, then env_remove, which wins on a same key (crates/superai-core/src/process.rs:218-235). That map is duct's only env input (crates/superai-core/src/process.rs:362-366), so precedence never depends on the order duct applies env wrappers. Windows keys fold ASCII-case-insensitively through one canonical key function (crates/superai-core/src/process.rs:204-216). The remove-wins rule is pinned on the composed map (crates/superai-core/src/process.rs:774-793) and end to end through a real child (crates/superai-core/src/process.rs:796-823).

toride's default is the opposite precedence: an explicit env entry beats env_remove (crates/toride-runner/src/duct_runner.rs:601-616, pinned at crates/toride-runner/src/duct_runner.rs:753-762). The conflict is now a knob: EnvPrecedence::RemoveWins (crates/toride-runner/src/policy.rs:17-25) applies removals after additions and is pinned both plain and under clear_env (crates/toride-runner/src/duct_runner.rs:617-625, tests at crates/toride-runner/src/duct_runner.rs:1194-1223). Two differences have no knob. toride's clear_env keeps SystemRoot, SystemDrive, and WINDIR on Windows (crates/toride-runner/src/policy.rs:237-248, applied at crates/toride-runner/src/duct_runner.rs:595-599), which is friendlier to children that need winsock; superai's clear_env is a truly empty map (crates/superai-core/src/process.rs:221-227). And toride's non-clear path is assembled from duct env()/env_remove() wrapper calls (crates/toride-runner/src/duct_runner.rs:601-625), leaning on duct's application order where superai hands over a finished map.

Verdict: KEEP.

### PATH resolution

superai refuses any `.` or `..` component in the executable (crates/superai-core/src/process.rs:293-303), passes absolute and separator-containing programs through, and resolves bare names against the child's composed PATH with the ambient one as fallback, skipping empty entries (POSIX reads them as the cwd), demanding the execute bit, and never searching the working directory (crates/superai-core/src/process.rs:237-289 and crates/superai-core/src/process.rs:291-321). Pinned from three angles: first-match order follows the child PATH (crates/superai-core/src/process.rs:852-873), the cwd is never searched even with a probe sitting in it and a colon-only PATH (crates/superai-core/src/process.rs:875-898), and dot-relative spellings are refused (crates/superai-core/src/process.rs:900-911).

toride's default, PathResolution::OsSearch, passes the program to the OS verbatim, with a which-based PATHEXT lookup for bare names on Windows (crates/toride-runner/src/policy.rs:100-107 and crates/toride-runner/src/policy.rs:134-140). The opt-in ChildEnvNoCwd is superai's algorithm rebuilt: same dot/dotdot refusal, same child-PATH resolution with parent fallback, same empty-entry skip and execute-bit demand (crates/toride-runner/src/policy.rs:109-131 and crates/toride-runner/src/policy.rs:142-212). So the 2026-10-01 conflict is expressible, per spec, off by default. discovery.rs is not in the spawn path at all: binary_exists and find_binary are which-based parent-PATH helpers for pre-flight checks (crates/toride-runner/src/discovery.rs:22-46), while the runner resolves through the policy gate (crates/toride-runner/src/duct_runner.rs:150).

Verdict: KEEP. Under toride, superai's guarantees become per-call-site opt-ins.

### Output limiting

This is the real gap, and it goes the other way. superai's cap is always on, 1 MiB by default (crates/superai-core/src/process.rs:29 and crates/superai-core/src/process.rs:116), but duct's capture buffers the whole stream and the check runs after the child exits (crates/superai-core/src/process.rs:368 and crates/superai-core/src/process.rs:423-435). The cap is a refusal, not a bound: a child that floods stdout until its timeout budget expires is fully buffered in superai's memory before the refusal lands. The DoS exposure is bounded only by the timeout and the producer's rate.

toride, when a limit is in effect, caps while capturing. It wires os_pipe pipes instead of duct capture (crates/toride-runner/src/duct_runner.rs:282-293), runs two reader threads doing 8 KiB bounded reads against a shared atomic counter, and the thread that pushes the total past the cap kills the child immediately (crates/toride-runner/src/duct_runner.rs:19-20 and crates/toride-runner/src/duct_runner.rs:499-549), with a late-breach re-check so a breach noticed after a clean exit still errors (crates/toride-runner/src/duct_runner.rs:429-438). Retained memory stays at cap plus two read buffers (crates/toride-runner/src/duct_runner.rs:499-506). It is tested against a newline-free flood, proving the cap fires rather than the timeout (crates/toride-runner/src/duct_runner.rs:1081-1100), and that the flood actually dies (crates/toride-runner/src/duct_runner.rs:1102-1132). But the path only runs when a limit applies, and the default is OptIn with no limit (crates/toride-runner/src/policy.rs:52-61, defaults at crates/toride-runner/src/spec.rs:107-112), so toride's default capture is unbounded with no refusal either.

Verdict: PORT the in-capture limiting into process.rs. Bounded memory under a flooding child is the one property here superai does not have, and the mechanics (os_pipe, reader threads, counter, latch, kill) are self-contained.

### Redaction

superai redacts argv positions and equals-forms (crates/superai-core/src/process.rs:124-157, flag list at crates/superai-core/src/process.rs:41-55) and scrubs stderr by blanking the entire field to [REDACTED] when any flag keyword appears, only when redact is set (crates/superai-core/src/process.rs:176-188). stdout is never scrubbed (crates/superai-core/src/process.rs:437-442).

toride collects the secret values themselves: values after sensitive flags, the value half of --flag=value, env values whose key matches REDACT_ENV_KEYS (TOKEN, SECRET, PASSWORD, AUTH, GH_, and more, crates/toride-runner/src/display.rs:16-33), and secret assignments in piped stdin. Each occurrence is replaced anywhere in the stream, longest first, with a minimum-length guard against mangling (crates/toride-runner/src/display.rs:264-302). Both streams get this on success when redact is set (crates/toride-runner/src/display.rs:241-250, applied at crates/toride-runner/src/duct_runner.rs:240-243), and the error path adds an always-on 4 KiB stderr byte cap on a char boundary (crates/toride-runner/src/display.rs:177-180 and crates/toride-runner/src/display.rs:406-420). The flag list is a superset of superai's, adding the ssh-keygen -N and -P passphrase flags (crates/toride-runner/src/redact.rs:24-41).

superai needs part of this. The whole-field [REDACTED] scrub is safe but destroys the diagnostic: when a harness fails with a token in the message, superai users lose the entire error text, while toride's users keep everything except the token. The production env-feeding site is the readiness probe in daemon.rs, which copies a ReadinessSpec's env into the child (crates/superai-core/src/daemon.rs:435-438), and those specs are only ever built by tests today (crates/superai-core/src/daemon.rs:1316 sits inside the test module at crates/superai-core/src/daemon.rs:960); no secrets flow through it, so env-value scrubbing is cheap insurance rather than a live hole, and stdin secret collection answers a flow superai does not have.

Verdict: PORT the value-level scrub for both streams; skip the stdin half.

### The execution seam

toride's Runner trait is one method plus a checked default (crates/toride-runner/src/runner.rs:15-35). FakeRunner backs it with FIFO or exact-match responses, strict mode, call recording, and shared-state clones so a test can inspect calls after handing the runner away (crates/toride-runner/src/fake.rs:71-135, crates/toride-runner/src/fake.rs:198-212, crates/toride-runner/src/fake.rs:221-252). Its matching ignores runtime policy fields such as timeout and output_limit, so tightening a safety knob does not break every registered fake (crates/toride-runner/src/fake.rs:263-276).

superai has no seam at this layer. run_command is a free function (crates/superai-core/src/process.rs:325) called directly by ten modules (crates/superai-core/src/detect.rs:775-879 among them), and the process tests execute real binaries: /bin/sh, echo, sleep, printenv (crates/superai-core/src/process.rs:626-925). The need has already surfaced once. The update flow grew its own private UpdateCommandRunner trait wrapping run_command so hermetic tests could avoid a real npm (crates/superai-core/src/install_execute.rs:1093-1117). One caller inventing a local trait is the tell that the layer wants one.

Verdict: PORT the design, not the crate. A small trait over (executable, args, ExecuteOpts) returning ProcessOutput in superai-core, run_command behind the real impl, a recording fake beside it, and UpdateCommandRunner folded in. The matching-ignores-policy-fields ruling is the detail worth copying exactly.

### Adoption readiness

Both gates from the routing rules pass: the semantics knobs exist and are tested, and the crate is published. Three things still argue against ADOPT today. First, posture is per-spec. DuctRunnerOptions carries only a default timeout and a log flag (crates/toride-runner/src/duct_runner.rs:30-47), so RemoveWins, ChildEnvNoCwd, an always-on cap, metachar rejection, and stdin_null must be spelled on every CommandSpec, and one forgotten knob silently reverts to toride's looser defaults; stdin inherits the parent terminal unless stdin_null is set (crates/toride-runner/src/spec.rs:33-44), which is wrong for a future GPUI app, while superai pins null stdin unconditionally (crates/superai-core/src/process.rs:368). Second, the publish is one day old, a single 0.1.0 with 26 downloads; the dependency rule asks for evidence of maintenance and there is none yet. Third, the crate drags tracing and which into superai-core (crates/toride-runner/Cargo.toml:22-24), which carries neither today. The async side is irrelevant rather than blocking: AsyncRunner, TokioRunner, and streaming sit behind tokio-runner and stream, which default features never pull (crates/toride-runner/Cargo.toml:13-19; crates/toride-runner/src/lib.rs:64-71), and GPUI runs its own executor, so superai would never enable them. Revisit once the crate shows a release history or grows runner-level policy defaults; the swap itself would be mechanical.

### Security notes

The superai-side finding: the always-on cap buffers a flooding child's full output in memory before refusing (crates/superai-core/src/process.rs:368 and crates/superai-core/src/process.rs:423-435), so a hostile or broken harness can grow superai's RSS to its timeout budget's worth of pipe output. The port under output limiting closes it. On argv injection, superai validates metachars at the config layer, not in the runner (crates/superai-core/src/plugin.rs:45 and crates/superai-core/src/plugin.rs:108, crates/superai-core/src/mcp.rs:59 through crates/superai-core/src/mcp.rs:120, crates/superai-core/src/skills.rs:378, crates/superai-core/src/skills.rs:1174, crates/superai-core/src/skills.rs:2631), and run_command itself checks only NUL (crates/superai-core/src/process.rs:336-349); toride's runner-side ArgvPolicy defaults to Allow (crates/toride-runner/src/policy.rs:41-50), so adoption would move that guarantee to per-spec discipline too. On PATH hijack, both sides skip empty entries and refuse cwd-relative programs in their safe modes; toride's default OsSearch inherits whatever the OS does with a colon-only PATH. On secret leakage, superai's env additions reach the child unscrubbed and its stdout is returned raw; the production env-feeding site is the readiness probe (crates/superai-core/src/daemon.rs:435-438) and nothing populates it outside tests, which keeps this a hardening item rather than a hole. On toride's side, the default timeout path kills and then calls handle.wait() with no bound (crates/toride-runner/src/duct_runner.rs:189-202), so a descendant holding the capture pipes can stretch a "timed out" call past its budget; superai bounds exactly this with a 500 ms reap grace and an abandonment note (crates/superai-core/src/process.rs:37 and crates/superai-core/src/process.rs:381-414).
