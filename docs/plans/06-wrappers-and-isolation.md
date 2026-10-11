# Subplan 06 — wrappers and isolation

Parent: [master plan](master-plan.md)  
Task prefix: WRP  

All five exit gates are closed: WRP-01..05/08 by area 6b, WRP-06/07 by area 6a, with
every catalog entry carrying a typed isolation class and support reason (48/48).

Shipped: WrapperPlan invocation spec, PowerShell/cmd generators with goldens,
resolve_wrapper_destination in preflight, fixed-path activation, and daemon lifecycle.

## Exit gate

- [x] Every adapter declares one isolation class and exact limitations.
- [x] Full/constrained/single-instance claims have runtime evidence.
- [x] Generated wrappers are deterministic, marked, quoted, and secret-free.
- [x] User wrappers are never overwritten.
- [x] Fixed-path and daemon flows use dedicated lifecycle behavior.

## Residuals

> Completion record (run 3): WRP-01..05/08 closed by area 6b (full WrapperPlan invocation
> spec fields, PowerShell/cmd generators with goldens, resolve_wrapper_destination wired
> LIVE into preflight, diagnostic_probe + isolation_evidence/IsolationVerdict with
> Constrained marking,
> `two_concurrent_ide_profiles_split_state_dirs`, foreign write_wrapper refusal
> bytes-untouched); WRP-06/07 closed by area 6a (activation lock/reconcile/backup + daemon
> port/pid/readiness lifecycle). Every catalog entry carries a typed isolation class and
> support reason (48/48, catalog-enforced); full WRP-01 invocation detail (state paths,
> shared-state warnings) is declared on the adapters whose corpus documents it
> (claude-code, cline, gptme) — breadth beyond that needs per-harness research, same
> discipline as capability declarations. Windows graceful-stop stage punted to area 8's
> taskkill /F both-stages decision; non-Linux stop verification conservatively refuses
> without platform evidence (plan-mandated).
>
> Final-gate-fix round: WRP-07's remaining spec elements landed —
> `DaemonStartConfig.foreground` (foreground launch: inherited stdio, start blocks until
> the daemon exits and cleans the identity; `DaemonLaunch::Foreground` carries the exit
> status — `foreground_launch_runs_to_exit_and_cleans_identity`) and
> `ShutdownCommand`/`StopOptions.shutdown` (graceful shutdown command run INSTEAD of
> signaling first, with the TERM/KILL escalation kept as fallback and an empty executable
> refused up front — `stop_uses_declared_shutdown_command_before_any_signal` proves the
> daemon exits through the command's own trigger via a graceful-exit marker a signal
> could never produce, `shutdown_command_that_does_not_stop_falls_back_to_signals`,
> `empty_shutdown_executable_is_refused_up_front`).
