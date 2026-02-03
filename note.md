# 状況メモ (2026-01-22)
# 2026-02-03 作業メモ (test 彩色/stdlib テスト調整/コンパイラ確認)
- stdlib/std/test.nepl の失敗メッセージを ANSI 赤色で表示するよう変更し、std/stdio の色出力を利用。
- stdlib/tests/error.nepl で `fail` の使用を避け、error_new 由来の診断が非空であることを確認する形に調整。
- stdlib/tests/cliarg.nepl/list.nepl/stack.nepl/vec.nepl/string.nepl/diag.nepl を更新し、失敗時のメッセージを明示するテストに整理。
- doc/testing.md の失敗時の表示説明を更新。
- コンパイラ確認: error::fail（callsite_span 経由）を含むテストで wasm 検証エラーが発生するため、std テスト側では該当経路を使わないようにして回避。Rust 側の callsite_span/codegen の相性は要調査。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (テスト失敗のメッセージ表示)
- stdlib/std/test.nepl を改修し、失敗時にメッセージを表示してから trap するよう変更した。
- stdlib/std/diag.nepl に diag_print_msg を追加し、Failure メッセージを表示できるようにした。
- stdlib/std/error.nepl の fail/context を callsite_span 付与に更新した。
- stdlib/tests/diag.nepl と stdlib/tests/error.nepl を強化し、文字列化や span の検証を追加した。
- doc/testing.md の assert 仕様を更新した。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (cliarg 追加)
- stdlib/std/cliarg.nepl を追加し、WASI args_sizes_get/args_get で argv を取得できるようにした。
- stdlib/tests/cliarg.nepl を追加し、範囲外/負の index が None になることを確認するテストを用意した。
- doc/testing.md の stdlib 一覧に std/cliarg を追記した。
- nepl-cli の WASI ランタイムに args_sizes_get/args_get を追加し、`--` 以降の引数を渡せるようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (cliarg 実引数テスト)
- stdlib/tests/cliarg.nepl を更新し、argv[1..] の値を検証するテストを追加した。
- nepl-cli の stdlib テスト実行で `--flag value` を argv に渡すよう変更した。
- doc/testing.md に stdlib テストが固定引数を渡す旨を追記した。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (stdlib コメント言語統一)
- stdlib/std/option.nepl と stdlib/std/result.nepl の英語コメント行を削除し、コメントが日本語のみになるよう統一。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (stdlib コメント/Option/Result 改修)
- stdlib/std の各ファイルに日本語コメント（ファイル概要/各関数の目的・実装・注意・計算量）を追加し、math.nepl は自動生成で関数コメントを挿入。
- list_tail を Option<i32> 返却に変更し、list_get の走査を unit になるよう調整（デバッグ出力も削除）。
- stdlib/tests/list.nepl を list_tail の Option 仕様に合わせて更新。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (import/resolve テスト拡充)
- nepl-core/tests/resolve.rs に default alias（相対/パッケージ）、selective 欠落名の扱い、merge open、visible map 優先順位（local/ selective/ open）を追加。
- nepl-core/src/module_graph.rs の unit テストに missing dependency/invalid import/duplicate export/non-pub import/ selective+glob re-export を追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (rpn 実行 + std/test 修正 + テスト実行)
- examples/rpn.nepl を `printf "3 4 +\n" | cargo run -p nepl-cli -- -i examples/rpn.nepl --target wasi --run` で実行し、REPL が結果を返して終了することを確認。
- stdlib/std/test.nepl の `assert_str_eq` を `if:` ブロック形式に修正し、`(trap; ())` の inline 1行式を排除してパーサエラーを解消。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (rpn import + diagnostics)
- examples/rpn.nepl の import を新仕様（`#import "..." as *`）へ更新。
- loader の parse でエラー診断がある場合は CoreError を返すようにし、構文エラーが型エラーに埋もれないよう修正。
- CLI の診断表示でキャレット長を行末に収め、巨大な ^ の出力を抑制。
- typecheck の簡易サマリ出力は verbose 時のみ表示するように変更。

