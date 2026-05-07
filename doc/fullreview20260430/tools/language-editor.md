# language and editor review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `nepl-language/src/lib.rs`
- `nepl-lsp/src/main.rs`
- `editors/zed/**`
- `web/src/language/**`
- `tests/playground_editor/**`
- `nodesrc/test_editor_diagnostic_code_contract.js`

## 良い点

`nepl-language` は editor 向けの Rust native analysis library として、lex、name resolution、semantics、diagnostics、token hints、hover doc の元情報を提供している。`nepl-web` とは別に Rust 側 analysis を共有できる入口がある。

`nepl-lsp` は `nepl-language` を使って LSP の initialize、didOpen/didChange/didSave、diagnostics、hover、definition、semantic tokens、inlay hints を実装している。stdlib root は initializationOptions / env / repo root から解決できる。

Zed extension は `nepl-lsp` を起動し、`NEPL_STDLIB_ROOT` を渡す thin wrapper になっている。editor 固有層が薄いことは良い。

tree-sitter grammar、highlight、indent、bracket 設定も存在し、syntax highlight と LSP analysis の二層構成がある。

## 問題とリスク

`editors/zed/target` が 945 件 tracked されている。これは generated Cargo artifact が source tree に混ざる問題であり、`ISS-20260507T153515441Z-ZED-EXTENSION-BUILD-ARTIFACTS-ARE-TR-B7D814F1` として issue 化した。

`nepl-language/src/lib.rs` は analysis model、name resolution trace、semantic trace、diagnostic conversion、tests を 1 ファイルに持つ。editor API が拡張されるほど肥大化し、compiler stage ごとの責務境界が見えにくくなる。

Zed extension は独立 crate で、root workspace の CI と同じ gate に入っていない。README には現行環境の Cargo 1.83 では `zed_extension_api` の edition2024 依存で build 検証できない旨がある。tracked target artifacts と合わせて、reproducible build の状態が曖昧である。

tree-sitter grammar は現行 NEPLg2 parser の完全な syntax authority ではない。char literal、new directives、type syntax、raw block 追加時に Rust parser / web analysis / tree-sitter の差分が出る可能性がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `nepl-language` | Rust native analysis library。 | 良いが file 分割余地あり。 |
| `nepl-lsp` | hover/definition/semantic tokens/inlay hints。 | 実装あり。integration CI は限定的。 |
| `editors/zed` | thin extension + grammar。 | source はあるが target artifacts tracked。 |
| tree-sitter grammar | highlight 用 parser。 | Rust parser との drift 注意。 |
| web language provider | browser analysis provider。 | Rust LSP との contract 統一が必要。 |

## 推奨対応

- `ISS-20260507T153515441Z-ZED-EXTENSION-BUILD-ARTIFACTS-ARE-TR-B7D814F1` を解決し、tracked generated artifacts を除去する。
- `nepl-language/src/lib.rs` を lex/name/semantic/diagnostic/range conversion の module に分ける。
- Zed extension build を CI で検証できる toolchain 方針を決める。
- tree-sitter grammar と Rust parser の差分を source policy または fixture で検出する。
