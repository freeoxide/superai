# Subplan 04 — instance lifecycle

Parent: [master plan](master-plan.md)  
Task prefix: INS  

Status: complete. INS-01..09 closed by areas 6a/6b over judge-verified rounds; INS-10 fixed-path activation and daemon lifecycle delivered in 6a.

What shipped: registration, create preflight and mirroring, rename, reconfigure, removal, repair, fixed-path activation, and daemon control in superai-core (activation.rs, daemon.rs).

## Exit gate

- [x] Default, created, mirrored, adopted, fixed-path, and daemon flows exist.
- [x] Record is committed only after target verification.
- [x] Source instance cannot change during mirror/create.
- [x] Remove choices are distinct and recoverable.
- [x] Repair is ownership-aware.
- [x] Instance records remain free of harness-owned values/secrets.

## Residuals

> Completion record (run 3): INS-01..09 closed by areas 6a/6b (judge-verified rounds 1-2:
> real foreign-managed determination, blocking provider/sink preflight, Linked/Transformed
> mirror kinds with mode preservation, real before/after source-digest proof, wrapper-file
> rename on disk, demo-marker-free reconfigure over provider/template/skill/plugin/MCP
> mutations, FixedPathEntries removal, TemplateVersionDrift + journal repairs); INS-10
> fixed-path activation + WRP-07 daemon lifecycle delivered in 6a (activation.rs/daemon.rs,
> `activation_swaps_content_with_backup_and_restores_prior_profile`,
> `daemon_start_ready_stop_round_trip`). Records secret/foreign-value-free pinned by
> `property_registry_no_forbidden_fields`. Daemon lifecycle for openclaw itself stays
> honestly ResearchBlocked (documented DaemonConstraints); the generic machinery is real
> and tested.
>
> Final-gate-fix round: INS-02's "asset inheritance choices" request field landed
> (`CreateRequest::asset_inheritance` / `AssetInheritance` — exclusions validated against
> the adapter's declared link-safe assets in preflight, opted-out assets copied privately;
> tests `asset_inheritance_choice_copies_excluded_shared_asset`,
> `asset_inheritance_undeclared_exclusion_is_a_preflight_conflict`), and the INS-05/INS-09
> rename inconsistency the final gate flagged (a verbatim byte move left the OLD instance
> name in the wrapper marker while repair detection regenerates with the new one, so every
> rename manufactured a spurious WrapperDrift finding) is fixed by design: rename now
> REGENERATES the wrapper through the deterministic generator (marker + digest updated,
> atomic write with backup; `rename_moves_wrapper_file_and_updates_record`,
> `rename_leaves_no_wrapper_drift_for_repair_detection`). Renaming an instance whose
> on-disk launcher is not superai-owned refuses typed (ForeignOwnership through the
> wrapper writer) rather than overwriting foreign bytes.