# 2026-02-03 作業メモ (Windows path canonicalization for tests)
- module_graph の lib テストで path 比較が Windows の canonicalize 差分で失敗するため、root path を canonicalize して比較するよう修正。
- resolve.rs 側の ModuleGraph 参照テストも同様に canonicalize を適用し、クロスプラットフォームで一致するようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (resolve import tests fix)
- nepl-core/tests/resolve.rs のテスト用ソースを `:` ブロック形式に修正し、parser の期待するインデント構造に合わせた。
- selective glob（`name::*`）が open import に反映されることを確認するテストを追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (resolve/import test expansion)
- nepl-core/tests/resolve.rs を追加実装し、prelude 指令の解析、merge clause 保持、alias/open/selective の解決、open import の曖昧性診断、std パッケージ解決のテストを追加。
- nepl-core/tests/neplg2.rs に prelude/import/merge 指令の受理確認テストを追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (tests import syntax migration)
- nepl-core/tests と stdlib 配下の #import/#use を新仕様（`#import "..." as *`）へ統一し、#use を除去した。
- loader_cycle のテストは `#import "./a"`/`#import "./b"` に変更して相対 import の仕様に合わせた。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (selective re-export test)
- module_graph の pub selective re-export の挙動を確認するテストを追加（alias のみ公開され、元名や未選択の公開項目は再エクスポートされないことを検証）。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (pub import selective re-export)
- build_exports が ImportClause::Selective を考慮し、pub import の再エクスポート範囲を selective に限定できるようにした（glob は全件再エクスポート扱い）。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (module_graph import clause)
- module_graph の import/deps に ImportClause を保持するようにし、resolve が AST ではなく ModuleGraph の情報から import 句を参照する形へ変更。
- resolve の import 走査を整理し、deps の clause を直接使って alias/open/selective/merge を構築。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (pub #import / pub item)
- lexer で `pub #import` を認識し、`#import pub ...` へ書き換える処理を追加（`pub` 前置のディレクティブは #import のみ許可）。
- parser で `pub fn/struct/enum/trait/impl` をトップレベルで解釈できるようにし、`pub` が先頭に来ても正しく定義を読めるようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (rewrite plan doc)
- doc/rewrite_plan.md を現行コード確認に基づいて拡充し、後方互換なしの設計書+実装計画書として整理した（モジュールID/manifest、import clause、prelude、名前解決優先順位、型推論/単相化、WASM ABI、CLI/stdlib境界、実装ロードマップ、テスト方針）。
- 現行パイプラインは loader の AST スプライス方式のままで、module_graph/resolve の実装は未統合である点を計画内に明記。
- plan.md には manifest/新import文法/prelude/mergeの仕様や CLI/ABI 境界の整理が未記載のため、追記が必要。
- テスト: 以前は `module_graph::tests::builds_simple_graph_and_exports` が unknown token で失敗していたが、`pub #import`/`pub fn` 対応後に `cargo test` も成功。

## 直近の実装サマリ
- 文字列リテラルと型 `str` を追加し、データセクションに `[len][bytes]` で配置して常時メモリをエクスポートする形に統一。
- `#extern` で外部関数を宣言可能にし、stdlib から `print` / `print_i32` を提供する構成に統一。ビルトイン関数は撤廃。
- CLI: `--target wasm|wasi` に対応（wasi が wasm を包含）。`--run` だけでも実行可。コンパイル失敗時に SourceMap 付き診断を出力。
- Loader/SourceMap を導入し、import/include で FileId/Span を保持したまま多ファイルを統合。
- パイプ演算子 `|>` を追加。スタックトップを次の呼び出しの第1引数に注入する仕様で、lexer/parser/typecheck まで実装済み。
- `:` ブロックと `;` の型検査を調整し、Unit 破棄や while の stack 深さ検証を改善。
- stdlib: math/mem/string/result/option/list/stdio を追加・更新。mem は raw wasm、string/result/option はタグ付けポインタ表現、stdio は WASI fd_write 前提。
- `#target wasm|wasi` をディレクティブとして追加。CLI がターゲットを指定しない場合は #target をデフォルトに用い、複数 #target は診断エラーにした。wasi 含有ルールは従来通り。
- stdlib/std/stdio を WASI `fd_write` 実装に置き換え、env 依存を排除。print_i32 は from_i32 → fd_write で出力。
- 型注釈の「恒等関数」ショートカットを削除し、ascription のみで扱う前提に揃えた。`|>`+注釈の回りのテストを追加。
- std/mem.alloc を要求サイズから算出したページ数で memory.grow する形にし、固定1ページ成長を解消（ただしページ境界アロケータのまま）。
- CLI の target フラグを省略可能にし、#target / stdio 自動 wasi 昇格と整合するようにした。
- テスト追加: #target wasi デフォルト動作、重複 #target エラー、pipe+型注釈の成功ケース。
- 言語に struct/enum/match を追加。enum/struct を TypeCtx に登録し、コンストラクタを自動バインド（`Type::Variant` / `StructName`）。match は網羅性チェックと型整合チェックを行う。
- Option/Result を enum ベースに再実装（OptionI32/ResultI32）。string/find/to_i32/list/get などを Result/Option 返却に差し替え。list の get は ResultI32 で境界エラーを返す。
- codegen に enum/struct コンストラクタと match を追加（runtime 表現は [tag][payload]/構造体フィールドを linear memory 上に確保し、std/mem.alloc 呼び出しを前提）。
- pipe の注入タイミングを調整し、型注釈 `<T>` を挟んでも `|>` が正しく次の callable に注入されるようにした。追加テストで確認。
- Loader の循環 import 検出テストを追加（temp ディレクトリに a.nepl/b.nepl を生成しロードでエラーを確認）。

## plan.md との乖離・注意点
- `#target`: ディレクティブとしては実装済みだが、plan.md には未記載。エントリーファイル以外に書かれた場合の扱いなど仕様明記が必要。
- 型注釈 `<T>`: 恒等関数ショートカットは削除したが、plan.md には「関数と見做す」とあるので記述を更新する必要あり。
- stdlib/stdio: WASI `fd_write` 実装に置き換え済み。wasm で import した際の専用診断はまだ無いので、エラーメッセージ改善の余地あり。
- stdlib/mem.alloc: サイズに応じたページ成長に修正したが、ページ境界アロケータのまま。細粒度管理や free は未対応。
- Option/Result/list: enum/match が無いためタグ付きポインタの暫定実装。型システム統合や多相化は未着手。list は i32 固定で get の範囲外診断なし。

## 追加で気付いたこと
- Loader は FileId/Span を保持して diagnostics に活用できている。#include/#import は一度きりロードで循環検出あり。
- コード生成は wasm のみ。CompileTarget::allows は wasi が wasm を包含する形で gate 判定を実装。

# 2026-01-23 作業メモ
- Rust ツールチェインを rustup で導入し、依存クレートを取得できるようにした。
- #if 関連の unknown token を解消するため lexer の `* >` / `- >` を Arrow として許可するよう緩和した。
- stdlib の構築途中コードが多数コンパイルを塞いでいたため、一時的に std/string・std/list・std/stdio を最小機能のスタブ実装に差し替え（option.unwrap_or を削除して重複解消）。
- enum コンストラクタの codegen を修正（payload store のオペランド順と、結果ポインタをスタックに残すように変更）。これにより Option::Some/None が正しく値を返し、`match_option_some_returns_value` が通過。
- std/list.get は境界外を常に `ResultI32::Err 1` で返す単純実装にし、スタック不整合の診断を解消。現状 in-bounds 取得は未対応だがテスト想定（OOB エラー）には合致。
- 現在 `cargo test` は 23/23 すべて成功。残課題は stdlib 機能の肉付け（list.get の正実装、文字列/オプションの汎用化など）。

## 今後の対応案（実装はまだしない）
- `#target wasi|wasm` をディレクティブとして追加し、ファイル内のデフォルトターゲットを決定（CLI 指定があればそちらを優先）。`#if[target=...]` 評価にも使用。
- 型注釈の古い恒等関数特例を撤去し、注釈は構文要素としてのみ扱う旨を仕様に明記。
- stdio を WASI fd_write 実装に戻す／もしくは wasm target で import された場合にコンパイル時エラーを出す。
- mem.alloc の size 対応とページ再利用、list の多相化・境界チェック強化、Option/Result を enum/match 連携へ移行。

# 2026-01-30 作業メモ
- stdlib/std/string.nepl の to_i32 内で if: ブロックに誤って if eq ok 1: / else: が混入するインデントになっており、if-layout 解析が "too many expressions" になる状態だったため、if eq ok 1: ブロックを1段デデントし、else ブロックのインデントを整えて if-layout が正しく分解されるよう修正。
- これにより std/string の cond/then/else 未定義エラーと block stack エラーが解消。cargo test は全件通過、examples/counter.nepl を wasi 実行しても完走することを確認。
- 文字列リテラルが allocator のメタ領域と衝突していたため、codegen_wasm の文字列配置開始オフセットを 8 バイト（heap_ptr + free_list_head）に変更し、data section で free_list_head=0 を明示。併せて data section を常に出力して heap_ptr を初期化するよう修正。

# 2026-02-01 if/while テスト無限ループ対応
## 問題発見
- ifテストが16GB以上のメモリ使用となり、実行が停止する無限ループ問題を発見。
- パーサー側は`if` ブロック分解で正常に動作している（テスト通過確認）。
- 無限ループはタイプチェック段階で発生している模様。

## 原因特定と修正
- `apply_function()` の `if` ケースで、関数型 `(bool, T, T) -> T` の `result` 型変数が統一されていなかった。
- 2つのブランチ型を統一した後、その結果を `result` 型変数に統一する必要があった。
- 修正: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);` を追加し、結果型を関数の result 型パラメータと統一。
- 同じく `while` も同様の問題があったため、`let final_ty = self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit());` で修正。

## テスト実行結果
- 修正後、部分的にテストが成功開始（8個テスト確認: if_mixed_cond_then_block_else_block など）
- 残り7個のテストでメモリスパイク続行
  - 失敗テスト: if_a_returns_expected, if_b_returns_expected, if_c_returns_expected, if_d_returns_expected, if_e_returns_expected, if_f_returns_expected, if_c_variant_lt_condition
  - これらは全て `#import "std/math"` と `#use std::math::*` を含む

