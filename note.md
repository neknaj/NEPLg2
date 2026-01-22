# 状況メモ (2026-01-22)
- 版を NEPLG2 に合わせて大規模再実装中。`nepl-core` に新しい lexer/parser/typecheck/codegen を追加し、旧 NEPLG1 資産は未使用。
- `plan.md` 以外の設計資料は NEPLG1 のまま。`plan2.md` と `doc/starting_detail.md` は存在しない。
- ツールチェインが edition 2024 を扱えなかったため、ワークスペースと各クレートの edition を 2021 に変更。
- 依存クレートを取得できず (`index.crates.io` へ DNS 解決不可)、`cargo test` は未実行。

# これからの作業方針
- 依存取得後に `cargo test --workspace --locked` を実行して型／コード生成の不足を洗う（ネット必要）。
- #wasm ブロックの検査強化（スタック整合チェックなど）は未着手。必要なら lexer/parser/typecheck に追加する。
- stdlib は NEPLG2 用に `stdlib/std.nepl` を最小構成(add/sub/lt/print_i32)で再作成済み。旧 NEPLG1 資産は削除。
- CLI は wasmi 0.31 系に合わせて起動確認（実行時は `--output <file>` が必須）。 
