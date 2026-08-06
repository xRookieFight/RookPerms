# Contributing

Thanks for taking the time to improve RookPerms.

## Getting started

```
rustup target add wasm32-wasip2
cargo build --release
cargo test -p rookperms-api --target x86_64-unknown-linux-gnu
```

`.cargo/config.toml` pins `wasm32-wasip2` as the default target, the plugin crate only links as a
WebAssembly component. Tests live in `rookperms-api` and must be run against the host target.

## Project layout

| Path | Purpose |
| --- | --- |
| `rookperms-api` | Permission model and resolution engine. No server dependency. |
| `rookperms-plugin` | PumpkinMC integration: commands, event listeners, JSON storage. |

Anything that can be expressed without the server belongs in `rookperms-api`, so it stays testable.
Keep `rookperms-plugin` limited to translating between the host API and the model.

## Before opening a pull request

- `cargo fmt --all`
- `cargo clippy --workspace --target wasm32-wasip2 -- -D warnings`
- `cargo clippy -p rookperms-api --target x86_64-unknown-linux-gnu --all-targets -- -D warnings`
- `cargo build --release`
- `cargo test -p rookperms-api --target x86_64-unknown-linux-gnu`
- Update `README.md` when you change commands, permissions or the stored data format.

## Style

- Follow the surrounding code. Naming, module layout and error handling are already consistent.
- Prefer composition over inheritance style trait stacking, and keep types focused.
- Comments are only for things the code cannot say. Write them in English.
- Use `-` instead of long dashes in documentation.

## Commits

Conventional commit titles, one purpose per commit:

```
feat: add temporary permission nodes
fix: keep default group when a parent is removed
docs: document the duration format
```

Avoid unrelated formatting changes so blame history stays useful.

## Adding a command

1. Add the handler to `rookperms-plugin/src/commands/`.
2. Wire it into the tree in `commands/mod.rs`.
3. Add the usage line to the help output in `commands/admin.rs`.
4. Document it in `README.md`.

## Reporting bugs

Open an issue with the server version, the plugin version, the command you ran and the relevant
log output. If the problem involves stored data, include the matching JSON file with any private
information removed.