## 次のステップ
- 失敗しているテストの共通点は import/use ステートメント
- ローダー或いはモノモルファイゼーション段階での無限ループの可能性を調査中

- これにより WASI 実行時の print（文字列リテラル）の無出力／ゴミ出力が解消。stdout の回帰検出用に `nepl-core/tests/fixtures/stdout.nepl` を追加し、`nepl-core/tests/stdout.rs` と `run_main_capture_stdout` を実装。
- 文字列操作のテストとして `nepl-core/tests/stdlib.rs` に len(文字列リテラル) と from_i32→len を追加。`cargo test -p nepl-core --test stdlib --test stdout` で確認。
- plan2.md と doc/starting_detail.md はリポジトリ内に存在しないため、参照できない状態のまま。
- stdlib/std/stdio に `println` を追加し、`print` + 改行文字列で実装。`print`/`print_i32` はそのまま維持。
- stdlib/std/stdio の `print_str` を `print` に改名し、`println_i32` を追加。str は `print`/`println`、i32 は `print_i32`/`println_i32` を提供する形に整理。
- `nepl-core/tests/fixtures/println_i32.nepl` と stdout テストを追加し、`println_i32` が改行を出力することを確認。
- examples の逆ポーランド記法電卓 `examples/rpn.nepl` を文字列パース方式に拡張し、ASCII トークンを走査して数値/演算子を処理する形に更新。
- stdlib/std/stdio から std/string の import を外し、print は文字列ヘッダ長を直接読む形に変更。print_i32 は同一ファイル内で数値→文字列変換を行い、std/list との `len` 衝突を回避。
- stdlib/std/stdio に `read_all` を追加し、WASI の fd_read で標準入力を取り込めるようにした。CLI ランタイムにも fd_read 実装と stdin バッファを追加。
- stdin の動作確認用に `nepl-core/tests/stdin.rs` と `nepl-core/tests/fixtures/stdin_echo.nepl` を追加し、日本語入力のエコーもテストに含めた。
- CLI の fd_read をオンデマンド読み込みに変更し、起動時に stdin を read_to_end しないことで対話入力でもブロックしないように調整。
- stdlib/std/stdio に `read_line` を追加し、REPL 向けに改行までの読み取りを提供。stdin テストに `stdin_readline.nepl` と日本語ケースを追加。
- examples/rpn.nepl を REPL 形式に変更し、1行ごとの評価とエラーメッセージ表示に対応。`read_line` を使うため、対話入力でも評価できるようにした。
- examples/rpn.nepl に REPL 使い方のメッセージを追加し、PowerShell パイプ時の BOM を無視する簡易スキップ処理を入れて unknown token を回避。
- stdout 用の fixture とテストを追加し、`println` が `\n` を出力することを確認。README の std/stdio 説明も `println` と WASI `fd_write` に合わせて更新。
- stdout テストで wasi fd_read の import 未提供により instantiate 失敗していたため、`nepl-core/tests/harness.rs` の `run_main_capture_stdout` に fd_read スタブを追加。`cargo test -p nepl-core --test stdin --test stdout` は警告付きで成功し、`printf '14 5 6 + -' | cargo run -q -- -i examples/rpn.nepl --run --target wasi` で REPL 出力と結果 3 を確認。
- PowerShell の UTF-16LE パイプ入力で数値が分割される可能性に備え、`examples/rpn.nepl` の数値パースで NUL バイトを無視する分岐を追加（BOM スキップと併用）。

