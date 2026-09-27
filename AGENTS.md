# AGENTS.md — py2pyd

> Rust CLI and library that compiles Python modules into native extension
> modules (`.pyd` on Windows, `.so` on Linux/macOS) by driving Cython through a
> generated `setup.py` inside an isolated `uv` environment.
> Navigation map for AI agents, not a reference manual. Follow the links; do
> not read everything up front.

## Build & test

No justfile in this repo — use cargo directly. This is a single crate with both
a `[lib]` and a `[[bin]]`.

```bash
cargo build                   # debug build
cargo test                    # unit + integration tests
cargo test --all-features     # includes the optional `mimalloc` feature
cargo build --release         # statically linked, zero-dependency binary
cargo fmt --check             # rustfmt gate (CI job "Rustfmt")
cargo clippy -- -D warnings   # lint gate (CI job "Clippy", see clippy.toml)
RUST_LOG=debug cargo run -- compile input.py   # debug a real compilation
```

Full local equivalent of CI:

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test --all-features
```

Cross-compilation is configured through `Cross.toml` (with `patches/` for
`libmimalloc-sys`) and exercised by `test-cross-compile.ps1`. The `mimalloc`
feature is **opt-in** and native-build only — never enable it for cross builds.

## Repo layout

| Path | Role |
|---|---|
| `src/lib.rs`, `src/main.rs` | Library API and CLI entry point |
| `src/python_env/` | Interpreter discovery: PATH lookup, `uv` selection, explicit path |
| `src/uv_env.rs`, `src/uv_compiler.rs` | Isolated `uv` environment and the Cython compile driver |
| `src/compiler/`, `src/parser/`, `src/transformer/` | Codegen, `rustpython-parser` AST parsing, source transforms |
| `src/build_tools.rs`, `src/batch_outcome.rs` | Toolchain probing and per-file batch results |
| `src/turbo_downloader.rs` | Accelerated download of toolchain artifacts (`turbo-cdn`) |
| `tests/` | Integration tests: `e2e_compilation_test.rs`, `batch_contract_test.rs`, `pip_download_test.rs`, `parser_test.rs`, `transformer_test.rs`, plus `fixtures/` |
| `examples/` | `math_utils.py` (sample input), `turbo_cdn_test.rs`, `test_runner.rs` |
| `docs/` | `RELEASE.md`, `VERSIONING.md` |
| `scripts/` | `release.sh`, `release.ps1` manual release helpers |
| `release-plz.toml`, `Cross.toml`, `clippy.toml` | Release, cross-compile and lint configuration |

## Release

- **release-plz** (not release-please) drives versioning and changelog from
  Conventional Commits on `main` — see `release-plz.toml`. `feat:` → minor,
  `fix:` → patch, `chore:`/`docs:`/`ci:` → **no release**.
- Use `chore:`/`docs:` for config and doc work so no valueless version is cut.
- Flow: release-plz bumps `Cargo.toml`, updates `CHANGELOG.md`, creates the
  `v<version>` tag and GitHub release, and publishes to crates.io. That tag
  triggers `.github/workflows/release.yml`, which builds per-platform binaries
  with `upload-rust-binary-action` and attaches them to the release.
- `docs/RELEASE.md` and `docs/VERSIONING.md` are the detailed procedure — read
  them before doing anything release-related.

## Do / Don't

- **Do** run compilation work through the `uv` environment helpers in
  `src/uv_env.rs` — builds must stay isolated from the host interpreter.
- **Do** add an integration test under `tests/` for each new compiler or
  parser behaviour; the suite already covers e2e compilation and pip packages.
- **Do** keep `semver_check = true` in `release-plz.toml` in mind: breaking
  library API changes need a major-version signal in the commit message.
- **Don't** enable the `mimalloc` feature in cross-compilation builds.
- **Don't** hand-edit `Cargo.toml` version — release-plz owns it.
- **Don't** add `CLAUDE.md` / `GEMINI.md` / `CURSOR.md` / `ANTHROPIC.md` /
  `OPENAI.md` / `COPILOT.md` / `CODEBUDDY.md` / `.cursorrules` / `.clinerules` /
  `.windsurfrules` at the root. This file is the only agent contract file.
- **Don't** hardcode an exact version in tests (`assert_eq!(VERSION, "X.Y.Z")`)
  — release bumps will break it. Compare semantically or read Cargo metadata.
- **Don't** commit build artifacts to the repo root (`target/`, `*.pyd`,
  `*.so`, generated `setup.py`, `clippy_check.txt`).
