# NEPLg2.1 実装設計ドキュメント

最終更新: 2026-03-17

このディレクトリは NEPLg2.1 コンパイラの**実装設計**に関するドキュメントを置く。
言語仕様（`doc/2.1spec/`）とは異なり、Rust ブートストラップコンパイラの具体的なファイル構成・移行戦略を記述する。

---

## ドキュメント一覧

| ドキュメント | 内容 |
|---|---|
| [compiler_structure.md](./compiler_structure.md) | ブートストラップコンパイラ（`nepl-core-2.1`）・CLI・セルフホストのファイル構成設計。現行（NEPLg2.0）との対応表・Stage 1–6 移行戦略を含む |

---

## 関連ドキュメント

| ドキュメント | 内容 |
|---|---|
| `doc/self_host.md` | セルフホスト計画の概要（目的・二層構造・ブートストラップ手順） |
| `doc/2.1spec/compiler.md` | コンパイラの言語仕様側の設計（M1–M6 仕様達成マイルストーン） |
| `doc/2.1spec/overview.md` | NEPLg2.1 言語概要と設計原則 |
| `doc/migration/index.md` | stdlib / tests / tutorials の NEPLg2.1 移行計画（並行ディレクトリ戦略） |

---

## 設計の要点

- **パイプラインステージ = ディレクトリ階層**: 依存方向がディレクトリ構造から読み取れる。
- **1 ファイル 800 行以下**: 現行 `typecheck.rs`（8,871 行）を役割ごとに分割。
- **Resource IR 第一級配置**: `resource/` ディレクトリに ownership/borrow/region/drop 解析を集約。
- **セルフホストとの命名パリティ**: Rust 側ディレクトリ名が `stdlib/neplg2/` の NEPL モジュール名に対応。
- **並行開発**: 現行 `nepl-core` は維持したまま `nepl-core-2.1` を新規クレートとして開発し、Stage 6 で切り替え。
