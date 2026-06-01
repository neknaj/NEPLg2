# NEPLg2 ドキュメント

---

## 仕様書

| ディレクトリ | 内容 |
|---|---|
| [neplg2/](./neplg2/README.md) | 現行 Rust 実装としての NEPLg2 設計・保守ドキュメント |
| [neplg2/neplg21_syntax_migration_plan.md](./neplg2/neplg21_syntax_migration_plan.md) | NEPLg2.1 表層構文移行計画 |
| [neplg3/](./neplg3/README.md) | NEPLg3 仕様・実装設計の参考入口。未着手・未確定であり現在の正仕様ではない |
| [neplg3/spec/](./neplg3/spec/index.md) | NEPLg3 draft 仕様書群。NEPLg2.1 実装では参考扱い |
| [neplg3/impl/](./neplg3/impl/index.md) | NEPLg3 コンパイラ実装設計 draft |
| [compare/](./compare/index.md) | NEPLg2.0 → NEPLg3 の過去比較資料。NEPLg2.1 の正仕様ではない |
| [migration/](./migration/index.md) | NEPLg3 移行計画 draft。NEPLg2.1 の移行計画ではない |

## ツール・開発

> 以下は現行実装（NEPLg2、`nepl-core`）に対応したドキュメント。
> NEPLg2.1 移行中は [neplg2/neplg21_syntax_migration_plan.md](./neplg2/neplg21_syntax_migration_plan.md) を優先し、NEPLg3 文書は参考に留める。

| ドキュメント | 内容 |
|---|---|
| [cli.md](./cli.md) | CLI コマンドリファレンス（NEPLg2 現行） |
| [lsp_api.md](./lsp_api.md) | Language Server Protocol API（NEPLg2 現行） |
| [editor_extensions.md](./editor_extensions.md) | エディタ拡張方針 |
| [llvm_ir_setup.md](./llvm_ir_setup.md) | LLVM IR セットアップ |
| [neplg2/gui_standard_library_spec.md](./neplg2/gui_standard_library_spec.md) | GUI / TUI を共通 UI substrate として扱う標準ライブラリ仕様 |
| [neplg2/gui_tui_implementation_plan.md](./neplg2/gui_tui_implementation_plan.md) | `core/gui` / `alloc/gui` / `std/gui` / platform backend と既存 TUI 再設計の実装計画 |
| [testing.md](./testing.md) | テスト（NEPLg2 現行） |
| [../issues/](../issues/README.md) | 新 Issue 管理（旧 review20260425 から移行済み） |
| [review20260425/](./review20260425/issues.md) | NEPLg2 実装レビュー Issue 台帳（履歴スナップショット） |
| [debug.md](./debug.md) | デバッグ |
| [web_playground.md](./web_playground.md) | Web Playground |
| [web_playground_editor_redevelopment_plan.md](./web_playground_editor_redevelopment_plan.md) | Web Playground editor 再開発計画 |
| [self_host.md](./self_host.md) | セルフホスト計画の入口 |
| [stdlib_doc_comment_policy.md](./stdlib_doc_comment_policy.md) | stdlib ドキュメントコメントポリシー |

## サンプルコード

| ディレクトリ | 内容 |
|---|---|
| [examples/](./examples/) | draft サンプル。NEPLg2.1 へ構文同期するまで現在の正仕様ではない |

## 履歴メモ

| ディレクトリ | 内容 |
|---|---|
| `chat/dump/` | 過去の検討メモ・会話ダンプ。現行仕様の正ではない。 |

## 標準ライブラリ

> **注**: `stdlib/` は現行 NEPLg2 実装の標準ライブラリであり、NEPLg2.1 構文へ同一ディレクトリ内で移行する。
> NEPLg3 の stdlib 文書や migration 文書は、この移行の正ではない。