# 2026-01-30 作業メモ (テスト/stdlib)
- stdlib に `std/test` を追加し、`assert`/`assert_eq_i32`/`assert_str_eq`/`assert_ok_i32`/`assert_err_i32` を提供。`trap` は `i32.div_s` を 0 で割る #wasm で実装し、WASM 側で確実に異常終了するようにした。
- `std/string` に `str_eq`（純粋再帰）を追加し、`std/test` 側の文字列比較でも同等ロジックを使用。
- CLI に `nepl test` サブコマンドを追加し、`stdlib/tests` 配下の `.nepl` を収集して WASI で実行するテストランナーを実装。
- stdlib テストを `stdlib/tests/{math,string,result,list}.nepl` に追加。式の括弧は使わず前置記法で記述し、Result の move を避けるため同一値を再生成して検証。
- `cargo run -p nepl-cli -- test` と `cargo test` が通ることを確認。
- doc に `doc/testing.md` を追加し、テスト機能の使い方と stdlib の現状範囲を整理。

# 2026-01-30 作業メモ (examples 実行確認)
- examples/counter.nepl と examples/fib.nepl を `#target wasi` に揃え、std/stdio の利用を明示。
- `cargo run -p nepl-cli -- -i examples/counter.nepl --run --target wasi` と `... fib.nepl ...`、`printf '14 5 6 + -\n' | ... rpn.nepl ...` を実行し、出力が正常であることを確認。
- `cargo test` を再実行し、全テストが通過することを確認。

# 2026-01-30 作業メモ (多相/単相化の現状)
- パーサは fn/enum/struct/trait/impl の型パラメータ宣言と型適用 `TypeName<...>` を受理し、TypeCtx には TypeKind::{Function,Enum,Struct} の type_params と TypeKind::Apply がある。
- 関数呼び出しでは typecheck が type_params を fresh var に instantiate し、呼び出し側に type_args を残す。monomorphize は FuncRef の type_args をもとに関数だけ単相化してマングル名を生成する。
- TypeKind::Apply は unify が扱わず、resolve も match 以外で使われていないため、型注釈やシグネチャで `Foo<...>` を使うと実質的に整合しない。
- enum/struct のコンストラクタは定義側の型情報を直接使っており、instantiate された params/result を反映しないため型変数がグローバルに束縛されやすく、ジェネリック enum/struct が実用になっていない。
- stdlib の list/option/result は i32 固定で、ジェネリクスは未導入。

## plan.md との差分メモ (追加)
- plan.md にはテスト実行コマンドや `std/test`/`nepl test` の仕様が未記載。テスト設計の章立てを追加する必要がある。
- plan2.md と doc/starting_detail.md は引き続きリポジトリ内に存在しないため参照不可。
- plan.md では「定義での多相は扱わない」としているが、実装には type_params と monomorphize が存在する。仕様整合の追記が必要。

# 2026-01-30 作業メモ (ジェネリクス修正)
- 型パラメータは .T 形式のみ許可するように parser を更新し、<T> はエラーにした。
- Apply を unify で resolve して enum/struct の具体型と統合できるようにし、resolve の結果は型引数を type_params に保持するよう変更。
- enum/struct コンストラクタは instantiate 後の params/result を使うようにし、型変数のグローバル束縛を避ける形に修正。
- type_to_string は enum/struct の type_params を含めるようにして単相化マングルの衝突を避けた。
- codegen で Apply を参照型として扱い、enum の variant 解決を Apply にも対応。
- Rust テスト `nepl-core/tests/generics.rs` を追加し、fn/enum/struct のジェネリクスとエラーケースを検証。

# 2026-01-30 作業メモ (ジェネリクス修正の追加)
- parser のエラー診断が出ている場合は compile_wasm を失敗させるようにし、<T> を実際にエラー扱いにした。
- Apply の型引数数不一致は unify で失敗させ、型注釈の不一致として診断されるようにした。
- 型引数は typecheck と monomorphize で resolve_id により実体型へ正規化し、単相化後に Var が残らないようにした。
- wasm 生成後に wasmparser で検証し、無効 wasm を診断として返すようにした。

