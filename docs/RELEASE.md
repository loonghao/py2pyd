# Release Process

This document describes the automated release process for py2pyd.

Releases are driven by [release-plz](https://github.com/release-plz/release-plz).
The authoritative configuration is [`release-plz.toml`](../release-plz.toml); the
workflows live in `.github/workflows/`.

## Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Pull requests to `main`, pushes to `main`/`develop`, nightly schedule | Clippy, test matrix (Linux/Windows/macOS), `rustfmt`, `cargo doc`, security audit, coverage |
| `release-plz.yml` | Push to `main` | Opens the version-bump PR, publishes to crates.io, creates the `v*` tag and the GitHub release |
| `release.yml` | Push of a `v*` tag | Cross-compiles binaries and attaches them to the existing GitHub release |

There is no `auto-release.yml` or `semantic-release.yml` in this repository.

## How a release happens

1. **Land conventional commits** on `main`. The commit types decide the next
   version — see [docs/VERSIONING.md](VERSIONING.md).

2. **Merge the release-plz PR.** The `release-plz-pr` job runs
   `release-plz release-pr`, which computes the next version, bumps `version` in
   `Cargo.toml`, regenerates `CHANGELOG.md`, and opens (or updates) a release PR.

3. **Publishing.** Once the bumped version is on `main`, the
   `release-plz-release` job runs `release-plz release`, which:
   - publishes the crate to crates.io using `CARGO_REGISTRY_TOKEN`;
   - creates the Git tag `v{{version}}` (from `git_tag_name` in
     `release-plz.toml`);
   - creates the GitHub release, non-draft, with the body template defined in
     `git_release_body`.

4. **Binary builds.** The `v*` tag triggers `release.yml`, which builds every
   matrix target and attaches the archives to the release that release-plz
   already created.

The `v` prefix on `git_tag_name` is load-bearing: `release.yml` only listens for
tags matching `v*`. Changing one without the other breaks binary publishing.

## Build matrix

Defined in `.github/workflows/release.yml`:

| Target | Runner | Build method |
|--------|--------|--------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | `taiki-e/setup-cross-toolchain-action` |
| `x86_64-unknown-linux-musl` | `ubuntu-22.04` | `cross` |
| `aarch64-unknown-linux-gnu` | `ubuntu-22.04` | `cross` |
| `aarch64-unknown-linux-musl` | `ubuntu-22.04` | `cross` |
| `x86_64-apple-darwin` | `macos-13` | native |
| `aarch64-apple-darwin` | `macos-14` | native |
| `x86_64-pc-windows-msvc` | `windows-2022` | native |
| `aarch64-pc-windows-msvc` | `windows-2022` | native |

## Release artifacts

`release.yml` uses `taiki-e/upload-rust-binary-action` with `tar: all`,
`zip: windows` and `checksum: sha256`, producing per target:

- `py2pyd-<target>.tar.gz` — every target
- `py2pyd-<target>.zip` — Windows targets only
- `py2pyd-<target>.sha256` — every target

Static linking is applied only to some targets:

- Windows MSVC jobs set `RUSTFLAGS=-C target-feature=+crt-static` in
  `release.yml`.
- `x86_64-unknown-linux-musl` is statically linked by that target's default;
  `release.yml` sets no `RUSTFLAGS` for it.
- The `*-linux-gnu` and `*-apple-darwin` targets link dynamically.

Not every matrix entry produces a published asset in every release. For example
`aarch64-unknown-linux-musl` and `x86_64-apple-darwin` are absent from both
[v0.1.5](https://github.com/loonghao/py2pyd/releases/tag/v0.1.5) and
[v0.1.6](https://github.com/loonghao/py2pyd/releases/tag/v0.1.6). Check the job
log for the target in the Actions tab if an expected asset is missing.

## Secrets

| Secret | Used by | Purpose |
|--------|---------|---------|
| `GITHUB_TOKEN` | all workflows | Default token; `release.yml` needs `contents: write` to attach assets |
| `CARGO_REGISTRY_TOKEN` | `release-plz.yml` | Publishing to crates.io |
| `RELEASE_PLZ_TOKEN` | `release-plz.yml` | Optional PAT; falls back to `GITHUB_TOKEN` |

`RELEASE_PLZ_TOKEN` matters for step 4: tags created with the default
`GITHUB_TOKEN` do not trigger other workflows, so without a PAT `release.yml`
never runs and the release ships without binaries. `release-plz.yml` already
uses `${{ secrets.RELEASE_PLZ_TOKEN || secrets.GITHUB_TOKEN }}`.

## Manual re-release

`release.yml` only reacts to tag pushes. To rebuild binaries for an existing
tag, re-run the workflow from the Actions tab against that tag, or delete and
re-push the tag.

## Troubleshooting

**No release after merging the release PR**
- The version in `Cargo.toml` may already be published on crates.io.
- The merged commits may contain no bump-triggering type (`docs:`, `chore:` and
  friends do not bump — see [docs/VERSIONING.md](VERSIONING.md)).

**Nothing happens on push to `main` at all**
- Both release-plz jobs are gated on
  `if: github.repository_owner == 'loonghao'`, so forks never release.

**Tag created, but no binaries attached**
- Usually `RELEASE_PLZ_TOKEN` is unset, so the `GITHUB_TOKEN`-created tag did
  not trigger `release.yml`. See [Secrets](#secrets).

**release-plz refuses to publish**
- `semver_check = true` runs `cargo-semver-checks` against the published crate.
  A breaking API change without a `!` commit still forces a bump; add a
  `feat!:` prefix or a `BREAKING CHANGE:` footer so the changelog matches.

**Build failures for one target**
- Open the failing job in the Actions tab. `release.yml` sets
  `fail-fast: false`, so other targets still publish.
