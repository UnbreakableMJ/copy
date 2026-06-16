# Contributing to Copy

Thanks for your interest in making Copy better! 🎉

## Ways to Contribute

### 🐛 Report Bugs

Found a bug? Help us fix it:
1. Check [existing issues](https://github.com/UnbreakableMJ/copy/issues) first
2. Open a new issue with:
   - What you expected to happen
   - What actually happened
   - Steps to reproduce
   - Your OS and Copy version (`copy --version`)

### 💡 Request Features

Have an idea? We'd love to hear it:
1. Check [existing issues](https://github.com/UnbreakableMJ/copy/issues) for similar requests
2. Open a new issue describing:
   - The feature you want
   - Why it would be useful
   - Example use case

### 📝 Improve Documentation & Test Coverage

- Fix unclear instructions
- Add examples
- Add more tests

### 🔧 Submit Code

Ready to contribute code? Great!

#### Setup
```bash
# Fork and clone
git clone https://github.com/UnbreakableMJ/copy
cd copy

# Build and test
cargo build
cargo test
```

#### Development
```bash
# Create a branch
git checkout -b fix/issue-name

# Make your changes
# ... edit files ...

# Format code
cargo fmt

# Run tests
cargo test

# Commit with clear message
git commit -m "Fix: description of what you fixed"
```

#### Pull Request
```bash
# Push your branch
git push origin fix/issue-name

# Open PR on GitHub with:
# - Clear title
# - Description of changes
# - Link to related issue (if any)
```

## Code Guidelines

- **Style**: Run `cargo fmt` before committing
- **Tests**: Add tests for bug fixes and new features
- **Commits**: Use clear messages (Fix:, Add:, Update:, etc.)
- **Small PRs**: Easier to review, faster to merge

## Questions?

- Open an issue for general questions
- Comment on relevant issues for specific questions
- Tag `@11happy` in PRs for review
