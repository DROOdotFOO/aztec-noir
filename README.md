# Zoir

[![Zed Downloads](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.zed.dev%2Fextensions%3Ffilter%3Dzoir&query=%24.data%5B0%5D.download_count&label=downloads&color=ff7edb&logo=data:image/svg%2Bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxNiAxNiI+CiAgPGRlZnM+CiAgICA8bGluZWFyR3JhZGllbnQgaWQ9InN1biIgeDE9IjAlIiB5MT0iMCUiIHgyPSIwJSIgeTI9IjEwMCUiPgogICAgICA8c3RvcCBvZmZzZXQ9IjAlIiBzdG9wLWNvbG9yPSIjZjk3ZTcyIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iNTAlIiBzdG9wLWNvbG9yPSIjZmY3ZWRiIi8+CiAgICAgIDxzdG9wIG9mZnNldD0iMTAwJSIgc3RvcC1jb2xvcj0iI2ZlZGU1ZCIvPgogICAgPC9saW5lYXJHcmFkaWVudD4KICA8L2RlZnM+CiAgPCEtLSBTdW4gd2l0aCBzY2FubGluZXMgLS0+CiAgPGNpcmNsZSBjeD0iOCIgY3k9IjEwIiByPSI2IiBmaWxsPSJ1cmwoI3N1bikiLz4KICA8IS0tIEhvcml6b24gY3V0b2ZmIC0tPgogIDxyZWN0IHg9IjAiIHk9IjEwIiB3aWR0aD0iMTYiIGhlaWdodD0iNiIgZmlsbD0iIzI2MjMzNSIvPgogIDwhLS0gU2NhbmxpbmVzIHRocm91Z2ggc3VuIC0tPgogIDxyZWN0IHg9IjIiIHk9IjUiIHdpZHRoPSIxMiIgaGVpZ2h0PSIxIiBmaWxsPSIjMjYyMzM1Ii8+CiAgPHJlY3QgeD0iMiIgeT0iNyIgd2lkdGg9IjEyIiBoZWlnaHQ9IjAuOCIgZmlsbD0iIzI2MjMzNSIvPgogIDxyZWN0IHg9IjIiIHk9IjkiIHdpZHRoPSIxMiIgaGVpZ2h0PSIwLjYiIGZpbGw9IiMyNjIzMzUiLz4KICA8IS0tIEdyaWQgbGluZXMgYmVsb3cgaG9yaXpvbiAtLT4KICA8bGluZSB4MT0iMCIgeTE9IjEyIiB4Mj0iMTYiIHkyPSIxMiIgc3Ryb2tlPSIjZmY3ZWRiIiBzdHJva2Utd2lkdGg9IjAuMyIgb3BhY2l0eT0iMC42Ii8+CiAgPGxpbmUgeDE9IjAiIHkxPSIxNCIgeDI9IjE2IiB5Mj0iMTQiIHN0cm9rZT0iI2ZmN2VkYiIgc3Ryb2tlLXdpZHRoPSIwLjMiIG9wYWNpdHk9IjAuNCIvPgo8L3N2Zz4K)](https://zed.dev/extensions?query=zoir)

Noir language support for [Zed](https://zed.dev), focused on [Aztec network](https://aztec.network) development.

## Aztec Focus

Zoir exists to give Aztec contract authors a first-class editor experience in Zed. Noir is the DSL Aztec uses for its private-by-default L2, and Zoir is scoped accordingly:

- Tracks Noir versions used by current Aztec releases
- Prioritizes editor support for the Aztec contract surface (notes, oracles, public/private functions)
- Tooling integration aimed at the `nargo` + Aztec CLI workflow

If you only need generic Noir editing without the Aztec slant, the [existing Noir extension](https://github.com/shuklaayush/zed-noir) covers that case.

## Features

- **Syntax highlighting** - Full highlighting for Noir keywords, types, operators, and ZK-specific constructs
- **Language server integration** - Powered by `nargo lsp` for diagnostics, hover, go-to-definition, and completions
- **Document outline** - Navigate functions, structs, traits, and modules
- **Bracket matching** - Including generic angle brackets
- **Auto-indentation** - Smart indentation for blocks and expressions
- **Vim text objects** - Function, class, parameter, and block selections
- **Runnable detection** - Detect `#[test]` functions and `main` for one-click execution

## Installation

### From Zed Extensions

1. Open Zed
2. Press `Cmd+Shift+X` (or `Ctrl+Shift+X` on Linux)
3. Search for "Zoir"
4. Click Install

### Manual Installation

Clone this repository into your Zed extensions directory:

```bash
git clone https://github.com/Hydepwns/zoir ~/.config/zed/extensions/zoir
```

## Requirements

### Language Server (nargo)

The extension automatically manages the `nargo` LSP binary:

1. **PATH lookup** - Uses `nargo` from PATH if available (respects [noirup](https://noir-lang.org/docs/getting_started/installation/) installations)
2. **Automatic download** - Downloads from GitHub releases if not found (macOS/Linux only)

#### macOS / Linux

Install nargo with noirup:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
noirup
```

#### Windows

Noir does not provide pre-built Windows binaries. You must build from source:

1. Install [Rust](https://rustup.rs/)
2. Clone and build nargo:
   ```powershell
   git clone https://github.com/noir-lang/noir
   cd noir
   cargo build --release -p nargo
   ```
3. Add `target\release` to your PATH

## Configuration

Configure the language server in your Zed settings (`~/.config/zed/settings.json`):

```json
{
  "lsp": {
    "nargo": {
      "settings": {
        "args": []
      }
    }
  }
}
```

### Running Tests

To enable one-click test running, add task templates to `~/.config/zed/tasks.json`:

```json
[
  {
    "label": "Noir: Run Test",
    "command": "nargo",
    "args": ["test", "--exact", "$ZED_CUSTOM_run"],
    "tags": ["noir-test"]
  },
  {
    "label": "Noir: Execute",
    "command": "nargo",
    "args": ["execute"],
    "tags": ["noir-main"]
  }
]
```

After adding these, you'll see run indicators next to `#[test]` functions and `main`.

## Noir Language

[Noir](https://noir-lang.org/) is a domain-specific language for zero-knowledge proofs. It enables writing private applications and verifiable computations.

Example:

```noir
fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[test]
fn test_main() {
    main(1, 2);
}
```

## Tree-sitter Grammar

Syntax highlighting is powered by [tree-sitter-noir](https://github.com/Hydepwns/tree-sitter-noir).

## Development

### Build the extension

```bash
cargo build --release --target wasm32-wasip1
```

## License

MIT OR Apache-2.0
