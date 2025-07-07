# Contributing to DeTrack Node Contract

Thank you for your interest in contributing to the DeTrack Node Contract! This document provides guidelines for contributing to this project.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Create a new branch for your feature or bug fix
4. Make your changes
5. Test your changes
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.60+ with `wasm32-unknown-unknown` target
- Docker (for optimized builds)
- Node.js 18+ (for testing scripts)

### Building the Contract

```bash
# Build the contract
cargo build

# Run tests
cargo test

# Create optimized build
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.14.0
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run integration tests
chmod +x scripts/test-contract.sh
./scripts/test-contract.sh
```

## Code Style

- Follow standard Rust formatting with `cargo fmt`
- Use `cargo clippy` to check for common mistakes
- Write comprehensive tests for new functionality
- Document public APIs with doc comments

## Commit Message Guidelines

- Use clear, descriptive commit messages
- Start with a verb in present tense (e.g., "Add", "Fix", "Update")
- Keep the first line under 50 characters
- Add detailed description in the body if needed

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add a clear description of the changes
4. Reference any related issues
5. Wait for code review and address feedback

## Reporting Issues

When reporting issues, please include:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, etc.)

## Security

If you discover a security vulnerability, please email the maintainers directly rather than opening a public issue.

## License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.