# 2026-01-30 作業メモ (ジェネリクス修正の追加2)
- 型注釈が未適用のまま let が先に簡約されるケースがあったため、pending_ascription がある間はその手前の関数を簡約しないよう guarded reduce を追加。
- type_args の resolve を引数 unify 後に行うようにし、単相化に Var が残らないように修正。

# 2026-01-30 作業メモ (ジェネリクス テスト拡張)
- generics.rs に .T 必須の enum/struct 定義エラー、payload の i32 演算検証、複数型パラメータ関数の単相化、型注釈不一致のエラーを追加。
- さらに、None の型決定、引数なしジェネリック関数の型決定、ジェネリック関数の委譲呼び出し、pipe 経由呼び出し、2型パラメータ enum の match、入れ子 Apply の payload・その不一致エラー、同一型パラメータの不一致エラー、payload 型不一致エラーを追加。
- 追加で、コンストラクタの型推論（引数位置）、ジェネリック関数での Pair 構築、Option::Some ラッパー関数、Option<Option<T>> の入れ子 match を OK ケースとして追加。

# 2026-01-31 作業メモ (ジェネリクス/構文/コード生成)
- if-layout の cond 識別子が変数名として使われるケースに対応するため、`normalize_then_else` で cond を無条件に消さず、then/else マーカーがある場合のみ除去するよう調整。
- `if cond:` のような行末 `:` 形式で cond が変数名の場合に stack エラーが出ていたため、if-layout 判定から `if cond:` の特例を外し、cond 変数を保持する形に変更。
- match 式が後続の行を吸い込むケースがあったため、`KwMatch` で match 式を読み込んだら prefix 解析を打ち切るように修正。
- wasm codegen の match が 2分岐固定だったため、任意個（1個以上）の分岐を if 連鎖で生成するように拡張し、1バリアント enum の match で unreachable が出る問題を解消。
- `generics_multi_type_params_function` の期待値は if の振る舞いに合わせて 3 に修正（false 分岐の確認）。
- `cargo test` は全件通過を確認。
- plan2.md と doc/starting_detail.md は引き続きリポジトリ内に存在しないため参照不可。

# 2026-01-31 作業メモ (テスト整合)
- nepl-core の `list_get_out_of_bounds_err` テストを現行 stdlib に合わせ、`list_nil/list_cons/list_get` と `Option` の `Some/None` マッチに更新。
- `cargo test` と `cargo run -p nepl-cli -- test` の両方が成功することを確認。

# 2026-01-31 作業メモ (ログ抑制)
- typecheck/unify/monomorphize/wasm_sig の成功時ログを削除し、OK時の `nepl-cli test` の出力を削減。
- `cargo run -p nepl-cli -- test` はテスト結果のみ表示されることを確認（Rust の警告は別途表示）。

# 2026-01-31 作業メモ (verbose フラグ)
- `nepl-cli` に `--verbose` を追加し、詳細なコンパイラログを必要時のみ出力できるようにした。
- `CompileOptions.verbose` で制御し、typecheck/unify/monomorphize/wasm_sig のログをフラグ連動にした。

# 2026-01-31 作業メモ (メモリアロケータ)
- `std/mem` の allocator を wasm モジュール内実装に変更し、`nepl_alloc` のホスト依存を除去。
- free list + bump 併用の簡易 allocator を実装し、`memory.grow` で拡張。
- `doc/runtime.md` に WASM/WASI のターゲット方針とメモリレイアウトを追加。

# 2026-01-31 作業メモ (nepl_alloc 自動 import の撤去)
- コンパイラが `nepl_alloc` を自動で extern に追加する処理を削除し、WASM 生成物がホスト依存の import を持たないようにした。
- `alloc`/`dealloc`/`realloc` は `std/mem` の定義か `#extern` により解決される前提になったため、モジュール側で `std/mem` を import していない場合は codegen でエラーになる。
- 既存の `a.wasm` などは再コンパイルが必要（古いバイナリには `nepl_alloc` import が残る）。
- `alloc` などのビルトイン自動登録も外したため、`std/mem` の関数定義がそのまま使用される。`alloc` を使うコードは `std/mem` を明示的に import する必要がある。

# 2026-01-31 作業メモ (std/mem の効果注釈)
- `std/mem` の `alloc`/`dealloc`/`realloc`/`mem_grow`/`store` を `*` 付きに変更し、純粋コンテキストから呼べないことを明示した。
- これにより `std/mem` 内部の `set`/`store_*` 呼び出しが純粋関数扱いになっていた問題を解消し、`match_arm_local_drop_preserves_return` の失敗原因を修正した。

# 2026-01-31 作業メモ (monomorphize のランタイム関数保持)
- エントリ起点の単相化で `alloc` が落ちる問題を避けるため、`monomorphize` の初期 worklist に `alloc`/`dealloc`/`realloc` を追加した。
- enum/struct/tuple の codegen が `alloc` を呼ぶ前提でも、未参照の `alloc` が除去されないようにした。

# 2026-01-31 作業メモ (テスト側の std/mem 明示)
- enum/struct/tuple を使うテストソースに `std/mem` の import を追加し、`alloc` が解決される前提を明確化した。
- `move_check` テストは Loader 経由で compile するように変更し、`#import` を解決できるようにした。

# 2026-01-31 作業メモ (標準エラー/診断の追加)
- `std/error` と `std/diag` を追加し、`ErrorKind`/`Error`/`Span` と簡易レポート生成を用意した。
- `callsite_span` の intrinsic を追加し、エラーに呼び出し位置を付与できるようにした。
- `std/string` に `concat`/`concat3` を追加し、診断文字列生成の最低限を実装した。

