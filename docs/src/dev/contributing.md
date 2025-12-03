# Contributing

## Quick Start

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid
cargo build --all-features
cargo test
cargo clippy
```

## Before PR

- [ ] `cargo fmt`
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] `cargo test --all-features`
- [ ] Update CHANGELOG.md if user-facing

## Commit Format

```
type(scope): description

feat(gui): Add video source browser
fix(mpv): Fix memory leak
docs: Update installation
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`

## Bug Reports

Include:
- wayvid version
- Compositor
- Steps to reproduce
- Logs: `RUST_LOG=debug wayvid run`

## License

MIT OR Apache-2.0
