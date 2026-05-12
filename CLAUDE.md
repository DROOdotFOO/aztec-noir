# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aztec Noir (codename "Zoir") is a Zed editor extension providing Noir language support, focused on Aztec network development. It wires `nargo` LSP into Zed and references an external tree-sitter grammar. Published in the Zed registry as `aztec-noir`; the repository, Rust crate, and internal types keep the `zoir` name.

## Repository Boundaries

This repository contains only the Zed extension. Two related concerns live elsewhere:

- **Tree-sitter grammar** -- `DROOdotFOO/tree-sitter-noir` (referenced by SHA in `extension.toml`). The `grammars/` directory is `.gitignore`d locally.
- **`nargo` language server** -- `noir-lang/noir` (downloaded at runtime when not on PATH).

If you need to edit the grammar (test corpus, conflicts, node names), do that work in `DROOdotFOO/tree-sitter-noir` and bump the `rev` in `extension.toml`. See [Working on the Grammar](#working-on-the-grammar) below.

## Key Commands (this repo)

```bash
# One-time: add WASM target for building
rustup target add wasm32-wasip1

# Build extension for Zed
cargo build --release --target wasm32-wasip1

# Check Rust compilation
cargo check --target wasm32-wasip1
```

## Architecture

### LSP Integration (src/lib.rs)

Four-tier nargo binary discovery (in `language_server_command` / `language_server_binary_path`):

1. `lsp.nargo.binary.path` from Zed settings (explicit override; errors if the file is missing)
2. PATH lookup via `worktree.which("nargo")` -- respects noirup installations
3. Cached binary path from previous download
4. GitHub release download from `noir-lang/noir`

The extension also honors `lsp.nargo.binary.arguments` (appended after the built-in `lsp` subcommand) and `lsp.nargo.binary.env` (passed through to the spawned process). These are the structured `CommandSettings` fields from `zed_extension_api`; do NOT reintroduce ad-hoc `settings.get("args")` JSON reads -- the reviewer of PR zed-industries/extensions#4787 explicitly asked for the structured form.

Platform asset mapping:

- macOS ARM64: `nargo-aarch64-apple-darwin.tar.gz`
- macOS x86: `nargo-x86_64-apple-darwin.tar.gz`
- Linux ARM64: `nargo-aarch64-unknown-linux-gnu.tar.gz`
- Linux x86: `nargo-x86_64-unknown-linux-gnu.tar.gz`
- Windows: Not available (noir-lang doesn't provide Windows binaries; users must build from source)

### Query Files

Zoir's editor queries live in `languages/noir/` (config, brackets, outline, indents, textobjects, runnables). These are read by Zed.

Highlights/locals/injections queries live in the upstream grammar repo under `queries/` and travel with the grammar `rev`.

### Versioning

`extension.toml` is the published version (Zed reads this). `Cargo.toml` must match. CI enforces this in `.github/workflows/ci.yml`.

## Working on the Grammar

The grammar is a separate project. To change it:

1. Clone `DROOdotFOO/tree-sitter-noir` outside this repo
2. Edit `grammar.js`, regenerate (`npx tree-sitter generate`), test (`npm test`)
3. Tag a release in that repo
4. Update the `rev` in this repo's `extension.toml`
5. Bump zoir's version (both `extension.toml` and `Cargo.toml`)

Reference notes about the upstream grammar (current as of last sync; verify in the upstream repo before relying on them):

- Keywords: `fn`, `struct`, `enum`, `impl`, `mod`, `use`, `let`, `mut`, `pub`, `for`, `if`, `else`, `match`, `return`, `global`, `comptime`, `unconstrained`
- Types: `Field`, `bool`, `u8`-`u128`, `i8`-`i128`, `str`, arrays, tuples, generics
- ZK-specific: `assert`, `assert_eq`, `constrain`, `#[recursive]`, parameter visibility (`pub`)
- Conflicts handled: type vs expression in generics, `mut` pattern vs let, `self` keyword, unary `&`/`*`
- Visibility split: `visibility_modifier` (items, supports `pub(crate)`/`pub(super)`) vs `parameter_visibility` (just `pub`, avoids tuple-type ambiguity)
- Const generics use `let` syntax: `<T, let N: u32>`; turbofish required for generic method calls (`foo.bar::<T>()`)
- Known quirk: `Field` parses as `type_identifier` (matches `/[A-Z][a-zA-Z0-9_]*/`), not `primitive_type` -- benign for highlighting

## Known Limitations

- No checksum/signature verification on the downloaded `nargo` binary. Zed's `download_file` API does not currently surface a hash parameter; track upstream API additions and add verification once available. Tracked in [#2](https://github.com/DROOdotFOO/aztec-noir/issues/2).
- No nargo version pinning via Zed settings -- always pulls latest GitHub release when downloading. Users who need a specific version should install it themselves and point at it via `lsp.nargo.binary.path`.
- Cleanup pass uses `fs::read_dir(".")` with a `nargo-` prefix match. Relies on Zed handing the extension a dedicated working directory.
- No `aztec-nargo` detection yet. The Aztec ecosystem ships its own forked nargo with Aztec-specific macro support; we currently only discover plain `nargo`. Planned follow-up: prefer `aztec-nargo` on PATH when present, with `binary.path` as the user-facing escape hatch.

## Future: Code Folding

Zed does not yet support `folds.scm` (see [Issue #22703](https://github.com/zed-industries/zed/issues/22703)). When support is added, create `languages/noir/folds.scm` targeting block, struct_body, impl_body, trait_body, and match_expression nodes.

## References

- Aztec network: https://aztec.network
- Noir language docs: https://noir-lang.org/docs/
- Noir compiler: https://github.com/noir-lang/noir
- Upstream grammar: https://github.com/DROOdotFOO/tree-sitter-noir
- Zed extension API: https://github.com/zed-industries/zed/tree/main/crates/zed_extension_api
- Tree-sitter docs: https://tree-sitter.github.io/tree-sitter/

## License

MIT OR Apache-2.0
