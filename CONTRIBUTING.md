# Contributing to jvlauncher

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Development Setup

1. Fork and clone the repository
2. Follow the setup instructions in [QUICK_START.md](QUICK_START.md)
3. Create a new branch for your feature: `git checkout -b feature/my-feature`

## Code Style

### Rust

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- Write tests for new functionality

### JavaScript

- Use modern ES6+ syntax
- Keep functions small and focused
- Add comments for complex logic
- Use meaningful variable names

### Commits

- Write clear, descriptive commit messages
- Use conventional commit format: `type(scope): description`
  - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
  - Example: `feat(launcher): add webapp session persistence`

## Pull Request Process

1. Update documentation if needed
2. Add tests for new features
3. Ensure all tests pass: `cargo test`
4. Update CHANGELOG.md with your changes
5. Submit PR with clear description of changes

## Reporting Bugs

When reporting bugs, please include:

- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Error messages or logs
- Screenshots if applicable

## Feature Requests

We welcome feature requests! Please:

- Check if the feature has already been requested
- Describe the use case clearly
- Explain why it would benefit other users
- Consider submitting a PR to implement it

## Code Review

All submissions require review. We aim to:

- Respond to PRs within 48 hours
- Provide constructive feedback
- Merge approved PRs quickly

## Testing

- Write unit tests for new backend functions
- Test on all supported platforms when possible
- Verify UI changes in both light and dark themes

## Questions?

Feel free to open an issue for questions or discussion.

Thank you for contributing!

