## Summary

<!-- What does this change and why? Link the issue it closes, for example "Closes #12". -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation
- [ ] Build, CI or tooling

## Checklist

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --target wasm32-wasip2 -- -D warnings`
- [ ] `cargo build --release`
- [ ] `cargo test -p rookperms-api --target x86_64-unknown-linux-gnu`
- [ ] Tested on a running PumpkinMC server
- [ ] `README.md` updated if commands, permissions or the data format changed

## Compatibility

<!-- Does this change stored JSON, permission nodes or command syntax? Describe the migration
     path for existing servers, or write "none". -->
