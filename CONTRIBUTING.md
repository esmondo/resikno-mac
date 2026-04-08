# Contributing to Resikno

Thank you for your interest in contributing to Resikno! This document provides guidelines for contributing.

## Development Setup

1. **Prerequisites**
   - Rust 1.75 or later
   - macOS 10.15+ (for testing)

2. **Clone and Build**
   ```bash
   git clone https://github.com/esmondo/resikno-mac.git
   cd resikno-mac
   cargo build
   ```

3. **Run Tests**
   ```bash
   cargo test
   ```

## Code Guidelines

- Follow Rust naming conventions
- Add comments for complex logic
- Ensure `cargo clippy` passes without warnings
- Format code with `cargo fmt`

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and ensure they pass
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Reporting Issues

When reporting issues, please include:
- macOS version
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior

## Code of Conduct

Be respectful and constructive in all interactions.