# 2026-01-31 作業メモ (WASI エントリポイント対応)
- codegen_wasm で entry 関数が指定されている場合、その関数を `_start` という名前でも export するようにした。
- これにより `wasmer run a.wasm` / `wasmtime run a.wasm` で WASI コンプライアンスに従い直接実行可能に。
- README.md に外部 WASI ランタイム（wasmtime/wasmer）での実行方法を追加。

# 2026-01-31 作業メモ (数値演算の完全化)
- stdlib/std/math.nepl を全面拡張し、i32/i64/f32/f64 のすべての演算機能を提供。
- **算術演算**：add/sub/mul/div_s/div_u/rem_s/rem_u（すべての型で符号別に提供）
- **ビット演算**：and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt（整数型のみ）
- **浮動小数点特有**：sqrt/abs/neg/ceil/floor/trunc/nearest/min/max/copysign（f32/f64）
- **型変換**：i32/i64 <-> f32/f64、符号付き/符号なし対応、飽和変換（trunc_sat）
- **ビット再解釈**：reinterpret_i32/f32/i64/f64

# 2026-02-03 作業メモ (web playground)
- Trunk の `public_url` を `/` に変更し、`trunk serve` のローカル配信パスを `http://127.0.0.1:8080/` に統一。
- `web/index.html` に `vendor` の copy-dir を追加し、`web/vendor` を用意して editor sample の静的配布を Trunk 経由で行えるようにした。
- README と doc/web_playground.md に editor sample の取得手順とローカル起動 URL を追記。
- `web/index.html` の CSS/JS を Trunk 管理のアセットとして宣言し、`styles.css` と `main.js` が dist に出力されるように調整。
- `web/main.js` は Trunk の `TrunkApplicationStarted` イベントと `window.wasmBindings` を利用して wasm-bindgen 生成物にアクセスする方式に変更。
- 埋め込み editor は `web/vendor/editorsample` が存在する場合のみ iframe に読み込み、存在しない場合はフォールバック textarea を使用するように変更。
- doc/web_playground.md に `public_url` と `serve-base` の関係を追記し、`trunk serve` のアクセスパスに関する注意点を明記。

## plan.md との乖離・注意点 (追加)
- plan.md に web playground の配信手順は未記載のため、必要なら仕様欄に追記が必要。

# 2026-02-03 作業メモ (kpread UTF-8 BOM 対応)
- PowerShell のパイプ入力が UTF-8 BOM (EF BB BF) を付与する場合、kpread の `scanner_read_i32` が先頭の BOM を数値として扱い、0 を返し続ける問題を確認。
- `scanner_skip_ws` に UTF-8 BOM のスキップを追加し、既存の UTF-16 BOM/NULL スキップと同じ位置で処理。
- 回帰テストとして `nepl-core/tests/fixtures/stdin_kpread_i32.nepl` を追加し、`stdin_kpread_utf8_bom` で BOM 付き入力を検証。
- 動作確認: `printf '\xEF\xBB\xBF1 3\n' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 作業メモ (日本語文字列の stdout)
- 文字列リテラルの lexer が UTF-8 を 1 バイトずつ `char` に変換していたため、日本語が mojibake になる問題を確認。
- 文字列リテラルの通常文字の読み取りを UTF-8 `char` 単位に変更し、`i` を `len_utf8` 分進めるよう修正。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_japanese.nepl` と `stdout_japanese_utf8` を追加。
- 動作確認: `cargo run -p nepl-cli -- -i examples/helloworld.nepl --run -o a`

# 2026-02-03 作業メモ (CLI --run の stdio プロンプト)
- `nepl-cli --run` の WASI `fd_write` が `print!` のみで flush しておらず、プロンプト `"> "` が入力後に表示される問題を確認。
- `fd_write` を raw bytes で `stdout.write_all` し、最後に `flush` するよう修正。
- 動作確認: `printf "3 5 3\n" | cargo run -p nepl-cli -- -i examples/stdio.nepl --run -o a`

# 2026-02-03 作業メモ (ANSI エスケープ出力)
- 文字列リテラルのエスケープに `\xNN` (hex) を追加し、`"\x1b[31m"` など ANSI エスケープを直接書けるようにした。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_ansi.nepl` と `stdout_ansi_escape` を追加。

# 2026-02-03 作業メモ (std/stdio の ANSI 色ヘルパー)
- `std/stdio` に `ansi_red` などの色コード関数と `print_color` / `println_color` を追加。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_color.nepl` と `stdout_ansi_helpers` を追加。

# 2026-02-03 作業メモ (Web playground terminal)
- `nepl-core` に `load_inline_with_provider` を追加し、仮想 stdlib ソースからのロードを可能にした。
- `nepl-web` (wasm-bindgen) を新設し、ブラウザ内でのコンパイルと stdlib テスト実行を提供。
- `web/` にターミナル UI を追加し、`run`/`test`/`clear` コマンドと stdin 入力を実装。
- `doc/web_playground.md` を追加し、Web playground の実行仕様を整理。
- Trunk 0.20 互換のため、`web/index.html` の `<link data-trunk>` から `data-type="wasm-bindgen"` を削除。
- `nepl-web` の `include_str!` パスを修正し、`nepl-core` ローダーに wasm 向けのファイルアクセス抑制を追加。
- Web UI を mlang playground の構成に合わせて整理し、WAT 出力パネルと操作ボタンを追加。
- 後方互換性のため、i32 のみの alias 関数（add/sub/mul/div_s/lt/eq など）を提供。

