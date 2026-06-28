# 0004 — Deployment model: env-var config and a hardened system service

- Status: accepted
- Date: 2026-06-28

## Context

The MVP turns telltaled from a one-shot probe into a resident daemon that must be
configured per host (endpoint, a sensitive per-host secret, sampling interval) and
run unattended on every machine in the fleet — first krypton, later helium and
beyond. Two questions need a durable answer because they shape both the daemon's
code surface and how every host is provisioned:

1. **How is the daemon configured?** Options ranged from a structured config file
   (TOML + serde) to CLI flags to environment variables. The daemon needs only a
   handful of values today; the secret must never land in a committed file; and the
   same mechanism should serve a hand-installed host (krypton) and a
   secrets-managed one (helium, via sops-rendered files).
2. **How does it run?** As a per-login user service or a system service; with what
   privilege. telltaled is a resident process with network egress, which AGENTS.md
   already flags as needing a security posture. The dotfiles repo has precedent for
   both user services (workstations) and system services (servers).

The overriding constraint is low host overhead, and a secondary goal is one
artifact that installs identically across hosts rather than per-host variants.

## Decision

**Configuration is environment variables only**, with no config-file parser in the
MVP: `TELLTALED_YGGIO_URL`, `TELLTALED_YGGIO_SECRET` (sensitive),
`TELLTALED_INTERVAL_SECS` (optional, default 60). They are delivered through
systemd's `EnvironmentFile`. Missing required variables are a clean startup error,
not a panic.

**telltaled runs as a system service on every host**, under a dedicated
unprivileged `telltaled` user, defined by a single hardened unit (e.g.
`NoNewPrivileges`, `ProtectSystem`, `ProtectHome`, `PrivateTmp`, and
`RestrictAddressFamilies` limited to inet). The daemon owns its deployment
contract: the service unit and a commented `*.env.example` template (placeholder
values, no real secret) live in a `packaging/` directory in the telltaled repo.
Installers — manual on krypton, Ansible on helium — only copy the binary, copy the
unit, render the host's secret into the env file, and enable the service.

## Consequences

- No `serde`/`toml` dependency for the MVP; reading three values is `std::env`.
  When many collectors need per-collector settings, a config file becomes
  worthwhile — a later decision that would supersede the config half of this ADR.
- The secret stays out of version control: it exists only in the host's untracked
  env file (mode 0600 on krypton; sops-encrypted in git and rendered on helium).
  One delivery mechanism (`EnvironmentFile`) serves both hosts.
- `/proc/loadavg` is world-readable, so the dedicated user needs no capabilities and
  the unit can be locked down hard — pre-empting the security follow-up AGENTS.md
  flags for a network-egress daemon.
- One unit definition installs identically everywhere; krypton needs `sudo` once to
  install (the cost of parity over the friction-free user-service path).
- The unit and the env-var contract live beside the code that defines them, so a
  change to what the daemon reads or how it runs lands in one commit; dotfiles and
  Ansible stay thin installers over an artifact the daemon repo owns.
