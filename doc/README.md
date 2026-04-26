# NEPLg2 / NEPLg3 ドキュメント

---

## 仕様書

| ディレクトリ | 内容 |
|---|---|
| [neplg2/](./neplg2/README.md) | NEPLg2.0 現行実装の設計・保守ドキュメント |
| [neplg3/](./neplg3/README.md) | NEPLg3 仕様・実装設計の入口 |
| [neplg3/spec/](./neplg3/spec/index.md) | NEPLg3 の現在の仕様書群（実装目標・正の仕様。凍結済みコア仕様と draft / 将来仕様の周辺領域を併記し、各章でステータスを明示） |
| [neplg3/impl/](./neplg3/impl/index.md) | NEPLg3 コンパイラ実装設計（ファイル構成・パイプライン・移行戦略） |
| [compare/](./compare/index.md) | NEPLg2.0 → NEPLg3 の変更点対照表 |
| [migration/](./migration/index.md) | stdlib / tests / tutorials の NEPLg3 移行計画（並行ディレクトリ戦略） |

## ツール・開発

> 以下は現行実装（NEPLg2.0、`nepl-core`）に対応したドキュメント。
> NEPLg3 実装計画は [neplg3/impl/](./neplg3/impl/index.md) を参照。

| ドキュメント | 内容 |
|---|---|
| [cli.md](./cli.md) | CLI コマンドリファレンス（NEPLg2.0 現行） |
| [lsp_api.md](./lsp_api.md) | Language Server Protocol API（NEPLg2.0 現行） |
| [editor_extensions.md](./editor_extensions.md) | エディタ拡張方針 |
| [llvm_ir_setup.md](./llvm_ir_setup.md) | LLVM IR セットアップ |
| [testing.md](./testing.md) | テスト（NEPLg2.0 現行） |
| [../issues/](../issues/README.md) | 新 Issue 管理（旧 review20260425 から移行済み） |
| [review20260425/](./review20260425/issues.md) | NEPLg2.0 現行実装レビュー Issue 台帳（履歴スナップショット） |
| [debug.md](./debug.md) | デバッグ |
| [web_playground.md](./web_playground.md) | Web Playground |
| [web_playground_editor_redevelopment_plan.md](./web_playground_editor_redevelopment_plan.md) | Web Playground editor 再開発計画 |
| [self_host.md](./self_host.md) | セルフホスト計画の入口 |
| [stdlib_doc_comment_policy.md](./stdlib_doc_comment_policy.md) | stdlib ドキュメントコメントポリシー |

## サンプルコード

| ディレクトリ | 内容 |
|---|---|
| [examples/](./examples/) | NEPLg3 コードサンプル（01_basics〜07_modules） |

## 履歴メモ

| ディレクトリ | 内容 |
|---|---|
| `chat/dump/` | 過去の検討メモ・会話ダンプ。現行仕様の正ではない。現在の仕様確認には `neplg3/spec/` と Zenn #1 / #2 を使うこと。 |

## 標準ライブラリ

> **注**: stdlib の詳細設計は [neplg3/spec/stdlib.md](./neplg3/spec/stdlib.md) を参照。
> NEPLg3 への stdlib 移行計画は [migration/index.md](./migration/index.md) を参照。
> `stdlib/` 配下の個別 API ドキュメントは実装と並行して整備予定。