# 2026-01-31 作業メモ (stdlib テストの充実化)
- stdlib/tests に新規テストファイルを追加：option.nepl/cast.nepl/vec.nepl/stack.nepl/error.nepl/diag.nepl
- 既存テストを拡張：math/string/result/list の各テストカバレッジを大幅増加。
- テスト対象：
  - **option**: is_some/is_none/unwrap/unwrap_or
  - **cast**: bool↔i32 変換
  - **vec**: vec_new/push/get/capacity/is_empty
  - **stack**: stack_new/push/pop/peek/len
  - **error**: error_new/各種 ErrorKind
  - **diag**: kind_str（ErrorKind → 文字列）
  - **math**: i32/i64 の全演算+ビット演算、浮動小数点操作
  - **string**: len/concat/str_eq/from_i32 の拡張テスト
  - **result**: ok/err/is_ok/is_err/unwrap_or
  - **list**: cons/nil/get/head/tail/reverse/len

# 2026-02-01 作業メモ (if式の無限メモリ割り当てバグ修正)
## 問題分析
- if テストで 15 個中 8 個が成功だが、残り 7 個でメモリ割り当てエラー（5.5GB）発生
- **失敗パターン**: `#import "std/math"` + `#use std::math::*` を含むすべてのテストケース
  - `if_a_returns_expected` (キーワード形式: `if true 0 1`)
  - `if_b_returns_expected` (キーワード形式: `if true then 0 else 1`)
  - `if_c_returns_expected` (レイアウト形式、マーカーなし)
  - その他 `if_d/e/f` とバリアント

- **成功パターン**: 同じく `#import "std/math"` を含むが、if: レイアウト形式で role マーカー(`cond`/`then`/`else`)を使用
  - `if_c_variant_cond_keyword` (cond マーカーあり)
  - `if_mixed_cond_then_block_else_block` (cond/then/else ブロック形式)
  - その他レイアウト形式マーカーあり

## 原因特定
- **根本原因は typecheck の apply_function における if / while ハンドラ内で result 型変数を unify する際に生じた型の循環参照**
- parser の修正により以下の 2 つのバグを fix 済み:
  1. マーカーに inline 式がある場合、ブランチが即座に finalize されず、後続の positional 行と grouping される
  2. 複数ステートメント positional ブランチが個別ブランチに split されない

- 新たに typecheck 内の if/while ケースで result 型との unify により**無限型構造**が生成されていた

## 修正内容
1. `typecheck.rs` 行 2369-2397 (if ケース):
   - 元: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);`
   - 修: `let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);` のみで result 型変数は使用しない
   - 理由: result は fresh 型変数で、これと unify すると型の循環参照が発生し、monomorphize 段階での型 substitution で exponential explosion

2. `typecheck.rs` 行 2400-2427 (while ケース):
   - 同様に `self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit())` を削除
   - 修: `self.ctx.unit()` を直接返す

3. parser.rs debug 診断の削除:
   - 行 859-890: if 形式のアイテムシェイプをダンプする diagnostic を削除
   - 行 1536-1550: if-layout ブランチ役割情報ダンプ diagnostic を削除
   - 行 1515-1530: marker 未検出の warning を削除

## 状態
- 全 if テスト 15 個が成功し、合計実行時間 5.12 秒でコンプリート（以前は一部でメモリ割り当てエラー）
- debug ファイル削除済み: `parse_if_debug.rs`、`compile_if_a.rs`

# 2026-02-03 作業メモ (if テスト停止/lexer)
## 問題発見
- if テストの一部でコンパイラが停止し、巨大メモリ割り当てエラーが発生。
- テスト内の `#import`/`#use` 行がトップレベルでインデントされていた。

## 原因特定と修正
- lexer がトップレベルのディレクティブ行でもインデント増加を `Indent` として出力してしまい、想定外のブロック構造になって typecheck が停止していた。
- `expect_indent` を追加し、直前の行末 `:` か `#wasm` ブロックの時のみインデント増加を許可するように修正。
- ディレクティブ行で不正なインデント増加がある場合はインデントを据え置き、トップレベル扱いに固定。

## テスト実行結果
- `cargo test -p nepl-core --test if` が通過。

# 2026-02-03 作業メモ (整数リテラル/move_check)
## 修正内容
- 整数リテラルの `i32` 変換が overflow で 0 になっていたため、`i128` でパースして `i32` にラップする実装に修正。`0x` 16進にも対応し、無効値は診断を出す。
- `Intrinsic::load`/`store` の move_check を特殊扱いし、アドレス側は borrow として扱うように修正。`load` はロード対象型が Copy のとき borrow 扱い、`store` は常にアドレスを borrow として処理。
- `visit_borrow` で `Intrinsic` の引数を再帰的に borrow として扱い、誤った move 判定を抑制。
- Struct/Enum/Apply は Copy ではない前提を維持。
- `std/vec` で len/cap/data をローカルに保持し、同一値への複数アクセスによる move_check 失敗を回避。

## テスト実行結果
- `cargo run -p nepl-cli -- test` が通過。
- `cargo test` が通過。

## plan.md との差分メモ (追加)
- トップレベルのディレクティブ行のインデント扱い（`#wasm` ブロック以外は増加を無視する仕様）が plan.md に未記載。
- 整数リテラルの overflow ルール（`i32` へのラップ）と 16 進表記の仕様が plan.md に未記載。
- move_check における `load`/`store` の borrow 扱いが plan.md に未記載。

