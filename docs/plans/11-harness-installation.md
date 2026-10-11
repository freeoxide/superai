# Subplan 11 — harness detection, installation, update, and uninstall

Parent: [master plan](master-plan.md)  
Task prefix: PKG  

Status: complete. PKG-01/02/05 were verified at baseline, PKG-03..10 were closed by area 7 and judge-verified, and no live package manager executes in the default suite.

What shipped in crates/superai-core: the install catalog and argv tokens (install_catalog.rs), planning and execution (install_plan.rs, install_execute.rs), the toride bridge for the six package-manager kinds (toride_bridge.rs), and duct-backed execution (process.rs).

## Exit gate

- [x] toride/mise/duct dependencies and package identifiers verified before use.
- [x] Detection covers pre-existing and multiple installs.
- [x] Install/update/uninstall are structured, previewed, and verified.
- [x] Binary lifecycle never implies config/data deletion.
- [x] Adapter compatibility is checked around updates.
- [x] External/manual installation is represented honestly.

## Residuals

> Completion record (run 3): PKG-01/02/05 verified at baseline (toride verified NOT on
> crates.io — duct 1.1.1 is the single direct dependency, spike recorded in process.rs);
> PKG-03 (pipx/uv/dpkg probes, hermetic), PKG-04 (real registry availability probes with
> typed Unavailable-with-reason; Direct/External misattribution fixed to docs-URL preview
> + execute refusal), PKG-06 (persisted receipts, verified-new-only), PKG-07 (skip-on-
> matching, strict post-update validation, read-only instance revalidation, hermetic
> update runner seam), PKG-08 (receipt-owned preflight + real binary-missing registry
> marking), PKG-09 (pin-exact-binary + wrapper pin + ambiguity report), PKG-10 (typed
> CoreError::ExternalInstallRequired) closed by area 7, judge-verified incl. the
> offline-probe fix round. No live package manager executes in the default suite.
