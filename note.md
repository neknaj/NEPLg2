# 状況メモ (2026-01-22)
- 言語に文字列リテラルを追加。型 `str` を追加し、文字列は線形メモリ上に `[len][bytes]` で配置するポインタ(i32)として扱う。WASM側でメモリを常時生成・エクスポートし、データセクションに文字列を配置。
- `#extern` で外部関数を宣言可能にし、stdlib から `env.print_i32` / `env.print_str` を import する構成に統一。コード生成は extern を import として埋め込み、ビルトイン関数は完全撤廃。
- CLI: `--target wasm|wasi` に対応（wasi は wasm を包含）、`--output` は任意、`--run` だけでも実行可能。埋め込みランナーは wasm だけを許可し、`print_str` 用にメモリを参照するホスト関数を登録。
- stdlib: `std/stdio` に `print_str` を追加（`print_i32` と同様に import）。`std/math`/`std.nepl` は NEPLG2 用の最小構成のまま。examples（counter/fib）を文字列出力を含む形に更新。
- README を最新仕様（ビルトインなし／std import 前提／print_str 追加）に更新。
- `nepl-core` HIR に文字列リテラルと string table を追加。コード生成でメモリ・データセクションを生成し、全モジュールでメモリをエクスポート。型システムに `str` を追加し、型注釈/extern シグネチャ/型推論が対応。
- テスト: `nepl-core/tests/neplg2.rs` に string_literal_compiles を追加。既存ターゲットゲートテストを wasi/wasi包含仕様に合わせて維持。
- `:`ブロックと`;`の仕様に合わせて型検査を修正（Unitの暗黙破棄・`;`は値を捨てるだけで式本体は維持）。`while` のWASM生成を block+loop の2段構造にし、構造化制御のラベル深さを修正。examples の while 本体を `;`付きの行末とし、最後を `()` で閉じる形に整備。
- Loader/SourceMap を導入し、ファイルごとに FileId を割り当てる形へ移行。`load_inline` でも import/include が解決されるよう統一し、`resolve_path` で拡張子補完を実施。
- CLI で CoreError::Diagnostics を受け取った際に SourceMap を使ってファイル名・行・桁・キャレット付きで表示する簡易レンダラを追加。compile 失敗時に診断を出力して終了する挙動に変更。
- パイプ演算子 `|>` を追加。スタックトップの値を次の呼び出しの第1引数として注入する仕様で、lexer/parser/typecheck/テストまで実装。
- `std/std/mem.nepl` を追加し、`load_u8/store_u8/load_i32/store_i32/memory_grow/alloc` を #wasm で提供。#wasm 命令セットに i32.load/store・memory.grow 等を追加してスタック検査を拡張。
- `std/std/math.nepl` を拡充（add/sub/mul/div_s/mod_s/lt/eq/le）。
- `std/std/string.nepl` を Result ベースに改修（slice/to_i32/find は err コード付きの Result を返す）。eq/concat/len も整備済み。
- `std/std/result.nepl` を実装（[tag,val] 表現で ok/err/is_ok/is_err/unwrap_or）。
- `std/std/option.nepl` を追加（none/some/is_* / unwrap_or）。
- `std/std/stdio.nepl` を wasi 専用に再構成し、`fd_write` import で print_str/print_i32 を実装。wasm では未提供。
- `std/std/list.nepl` を追加（i32 専用の簡易 new/len/push/get。多相化は未対応）。

# これからの作業方針
- list の多相化と Option/Result を型システムで正式サポートする（現在はポインタ表現のみ）。
- import/use の厳密化（モジュール境界保持、衝突検出、値/型名前空間分離、hoisting 整理）。
- string/list/mem/stdio の境界テストを追加（特に wasi で fd_write import があることを確認）。
- wasm target で stdio を使った場合にコンパイル時に明確なエラーを出す仕組み。
