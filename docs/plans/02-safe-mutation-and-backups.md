# Subplan 02 — safe mutation, backups, restore, and rollback

Parent: [master plan](master-plan.md)  
Task prefix: MUT  

Status: done. Every supported write crosses the single transaction boundary; crash recovery
runs against the production journal. One punted item: MUT-01's `schema_version` token field.

What shipped: one single-file boundary, `transaction::commit_file`/`commit_file_expecting`,
carries every codec store and superai-core write; symlinks are followed only in declared roots.

## Exit gate

- [x] All supported writes go through one transaction boundary.
      > Closed (area 9b): `transaction::commit_file`/`commit_file_expecting` is the ONE
      > single-file mutation boundary — a one-step Transaction whose prepare/commit core
      > performs the full discipline (path validation → fresh snapshot → backup of
      > existing contents → staged parse-validation → §4.2 conflict recheck → atomic
      > replacement → read-back verify). Every codec store (json/jsonc/toml/yaml/env)
      > and the raw-editor commit route through it or the shared
      > `stage_temp_file`+`commit_staged_file` core; every superai-core production
      > write (wrapper, receipts, profiles/active identity, provider env, daemon
      > identity) was migrated off the raw primitive; the `atomic_write` family is
      > `pub(crate)` (journal bookkeeping + backup restore only), and
      > `atomic_write_{injected,with_expected_digest,with_snapshot}` were deleted.
      > Machine-checked structurally by `plan02_atomic_write_family_is_crate_internal`,
      > `plan02_codec_stores_share_the_boundary`, and
      > `plan02_core_and_cli_have_no_direct_atomic_writes` (source-scan tests), plus
      > behavior tests `commit_file_creates_missing_and_backs_up_existing` and
      > `commit_file_expecting_aborts_on_foreign_edit_and_cleans_temp`.
- [x] Backup-before-foreign-write is structurally unavoidable.
- [x] Same-file conflicts abort.
- [x] Single-file replacement is atomic per supported platform.
- [x] Multi-file failure rolls back or reports verified residuals.
- [x] Backup listing and restore work by stable IDs.
- [x] Crash recovery tests pass.

>
> Final-gate-fix round: MUT-02's default link policy ("follow an existing symlink only
> after resolving target within adapter-allowed roots; preserve link itself and mutate
> target") landed at the mutation layer. A `Write` step whose target is currently a
> symlink resolves within the transaction's declared follow roots
> (`Transaction::with_symlink_follow_roots`, canonical comparison) or is refused with the
> typed `ConfigError::SymlinkFollowRefused` — the link is never silently replaced by a
> regular file. Allowed follows retarget staging, backup, §4.2 snapshot, commit, and
> journal onto the referent (link preserved, referent mutated, foreign referent backed
> up) and re-verify the link at commit (a retarget between prepare and commit aborts).
> `Symlink` step targets are constrained to the declared root set whenever roots are
> declared. The single-file boundary exposes the same policy
> (`commit_file_expecting_with_roots`). Tests:
> `write_onto_symlink_refused_outside_follow_roots`,
> `write_follows_and_preserves_symlink_within_declared_roots`,
> `followed_symlink_retarget_between_prepare_and_commit_aborts`,
> `symlink_step_target_outside_declared_roots_is_refused`, and the abuse-suite
> symlink-swap race now asserts both the follow-refused default and the swap-detected
> ConcurrentModification under declared roots.

## Residual

> Completion record (run 3): every store site performs the backup internally and the guard
> is mutation-tested (`mutant_backup_before_write_is_not_skippable`, superai-config
> property_tests); §4.2 conflict window closed by area 2 (fresh recheck before every
> transaction step, judge-verified — no overwrite path for externally-changed files);
> Windows replacement semantics (readonly-attribute projection, clear-before-rename,
> flush discipline) landed in area 8 with the windows CI runner as executor of record;
> rollback/residuals pinned by `commit_failure_surfaces_intermediate_rollback_in_outcome`;
> restore-by-BackupId verified at baseline; crash recovery runs against the production
> journal (`abandoned_journal_at_each_phase_recovers_via_production_journal`). MUT-01's
> `schema_version` token field remains punted (no config-layer schema-version vocabulary;
> area-2 disclosure).
