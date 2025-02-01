# Contributing to Obel

## Getting Started

1. Fork the repository and clone your fork
2. Create a new branch: `git checkout -b my-branch-name`
3. Make your changes
4. Push to your fork and submit a pull request

## Development Setup

1. Install Rust via [rustup](https://rustup.rs/)
2. Install OS-specific dependencies:
   - Linux: `sudo apt-get install libasound2-dev libudev-dev`
   - Windows: No additional dependencies required
   - macOS: No additional dependencies required

## Project Structure

- `/crates`: Core Obel crates
- `/examples`: Example projects showcasing Obel features
- `/assets`: Game assets used by examples
- `/benches`: Performance benchmarks
- `/tools`: Development tools and scripts

## Pull Request Guidelines

1. Update documentation for any modified public APIs
2. Add tests for new functionality
3. Ensure all tests pass: `cargo test --workspace`
4. Format your code: `cargo fmt`
5. Run clippy: `cargo clippy --workspace`

## Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Reference issues and pull requests where appropriate

## Code Style

- Follow Rust standard naming conventions
- Document public APIs using rustdoc
- Write clear commit messages
- Keep code modular and maintainable

## Testing

1. Run the test suite: `cargo test --workspace`
2. Run specific examples: `cargo run --example my_example`
3. Test on all supported platforms if making platform-specific changes

## Documentation

- Update relevant documentation in `/docs`
- Document new features with examples
- Keep the API documentation up to date

## Community

- Join our [Discord server](https://discord.gg/3jq8js8u)
- Follow our [Code of Conduct](CODE_OF_CONDUCT.md)
- Be welcoming and respectful to all contributors

## License

By contributing to Obel, you agree that your contributions will be licensed under its dual MIT/Apache-2.0 license.
