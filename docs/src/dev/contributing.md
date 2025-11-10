# Contributing

Thank you for considering contributing to wayvid!

## Quick Start

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit a pull request

## Development Setup

See [Building from Source](./building.md) and [Development Workflow](./workflow.md).

## Code Style

- Follow `rustfmt` (run `cargo fmt`)
- Pass `clippy` (run `cargo clippy -- -D warnings`)
- Write documentation for public APIs
- Add tests for new features

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Examples:
```
feat(gui): Add video source browser
fix(mpv): Resolve memory leak in HDR pipeline
docs: Update installation instructions
```

## Pull Requests

### Checklist
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy --all-features -- -D warnings`)
- [ ] Tests pass (`cargo test --all-features`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if user-facing)

### PR Title
Use same format as commits: `type(scope): description`

### Description
Include:
- What changed
- Why it changed
- How to test
- Related issues

## Testing

### Required
- Unit tests for new logic
- Integration tests for features
- Manual testing on supported compositors

### Test Coverage
Aim for:
- Core logic: 80%+
- Backend: 60%+
- Utils: 70%+

## Documentation

Update relevant docs:
- Code comments (public APIs)
- User guide (new features)
- Reference (config options)
- CHANGELOG.md

## Issue Reports

### Bug Reports
Include:
- wayvid version
- Compositor and version
- System info (distro, GPU)
- Steps to reproduce
- Logs (`RUST_LOG=debug wayvid`)

### Feature Requests
Include:
- Use case
- Expected behavior
- Alternative solutions considered

## Code Review

### What We Look For
- Correctness
- Performance
- Maintainability
- Documentation
- Test coverage

### Response Time
- Initial review: 1-3 days
- Follow-up: 1-2 days

## License

By contributing, you agree to license your code under MIT OR Apache-2.0.

## Questions?

- GitHub Discussions
- Issues for bugs/features
- Email: YangYuS8@163.com

## Recognition

Contributors are:
- Listed in README.md
- Mentioned in release notes
- Given credit in commits

Thank you for contributing! 🎉
