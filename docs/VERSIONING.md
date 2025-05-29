# Versioning and Release Strategies

This project supports two different release strategies. You can choose the one that best fits your workflow.

## Strategy 1: Manual Version Management (Current)

**Workflow**: `.github/workflows/auto-release.yml`

### How it works
1. **Manual version update**: Developer manually updates version in `Cargo.toml`
2. **Push to main**: When pushed to main branch, CI detects version change
3. **Automatic release**: If version changed, automatically builds and releases

### Usage
```bash
# Update version in Cargo.toml
sed -i 's/version = "0.1.0"/version = "0.1.1"/' Cargo.toml

# Commit and push
git add Cargo.toml
git commit -m "bump: version 0.1.1"
git push origin main
```

### Pros
- ✅ Full control over versioning
- ✅ Explicit version decisions
- ✅ Simple and predictable
- ✅ Works with any commit message format

### Cons
- ❌ Manual version management required
- ❌ Risk of forgetting to update version
- ❌ No automatic semantic versioning

## Strategy 2: Semantic Release (Optional)

**Workflow**: `.github/workflows/semantic-release.yml`

### How it works
1. **Conventional commits**: Use standardized commit message format
2. **Automatic version calculation**: CI analyzes commits and determines version bump
3. **Automatic version update**: CI updates `Cargo.toml` automatically
4. **Automatic release**: Builds and releases with new version

### Conventional Commit Format
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Version Bump Rules
| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `feat:` | Minor (0.1.0 → 0.2.0) | `feat: add new compilation feature` |
| `fix:` | Patch (0.1.0 → 0.1.1) | `fix: resolve memory leak in parser` |
| `BREAKING CHANGE:` | Major (0.1.0 → 1.0.0) | `feat!: redesign API interface` |
| `docs:`, `style:`, `refactor:`, `perf:`, `test:`, `chore:` | Patch | `docs: update README` |

### Usage Examples
```bash
# Feature addition (minor bump)
git commit -m "feat: add support for Python 3.12"

# Bug fix (patch bump)
git commit -m "fix: handle edge case in file parsing"

# Breaking change (major bump)
git commit -m "feat!: redesign command-line interface

BREAKING CHANGE: The --input flag is now required"

# Documentation (patch bump)
git commit -m "docs: add installation instructions"
```

### Pros
- ✅ Automatic version management
- ✅ Semantic versioning compliance
- ✅ Clear commit history
- ✅ No manual version updates needed

### Cons
- ❌ Requires conventional commit discipline
- ❌ More complex workflow
- ❌ Less direct control over versions

## Choosing a Strategy

### Use Manual Version Management if:
- You prefer explicit control over versions
- Your team doesn't follow conventional commits
- You want simple, predictable releases
- You have infrequent releases

### Use Semantic Release if:
- Your team follows conventional commits
- You want automated version management
- You have frequent releases
- You want semantic versioning compliance

## Implementation

### Current Setup (Manual)
The project currently uses manual version management with `auto-release.yml`.

### Switching to Semantic Release
To switch to semantic release:

1. **Disable current workflow**:
   ```bash
   mv .github/workflows/auto-release.yml .github/workflows/auto-release.yml.disabled
   ```

2. **Enable semantic release**:
   ```bash
   # The semantic-release.yml is already created
   ```

3. **Update team guidelines** to use conventional commits

### Running Both (Not Recommended)
Running both workflows simultaneously is not recommended as they may conflict. Choose one strategy and stick with it.

## Commit Message Guidelines

If using semantic release, follow these guidelines:

### Types
- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools

### Examples
```bash
# Good conventional commits
feat(parser): add support for async functions
fix(compiler): resolve segmentation fault on Windows
docs(readme): add installation instructions
refactor(core): simplify error handling logic
perf(build): optimize compilation speed
test(parser): add unit tests for edge cases
chore(deps): update rustpython-parser to 0.4.1

# Breaking changes
feat(api)!: redesign command-line interface
feat(core): remove deprecated functions

BREAKING CHANGE: The old API has been removed
```

## Migration Guide

### From Manual to Semantic
1. Ensure all team members understand conventional commits
2. Update contribution guidelines
3. Switch workflows as described above
4. Start using conventional commit messages

### From Semantic to Manual
1. Switch workflows back
2. Resume manual version management
3. Update team guidelines

## Best Practices

1. **Be consistent**: Choose one strategy and stick with it
2. **Document your choice**: Make it clear which strategy you're using
3. **Train your team**: Ensure everyone understands the chosen workflow
4. **Monitor releases**: Regularly check that releases are working as expected
