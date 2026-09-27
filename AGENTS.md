# AGENTS.md — py2pyd

> Navigation map, not a reference manual. Follow the links; don't read
> everything upfront.

py2pyd is a Rust command-line tool that compiles Python modules to extension
modules (`.pyd` on Windows, `.so` on Linux/macOS). It resolves a Python
interpreter from PATH, `uv`, or an explicit path, and drives Cython through a
generated `setup.py` inside an isolated `uv` environment.

---

## Repository Contract

**Rust crate — run cargo at the repository root.**

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Release build | `cargo build --release` |
| Test | `cargo test` |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` |
| Format check | `cargo fmt --all --check` |

**Repository layout**

| Path | Role |
|------|------|
| `src/` | Rust crate sources |
| `tests/` | Integration tests |
| `examples/` | Example invocations |
| `docs/` | Documentation |
| `patches/`, `scripts/` | Cython patches and release helpers |
| `release-plz.toml` | Release automation configuration |

**Release flow** — `release-plz` drives `CHANGELOG.md` and the version in `Cargo.toml` from
Conventional Commit subjects; tagging and zero-dependency binary publishing
run in CI. Never edit `CHANGELOG.md` or a version string by hand.

**Prohibitions**

- Do not edit `CHANGELOG.md` or version strings manually.
- Do not add a second agent contract file at the repository root; `AGENTS.md` is the single source.
- Do not weaken `clippy` lints (`clippy.toml` is the crate policy) — CI builds with `-D warnings`.
- Do not assume a system Python exists at build time; the tool must resolve it from PATH, `uv`, or an explicit path.

---

## Agent Contract Files

`AGENTS.md` is the **only** agent contract file at the repository root. It is the
native instruction file for Codex, OpenCode, Cursor, GitHub Copilot, Windsurf,
Cline, Roo Code, Kiro, Trae, and Augment, and Claude Code falls back to it when
no `CLAUDE.md` exists — so do not add `CLAUDE.md`, `GEMINI.md`, `CURSOR.md`, or
any other vendor-specific variant.

**Gemini CLI exception:** Gemini CLI defaults its context file to `GEMINI.md`. To
make it read `AGENTS.md`, set `context.fileName` once in `~/.gemini/settings.json`:

```json
{
  "context": {
    "fileName": ["AGENTS.md", "GEMINI.md"]
  }
}
```
