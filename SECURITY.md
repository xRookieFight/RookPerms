# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| `0.1.x` | yes |

Only the latest release receives fixes while the project is pre-1.0.

## Reporting a vulnerability

Do not open a public issue for a security problem.

Use GitHub's private vulnerability reporting instead:
[Report a vulnerability](https://github.com/xRookieFight/RookPerms/security/advisories/new).

Please include:

- the plugin and PumpkinMC version
- what an attacker can achieve, for example privilege escalation to a higher group
- the smallest set of steps that reproduces it
- any relevant permission data, with private information removed

You can expect a first reply within seven days. Fixes are released as soon as a patch is verified,
and the report is credited in the release notes unless you prefer otherwise.

## Scope

RookPerms decides whether a player holds a permission node, so the following are treated as
security issues:

- gaining a permission that was never granted, directly or through inheritance, wildcards, contexts
  or expiry handling
- bypassing the `RookPerms:command.manage` requirement on the management command
- reading or writing files outside the plugin data folder
- crashing or hanging the server through plugin input

Out of scope: misconfigured groups, permissions granted by other plugins, and issues in PumpkinMC
itself, which should be reported to that project.
