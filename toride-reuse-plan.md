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
