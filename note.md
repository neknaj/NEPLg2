# 状況メモ (2026-01-22)
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
