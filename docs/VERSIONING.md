# Versioning

py2pyd has a single release strategy: [release-plz](https://github.com/release-plz/release-plz)
driven by [Conventional Commits](https://www.conventionalcommits.org/).

- Configuration: [`release-plz.toml`](../release-plz.toml)
- Workflow: [`.github/workflows/release-plz.yml`](../.github/workflows/release-plz.yml)
- Binary publishing: [`.github/workflows/release.yml`](../.github/workflows/release.yml),
  described in [docs/RELEASE.md](RELEASE.md)

This repository does not use `auto-release.yml` or `semantic-release.yml`, and
the version in `Cargo.toml` is never edited by hand — release-plz owns it.

## How it works

1. Conventional commits are merged into `main`.
2. The `release-plz-pr` job runs `release-plz release-pr`, which computes the
   next version, bumps `version` in `Cargo.toml`, regenerates `CHANGELOG.md`,
   and opens or updates a release PR.
3. Merging that PR puts the new version on `main`.
4. The `release-plz-release` job runs `release-plz release`, which publishes to
   crates.io, creates the `v{{version}}` tag, and creates the GitHub release.
5. The `v*` tag triggers `release.yml`, which builds and attaches the binaries.

## Version bump rules

The next version is the higher of two inputs: the bump implied by the merged
conventional commits, and the bump required by the semver check.

### From conventional commits

| Commit type | Bump at 1.0 and later | Bump at 0.x (current) |
|-------------|-----------------------|-----------------------|
| `feat:` | Minor — `1.1.0` → `1.2.0` | Minor — `0.1.6` → `0.2.0` |
| `fix:` | Patch — `1.1.0` → `1.1.1` | Patch — `0.1.6` → `0.1.7` |
| Any type with `!`, or a `BREAKING CHANGE:` footer | Major — `1.1.0` → `2.0.0` | Minor — `0.1.6` → `0.2.0` |
| `docs:`, `style:`, `refactor:`, `perf:`, `test:`, `chore:`, `ci:`, `build:` | No bump | No bump |

`Cargo.toml` currently declares `version = "0.1.6"`, so the 0.x column applies:
breaking changes bump the minor, not the major.

### From the semver check

`release-plz.toml` sets `semver_check = true` for the `py2pyd` package, so
release-plz also runs `cargo-semver-checks` against the version already on
crates.io. A breaking change in the public API forces a bump even when no commit
carried `!`. Mark such commits explicitly with `feat!:` or a `BREAKING CHANGE:`
footer so the changelog reflects the bump.

## Commit message format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Examples:

```bash
# Feature addition (minor bump)
git commit -m "feat: add support for Python 3.12"

# Bug fix (patch bump)
git commit -m "fix: handle edge case in file parsing"

# Breaking change
git commit -m "feat!: redesign command-line interface

BREAKING CHANGE: The --input flag is now required"

# Documentation (no bump)
git commit -m "docs: add installation instructions"
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes to the build system or dependencies
- **ci**: Changes to CI configuration
- **chore**: Other changes that don't modify source or test files

## Changelog

`changelog_update = true` is set at both workspace and package level, so the
release PR regenerates `CHANGELOG.md`. The header comes from the
`[changelog] header` value in `release-plz.toml`.

## Tag format

`release-plz.toml` sets `git_tag_name = "v{{version}}"`, and
`.github/workflows/release.yml` triggers on `push: tags: ["v*"]`. The `v` prefix
is what connects the two — do not change one without the other.
