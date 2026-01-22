# 状況メモ (2026-01-22)
- 言語に文字列リテラルを追加。型 `str` を追加し、文字列は線形メモリ上に `[len][bytes]` で配置するポインタ(i32)として扱う。WASM側でメモリを常時生成・エクスポートし、データセクションに文字列を配置。
- `#extern` で外部関数を宣言可能にし、stdlib から `env.print_i32` / `env.print_str` を import する構成に統一。コード生成は extern を import として埋め込み、ビルトイン関数は完全撤廃。
- CLI: `--target wasm|wasi` に対応（wasi は wasm を包含）、`--output` は任意、`--run` だけでも実行可能。埋め込みランナーは wasm だけを許可し、`print_str` 用にメモリを参照するホスト関数を登録。
- stdlib: `std/stdio` に `print_str` を追加（`print_i32` と同様に import）。`std/math`/`std.nepl` は NEPLG2 用の最小構成のまま。examples（counter/fib）を文字列出力を含む形に更新。
- README を最新仕様（ビルトインなし／std import 前提／print_str 追加）に更新。
- `nepl-core` HIR に文字列リテラルと string table を追加。コード生成でメモリ・データセクションを生成し、全モジュールでメモリをエクスポート。型システムに `str` を追加し、型注釈/extern シグネチャ/型推論が対応。
- テスト: `nepl-core/tests/neplg2.rs` に string_literal_compiles を追加。既存ターゲットゲートテストを wasi/wasi包含仕様に合わせて維持。

# これからの作業方針
- 文字列以外の型/命令（例: f32 演算や追加の wasm 命令）のスタック検査を拡充する場合は `parse_wasm_line`/`validate_wasm_stack` に命令効果を追加する。
- 追加の標準ライブラリ機能（乱数、ファイルI/O など）を入れる場合は `#extern` 経由で import し、CLI の Linker 側でホスト実装を増やす。
- WASI 対応の実行系（fd_write 等）を入れる場合は target=wasi のランナーを別途実装する。現状は wasm 専用ランナー。