# 2026-02-03 作業メモ (CLI 出力/emit 拡張)
## 修正内容
- `--emit` を複数指定可能にし、`wasm`/`wat`/`wat-min`/`all` を選択できるように拡張。
- `--output` をベースパスとして扱い、`.wasm`/`.wat`/`.min.wat` を派生生成するよう変更。
- pretty WAT は `wasmprinter::print_bytes` の出力を使用し、minified WAT はその出力を空白圧縮して生成。
- CLI 出力のユニットテストを追加（emit 解析、出力ベース判定、minify、出力ファイル生成）。
- `doc/cli.md` と README の CLI 例を更新。
- GitHub Actions の `nepl-test.yml` に multi-emit の出力確認ステップを追加。

## テスト実行結果
- `cargo test -p nepl-cli`

## plan.md との差分メモ (追加)
- `--emit` の複数指定と `wat-min` 出力、`--output` のベースパス運用が plan.md に未記載。

# 2026-02-03 作業メモ (kpread/abc086_a)
## 修正内容
- `kp/kpread` の Scanner を i32 ポインタベースに変更し、buf/len/pos を固定オフセットで `load_i32`/`store_i32` する実装に変更。
- `scanner_*` の引数型を `(i32)` に統一し、`scanner_new` は 12 バイトのヘッダ領域に buf/len/pos を格納する形式に変更。
- `examples/abc086_a.nepl` の Scanner 型注釈を i32 に更新。

## テスト実行結果
- `printf "1 3" | cargo run -p nepl-cli -- -i examples/abc086_a.nepl --run`

# 2026-02-03 作業メモ (if[profile])
## 修正内容
- `#if[profile=debug|release]` を lexer/parser/AST/typecheck に追加し、コンパイル時プロファイルに応じてゲートするようにした。
- `nepl-core/tests/neplg2.rs` に profile ゲートのテストを追加。

# 2026-02-03 作業メモ (profile オプション/デバッグ出力)
## 修正内容
- コンパイラの `CompileOptions` に `profile` を追加し、`#if[profile=debug|release]` を CLI から制御できるように拡張。
- CLI に `--profile debug|release` を追加し、未指定時はビルド時のプロファイルを使用。
- `std/stdio` に `debug`/`debugln` を追加（debug では出力、release では no-op）。
- `std/diag` に `diag_debug_print`/`diag_debug_println` を追加。
- `README.md` と `doc/cli.md`/`doc/debug.md` を更新。

## テスト実行結果
- `cargo test -p nepl-core --test neplg2`

# 2026-02-03 設計メモ (リライト方針まとめ)
- `doc/rewrite_plan.md` を追加。現行実装のスナップショットと課題、後方互換なしでの再設計アーキテクチャ/実装ロードマップを記載。
- モジュールはファイルスプライス前提をやめ、`nepl.toml` によるパッケージ/依存管理と `#import ... as {alias|*|{...}|@merge}`、`pub #import` による再エクスポートを採用する方針。
- 名前解決は DefId ベースの二段階（定義収集→解決）、Prelude 明示化、選択/オープン/エイリアス優先順位を整理。
- 型システムは DefId 付き HIR と単相化 (monomorphize) を再構築し、MIR を経て WASM に落とす計画。CLI の target 自動推測は廃止し、manifest 駆動にする。
- 今回はドキュメントのみ追加。テストは未実行。

# 2026-02-03 モジュールグラフ(Phase2) 着手
- `nepl-core/src/module_graph.rs` を追加。依存グラフと循環検出のみを実装し、ファイルスプライスせずに AST を保持するノードを構築する段階。
- `ModuleGraphBuilder` は stdlib を既定依存として登録し、`#import` パス（相対/パッケージ）からファイルを解決。DFS で cycle を検出し、topo 順を保持。
- `lib.rs` に module_graph を公開。
- まだ名前解決/可視性/Prelude 反映は未実装（Phase3 以降で対応予定）。

# 2026-02-03 Export表(Phase3) 基礎実装
- AST/lexer/parser に `pub` 可視性を導入し、`fn/struct/enum/trait` で公開指定をパース可能に。
- ModuleGraph に pub 定義と pub import の再エクスポートを集計する ExportTable を追加。重複は DuplicateExport として検出。
- ModuleNode に import の可視性と依存先 ModuleId を保持し、topo 順に基づき export を固定点なしで構築。
- テスト: ネットワークなし環境のため cargo test 実行不可（wasmparser ダウンロードで失敗）だが、ローカル追加テストを用意。

# 2026-02-03 名前解決準備(Phase4) 着手
- `nepl-core/src/resolve.rs` を追加し、DefId/DefKind とモジュールごとの公開定義テーブルを収集する `collect_defs`、ExportTable と合成する `compose_exports` を実装（式中識別子の解決までは未接続）。
- Phase4 の本体（スコープ優先順位、Prelude、@merge を含む解決）は未着手。次ステップで Resolver を HIR 生成に組み込む必要あり。

# 2026-02-03 ビルド調整
- `lib.rs` で `extern crate std` を条件付きでリンクし、module_graph などの std 依存を解決（wasm32 以外）。

# 2026-02-03 作業メモ (kpread UTF-16LE 入力)
## 修正内容
- `kp/kpread` の `scanner_skip_ws`/`scanner_read_i32` が UTF-16LE の NUL バイトを文字として扱っていたため、NUL をスキップする処理を追加。
- PowerShell パイプでの `\"1 3\"` 入力でも `abc086_a.tmp.nepl` が正しく Odd を出すように修正。

## テスト実行結果
- `printf '1\0 3\0' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`
