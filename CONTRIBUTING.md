# Contributing to Resourcely

Thank you for your interest in contributing to Resourcely! This document provides guidelines for contributing to this project.

## Project Philosophy

Resourcely follows a philosophy of **minimalism with maximum flexibility**. We aim to:

- Keep the API surface as small as possible while providing essential functionality
- Maintain high code quality with comprehensive documentation and test coverage
- Provide flexibility without compromising simplicity
- Follow Rust best practices and idiomatic patterns

## Development Status

**Pre-Release Phase**: This library is currently in pre-release development. The first public release is planned once local CRUD operations are implemented, representing a complete MVP for practical use cases.

We maintain a clean development roadmap and can accommodate new requirements as they arise, but all contributions must align with the library's core principles.

## Contribution Standards

All contributions must meet these requirements:

### 1. **Code Quality**

- Follow Rust conventions and best practices
- Use `rustfmt` for code formatting
- Pass all `clippy` lints without warnings
- Maintain the existing code style and patterns

### 2. **Documentation**

- **All public APIs must have comprehensive documentation**
- Include doc comments for all public functions, structs, enums, and traits
- Provide code examples in documentation where appropriate
- Update README.md if adding new features or changing existing behavior

### 3. **Testing**

- **All new functionality must have comprehensive test coverage**
- Include both unit tests and integration tests where applicable
- Test edge cases and error conditions
- Ensure all tests pass before submitting

### 4. **Scope Alignment**

- Contributions should align with the library's minimalist philosophy
- New features should provide significant value without adding complexity
- Consider if the feature belongs in core or as an optional extension

## Types of Contributions

### High Priority (Welcome)

- Bug fixes with tests
- Documentation improvements
- Performance optimizations
- Test coverage improvements
- Implementation of planned features from the TODO list

### Medium Priority (Considered)

- New features that align with the roadmap
- API improvements that maintain backward compatibility
- Additional format support (TOML, plain text)

### Low Priority (Selective)

- Large architectural changes
- Features that significantly expand the scope
- Breaking changes (will be carefully evaluated)

## Submission Process

### 1. **Before You Start**

- Check existing issues and pull requests to avoid duplication
- For significant changes, open an issue first to discuss the approach
- Ensure your contribution aligns with the project philosophy

### 2. **Development Process**

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes following the contribution standards
4. Write comprehensive tests
5. Update documentation
6. Ensure all tests pass: `cargo test`
7. Check code formatting: `cargo fmt --check`
8. Run clippy: `cargo clippy -- -D warnings`

### 3. **Pull Request Requirements**

- **Clear description** of what the PR does and why
- **Reference any related issues**
- **Include test results** showing all tests pass
- **Documentation updates** for any API changes
- **Breaking changes** must be clearly marked and justified

### 4. **Review Process**

- All PRs require review and approval
- Reviews focus on code quality, test coverage, and alignment with project goals
- Be prepared to make revisions based on feedback
- Maintain a respectful and collaborative tone

## Development Setup

```bash
# Clone the repository
git clone https://github.com/dominikj111/resourcely.git
cd resourcely

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build documentation
cargo doc --open
```

## Code Style Guidelines

### Naming Conventions

- Use descriptive names for functions and variables
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use meaningful type names that reflect their purpose

### Error Handling

- Use `Result` types for operations that can fail
- Provide descriptive error messages using `thiserror`
- Follow the existing error handling patterns

### Documentation Style

````rust
/// Brief description of what the function does.
///
/// More detailed explanation if needed, including:
/// - Important behavior notes
/// - Examples of usage
/// - Error conditions
///
/// # Arguments
///
/// * `param` - Description of the parameter
///
/// # Returns
///
/// Description of what is returned
///
/// # Errors
///
/// When this function will return an error
///
/// # Examples
///
/// ```rust
/// // Example usage
/// ```
pub fn example_function(param: Type) -> Result<ReturnType, Error> {
    // Implementation
}
````

## Questions and Support

- **Issues**: For bugs, feature requests, or questions
- **Discussions**: For general questions about usage or design
- **Email**: For private inquiries

## Recognition

Contributors will be recognized in the project documentation and release notes. We appreciate all contributions that help make Resourcely better!

## License

By contributing to Resourcely, you agree that your contributions will be licensed under the BSD 3-Clause License.
