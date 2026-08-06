<div align="center">

<img src=".github/assets/logo.svg" alt="RookPerms" width="128" height="128">

# RookPerms

**Group based permission management plugin for [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin).**

[![CI](https://img.shields.io/github/actions/workflow/status/xRookieFight/RookPerms/ci.yml?branch=master&label=CI&style=flat-square)](https://github.com/xRookieFight/RookPerms/actions/workflows/ci.yml)
[![Format](https://img.shields.io/github/actions/workflow/status/xRookieFight/RookPerms/fmt.yml?branch=master&label=fmt&style=flat-square)](https://github.com/xRookieFight/RookPerms/actions/workflows/fmt.yml)
[![Release](https://img.shields.io/github/v/release/xRookieFight/RookPerms?include_prereleases&style=flat-square)](https://github.com/xRookieFight/RookPerms/releases)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/wasm32--wasip2-component-654ff0?style=flat-square&logo=webassembly&logoColor=white)](https://component-model.bytecodealliance.org)

</div>

---

## Features

- **Groups with inheritance** - a group may inherit any number of other groups, at any depth.
- **Weights** - when two groups disagree, the heavier one wins.
- **Wildcards** - `essentials.*` grants every child node, `*` grants everything.
- **Negative nodes** - explicitly deny a node that inheritance would otherwise grant.
- **Temporary permissions** - `settemp ... 7d`, expired automatically without a restart.
- **Contexts** - limit a node to a world, so `essentials.fly` can exist only in the nether.
- **Prefix and suffix** - highest priority meta wins, applied to the player display name.
- **Offline editing** - reference players by name through the username index, or by uuid.
- **Live updates** - every change invalidates the affected caches immediately.

## Installation

Grab `rookperms.wasm` from the [latest release](https://github.com/xRookieFight/RookPerms/releases)
and drop it into your server's `plugins` folder, then restart. The plugin creates its own data
folder on first start and registers a `default` group.

## Building

```bash
rustup target add wasm32-wasip2
cargo build --release
```

`.cargo/config.toml` pins the build target, the plugin only links as a WebAssembly component. The
artifact lands at `target/wasm32-wasip2/release/rookperms.wasm`.

Tests run against the host target:

```bash
cargo test -p rookperms-api --target x86_64-unknown-linux-gnu
```

## How it works

Every permission check the server performs for a player is intercepted through
`PlayerPermissionCheckEvent`. If RookPerms has a decision for the requested node it overrides the
server result, otherwise the server's own answer is kept untouched, so other plugins and vanilla
operator levels keep working.

Resolution applies holders in order, lowest priority first:

1. inherited groups, deepest in the inheritance tree first
2. groups with a higher weight
3. the user's own nodes

Within a holder, a node only applies if it has not expired and its contexts are satisfied by the
player's current contexts. Lookup then tries the exact node, then each wildcard parent from the most
specific to the least, and finally `*`.

```
admin        (weight 100)  essentials.*        = true
  member     (weight 10)   essentials.fly      = false
Rook         (user)        essentials.home     = false

Rook -> essentials.fly   = true   (admin outweighs member)
Rook -> essentials.home  = false  (user node beats the wildcard)
Rook -> worldedit.set    = undefined, the server decides
```

## Data

Everything is stored as JSON inside the plugin data folder:

```
data/groups/<group>.json
data/users/<uuid>.json
data/usernames.json
```

```json
{
  "name": "admin",
  "weight": 100,
  "nodes": [
    { "key": "essentials.*", "value": true },
    { "key": "essentials.fly", "value": false, "expiry": 1767225600, "context": { "world": ["nether"] } }
  ],
  "parents": ["member"],
  "prefix": { "value": "[Admin] ", "priority": 100 }
}
```

## Commands

Base command: `/rookperms`, aliases `/rp` and `/perms`. Requires `RookPerms:command.manage`,
granted to permission level 3 and above by default. Running `/rp` without arguments prints the help.

### Groups

| Command | Description |
| --- | --- |
| `/rp group <group> create` | Creates a group |
| `/rp group <group> delete` | Deletes a group and detaches it from every holder |
| `/rp group <group> info` | Shows weight, meta, parents and nodes |
| `/rp group <group> permission set <node> <true\|false>` | Sets a permission |
| `/rp group <group> permission settemp <node> <true\|false> <duration>` | Sets a temporary permission |
| `/rp group <group> permission unset <node>` | Removes a permission |
| `/rp group <group> parent add\|remove <parent>` | Manages inheritance |
| `/rp group <group> meta weight <number>` | Sets the group weight |
| `/rp group <group> meta prefix\|suffix <value>` | Sets meta, `none` clears it |

### Users

| Command | Description |
| --- | --- |
| `/rp user <player> info` | Shows the stored and resolved state |
| `/rp user <player> check <node>` | Shows the resolved value of a node |
| `/rp user <player> permission set <node> <true\|false>` | Sets a permission |
| `/rp user <player> permission settemp <node> <true\|false> <duration>` | Sets a temporary permission |
| `/rp user <player> permission unset <node>` | Removes a permission |
| `/rp user <player> parent add\|remove <group>` | Manages group membership |
| `/rp user <player> parent set <group>` | Replaces all groups with one |
| `/rp user <player> meta prefix\|suffix <value>` | Sets meta, `none` clears it |

A player may be referenced by name or by uuid. Offline players are looked up through the username
index.

### Maintenance

| Command | Description |
| --- | --- |
| `/rp help` | Lists every command, also shown when `/rp` is used alone |
| `/rp listgroups` | Lists every group with its weight |
| `/rp reload` | Reloads all data from disk |
| `/rp save` | Writes all loaded data to disk |

Durations accept combined units, for example `30m`, `12h`, `7d` or `1d12h`.

## Example setup

```
/rp group member create
/rp group member meta weight 10
/rp group member meta prefix [Member]
/rp group member permission set essentials.home true

/rp group admin create
/rp group admin meta weight 100
/rp group admin meta prefix [Admin]
/rp group admin parent add member
/rp group admin permission set essentials.* true

/rp user Rook parent set admin
/rp user Rook permission settemp worldedit.* true 2h
```

## Contributing

Bug reports, ideas and pull requests are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), and
report security problems privately through the
[security policy](SECURITY.md).

## License

[MIT](LICENSE) - Copyright © 2026 xRookieFight
