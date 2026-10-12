# toride VPS services plan

Task prefix: VPS (toride gains)
Estimate: M

## Context

toride manages local workstation services well and remote machines not at all: it can
prepare ssh keys and configs, but it cannot run a command on a host, upload a file to
one, or install a systemd unit there. This plan adds exactly those three abilities and
publishes the touched crates, so any external tooling that deploys services to VPSes
can drive toride instead of shelling out to ad-hoc ssh scripts. The gains are generic:
nothing here knows which service is being deployed.

Baseline, verified this round: 24 crates. Published on crates.io at 0.1.0 (checked
2026-10-12 against `https://crates.io/api/v1/crates/<name>`): toride-ssh, toride-service,
toride-proxy, toride-wireguard, toride-tailscale, toride-runner, toride-apps. toride-ssh
manages local ssh keys/config/agent
(`crates/toride-ssh/src/lib.rs:78`) with no remote-exec or file-transfer service;
toride-service wraps local systemctl only (`crates/toride-service/src/manager.rs:121-237`);
toride-proxy renders/manages nginx (default) and Caddy (off-default feature) with
backup/diff/validate (`crates/toride-proxy/src/lib.rs` module docs, `src/caddy.rs:15`);
toride-wireguard wraps wg with config read/write and backup
(`crates/toride-wireguard/src/client.rs:36`, `src/config.rs:25`); toride-tailscale wraps
the tailscale API (`crates/toride-tailscale/src/client.rs:45`). toride ships musl
release artifacts via cargo-dist (`cargo-dist.toml`).

## VPS-01 — what the deploy flow needs that toride lacks today

Verified against the crates:

1. Remote execution and file transfer. `toride-ssh` today manages local ssh
   keys/config/agent/known-hosts/certificates (`crates/toride-ssh/src/lib.rs:143-184`)
   with no exec or upload service. Add an `exec` service (run a command on a host via
   argv-disciplined `ssh` invocations through `toride_runner::Runner`
   (`crates/toride-runner/src/runner.rs:15`)) and an `upload` service (sftp batch or
   `scp` with the same argv discipline; explicit host-key policy; no shell string
   assembly anywhere).
2. Unit installation. `toride-service` wraps systemctl verbs
   (`crates/toride-service/src/manager.rs:121-237`) but cannot write a unit file or run
   `daemon-reload`. Add unit install (atomic write to `/etc/systemd/system/<name>.service`
   with pre-write backup, reusing toride-fs atomic-write and fd-lock primitives) plus
   `daemon-reload`, `is-active`/`enable`/`start` composed over the existing manager.
3. Publish the touched crates (toride-ssh 0.2, toride-service 0.2) per the toride
   publishing doc; downstream consumers take them from crates.io.
4. Abuse test family for the new surfaces, in toride's own suite: exec/upload reject
   shell-string arguments and untrusted host aliases, host-key policy violations are
   typed errors not warnings, unit install backs up before overwrite and restores on
   mid-sequence failure, and no uploaded payload or remote path leaks into logs.

Nothing else in toride is modified: toride-proxy (nginx default; caddy behind its
off-default feature) already renders, validates, diffs, and backs up ingress config
(`crates/toride-proxy/src/lib.rs` module docs); toride-wireguard already reads/writes
interface configs with backup (`crates/toride-wireguard/src/config.rs:44-61`); toride-
tailscale already wraps the API (`crates/toride-tailscale/src/client.rs:45`).

## Tests

- exec/upload reject shell-string arguments and untrusted host aliases; host-key
  policy violations are typed errors, not warnings.
- Unit install backs up before overwrite and restores on mid-sequence failure.
- No uploaded payload or remote path leaks into logs.

## Exit gate

- [ ] exec and upload services run argv-disciplined ssh/sftp with an explicit
      host-key policy and no shell string assembly.
- [ ] Unit install writes atomically with pre-write backup, then daemon-reload,
      is-active/enable/start compose over the existing manager.
- [ ] toride-ssh 0.2 and toride-service 0.2 published per the toride publishing doc.
- [ ] The abuse family of VPS-01 item 4 passes in toride's own suite.

## Out of scope

- Any change outside toride-ssh and toride-service; the other crates already cover
  their part of a deploy flow (see VPS-01's closing paragraph).
- Anything specific to a particular deployed service; the gains stay generic.

## Companion plans

The consumer is external tooling; the known one lives in the superai repository's
subplan 16 (VPS deploy and geo-exit), which drives these services to deploy and
monitor a remote proxy. This document's canonical home is the toride repository; the
copy in the superai repo under `docs/plans/export/` is an export.
