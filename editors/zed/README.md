# Zed Extension Plan

## 目的

- Zed で NEPLg2 の syntax highlight / diagnostics / hover / definition / inlay hints を提供する。
- 解析本体は `nepl-language` を共有利用し、`nepl-web` には依存しない。

## 構成

- `nepl-language`
  - compiler 実装を再利用した editor 向け解析 lib
- `tree-sitter-neplg2`
  - syntax highlight 用 grammar
- `nepl-lsp` または同等の Rust binary
  - `nepl-language` を使って Zed / VSCode 共通の editor 機能を提供する
- `editors/zed`
  - Zed 固有の薄い package / 起動設定

## 現在

- `nepl-language` は追加済み。
- `nepl-lsp` は追加済みで、`cargo test -p nepl-lsp` は通る。
- Zed package shell も追加済みで、`editors/zed/Cargo.toml` は独立 crate として切り離した。
- ただし `zed_extension_api` は現行環境の Cargo 1.83 では `edition2024` 依存により build 検証できない。

## 次にやること

1. tree-sitter grammar を追加する。
2. `nepl-language` を使う Language Server binary を追加する。
3. Zed extension package からその binary を起動する。
4. Zed 側の build 検証用に Rust/Cargo を `edition2024` 対応版へ上げるか、互換のある `zed_extension_api` 世代を固定する。
5. VSCode extension も同じ binary を使う。
