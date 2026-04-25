# Core レビュー

作成日: 2026-04-25

対象: `nepl-core/src/**`

## レビュー範囲

確認した主要ファイルは次の通りです。

| 区分 | ファイル |
|---|---|
| パイプライン | `lib.rs`, `compiler.rs`, `loader.rs`, `module_graph.rs`, `target_precheck.rs` |
| frontend | `lexer.rs`, `parser.rs`, `ast.rs`, `nm.rs` |
| 型・名前解決 | `types.rs`, `typecheck.rs`, `resolve.rs`, `name_resolve.rs`, `effects.rs` |
| HIR / pass | `hir.rs`, `passes/move_check.rs`, `passes/drop_insertion.rs`, `passes/codegen_precheck.rs` |
| codegen | `codegen_wasm.rs`, `codegen_llvm.rs`, `wasm_shared.rs`, `runtime_helpers.rs`, `monomorphize.rs` |
| 診断 | `diagnostic.rs`, `diagnostic_ids.rs`, `error.rs`, `span.rs`, `log.rs` |

行数上の最大リスクは `typecheck.rs` 8759 行、`parser.rs` 3891 行、`codegen_llvm.rs` 3528 行、`codegen_wasm.rs` 2339 行です。設計書 `doc/2.1impl/compiler_structure.md` が指摘している巨大ファイル問題は、現行コードにもそのまま残っています。

## 総評

NEPLg2.0 の core は、動く経路を増やすために型検査・名前解決・HIR 生成・trait 解決・effect 判定・一部の codegen 前処理が密結合しています。特に `typecheck.rs` の hot path は、全走査、`Vec::remove`、`TypeCtx` clone、`BTreeMap` 線形探索を多用しており、「コンパイラが異常に遅い」原因候補として優先的に分解・測定すべきです。

また、`core` は no_std 境界、診断、import semantics、Resource IR のいずれも計画とずれており、局所パッチではなく pipeline stage ごとの分割が必要です。

## RV-CORE-001: core の no_std 境界が崩れている

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: architecture
- 対象: `nepl-core/src/lib.rs`, `nepl-core/src/compiler.rs`, `nepl-core/src/typecheck.rs`, `nepl-core/src/codegen_wasm.rs`

### 根拠

- `nepl-core/src/lib.rs:1`: crate root は `#![no_std]`。
- `nepl-core/src/compiler.rs:2`: `extern crate std;` が unconditional。
- `nepl-core/src/typecheck.rs:3`: `extern crate std;` が unconditional。
- `nepl-core/src/typecheck.rs:9`: `std::path::Path` を core 側で直接使用。
- `nepl-core/src/codegen_wasm.rs:4`: wasm backend も `extern crate std;` を持つ。

### 問題

`plan.md` と `doc/self_host.md` は compiler core を WASI なしの純粋 core として扱う方針ですが、現行 core は `std` に直接依存しています。`nepl-core` を wasm32/no_std 対象へ切り出す前提が崩れ、CLI と core の責務分離も不明確です。

### 影響

core をブラウザ / WASM / self-host bootstrap で再利用する際に、host I/O や path 処理が混入します。将来の `stdlib/neplg2/core` へ移植するときも、どこまでが pure compiler core なのか判別しにくくなります。

### 修正方針

`SourceMap` の path 表示、debug output、host filesystem access を core API の外に出します。core は `alloc` までに限定し、CLI / web / test harness が path と I/O adapter を渡す構成に分けます。

### 検証

`nepl-core` を `wasm32-unknown-unknown` または同等の no_std 条件でビルドする CI job を追加します。

## RV-CORE-002: typecheck.rs が巨大化しすぎて責務が分離できていない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: architecture
- 対象: `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/typecheck.rs`: 8759 行。
- `nepl-core/src/typecheck.rs:566`: instantiation cache を型検査本体が直接持つ。
- `nepl-core/src/typecheck.rs:7777`: `Env` / `Scope` 実装も同じファイル内。
- `nepl-core/src/typecheck.rs:8354`: `LabelEnv` / `StringTable` など補助構造も同居。

### 問題

型推論、trait capability、overload、名前探索、HIR lowering、effect 判定、import alias 補助、diagnostic recovery が 1 ファイルに集中しています。局所修正が別機能へ波及しやすく、性能改善も profiling point を切りにくい状態です。

### 影響

既知バグの根本原因を追いにくく、修正のたびに回帰範囲が広がります。`doc/2.1impl/compiler_structure.md` で示されている `check/`, `ty/`, `resolve/`, `hir/` 分割方針との乖離も大きいです。

### 修正方針

短期的には `Env`、overload 解決、prefix reduction、trait impl 検査を別 module へ切り出します。中期的には AST flat list から HIR への lowering を型検査結果と分離し、`resolve` / `ty` / `check` の依存方向を固定します。

### 検証

分割ごとに既存 `tests/compiler/**` と stdlib doctest を走らせ、ファイル移動だけの段階では出力 JSON の差分がないことを確認します。

## RV-CORE-003: reduce_calls が O(n^2) 化しやすく固定上限で正当な入力を落とす

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: performance
- 対象: `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/typecheck.rs:5540`: `reduce_calls` に `max_iterations = 1000`。
- `nepl-core/src/typecheck.rs:5553`: 各 iteration で stack を後ろから全走査。
- `nepl-core/src/typecheck.rs:5629`: 引数取り出しに `stack.remove(func_pos + 1)` を使用。
- `nepl-core/src/typecheck.rs:5632`: callee 取り出しも `stack.remove(func_pos)`。
- `nepl-core/src/typecheck.rs:5730`: guarded reduction 側にも同じ 1000 上限。

### 問題

prefix expression の縮約が「全走査して middle remove」を繰り返すため、長い式や overload が多い式で O(n^2) 以上になりやすいです。さらに iteration 上限が 1000 固定なので、入力が正しくても `TypeCallReductionLimitExceeded` になる可能性があります。

### 影響

コンパイル時間が式長と候補数に対して急激に増えます。stdlib の巨大な関数や tutorial の複雑な式で体感速度が悪化し、CI timeout の原因にもなります。

### 修正方針

`Vec` の middle remove を避け、index span を持つ reduction queue または小さな call frame stack に置き換えます。上限で止めるのではなく、進捗がない状態を検出して診断する方式にします。

### 検証

長い prefix chain、深い pipe、overload 候補が多い call を含む performance fixture を追加し、縮約回数と処理時間を JSON に出します。

## RV-CORE-004: overload 解決が候補ごとに TypeCtx 全体を clone している

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: performance
- 対象: `nepl-core/src/typecheck.rs`, `nepl-core/src/codegen_llvm.rs`, `nepl-core/src/codegen_wasm.rs`

### 根拠

- `nepl-core/src/typecheck.rs:6497`: overload candidate ごとに `let mut tmp_ctx = self.ctx.clone();`。
- `nepl-core/src/typecheck.rs:7234`: 関数値解決でも `TypeCtx` clone。
- `nepl-core/src/codegen_llvm.rs:2976` 付近: layout 計算で `types.clone()` を使う。
- `nepl-core/src/codegen_wasm.rs:75` 以降: generic Apply の storage 計算で `ctx.clone()` を使う。

### 問題

`TypeCtx` は arena と型変数状態を持つため、clone は候補数と型数に比例して重くなります。overload が stdlib 全体に広がるほど、1 call の型解決で大量の一時 arena clone が発生します。

### 影響

「コンパイラが異常に遅い」問題の主要候補です。特に `add`, `eq`, `len`, `get`, `push` のような多重定義名が頻出する stdlib / tutorial で悪化します。

### 修正方針

候補検査用の snapshot/rollback を `TypeCtx` に実装し、arena 全体 clone を禁止します。型代入は trail に記録し、候補検査後に rollback します。layout 計算は substitution cache を導入します。

### 検証

overload 解決ごとの candidate count、clone count、unify count を profiling counter として取得し、修正前後で比較します。

## RV-CORE-005: loader が import clause を無視して全 import をフラット結合している

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/loader.rs`, `nepl-core/src/parser.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/parser.rs:458`: `ImportClause` は `DefaultAlias`, `Alias`, `Open`, `Selective`, `Merge` まで parse している。
- `nepl-core/src/loader.rs:436`: `Directive::Import { path, .. }` として clause を捨てている。
- `nepl-core/src/loader.rs:457`: imported module の root items を clause に関係なく親へ push。
- `nepl-core/src/typecheck.rs:8310`: qualified alias map は後段で別途作っているが、loader の item 可視性は制御していない。

### 問題

`#import "x" as name` や `as { a }` の構文があっても、loader は対象 module の item をすべて親 module に混ぜます。alias / selective import は型検査の一部補助にしか効かず、未選択 symbol が見えてしまう可能性があります。

### 影響

名前衝突、意図しない overload 候補増加、import 順依存の挙動が起きます。性能面でも、使っていない module の item まで常に型検査候補になります。

### 修正方針

loader は AST を物理的に結合せず、module graph と export/import table を構築するだけにします。可視性は `resolve` stage で clause に従って解決し、HIR には canonical symbol を渡します。

### 検証

`as name` で未修飾参照が失敗するテスト、selective import で未選択 symbol が失敗する compile_fail テスト、open import の曖昧性テストを追加します。

## RV-CORE-006: 通常実行でデバッグ出力が stderr へ漏れる

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/loader.rs`, `nepl-core/src/typecheck.rs`, `nepl-core/src/types.rs`, `nepl-core/src/monomorphize.rs`

### 根拠

- `nepl-core/src/loader.rs:166`: `load_inline_with_provider` が常時 `eprintln!`。
- `nepl-core/src/loader.rs:271`: `load_from_contents_with` が path/canon を出す。
- `nepl-core/src/loader.rs:320`: `load_file` が常時 loading log を出す。
- `nepl-core/src/typecheck.rs:1965`: diagnostics summary を stderr 出力する経路がある。
- `nepl-core/src/types.rs:1527`: `type_to_string` cycle で常時 `eprintln!`。

### 問題

verbose option と関係なく stderr が汚染されます。compiler を library として使う web / test runner / CLI で、stderr を診断やプログラム出力として扱う経路と衝突します。

### 影響

doctest の `stderr` 比較、JSON 出力、CI log が不安定になります。大量 import ではログ出力そのものも性能劣化要因になります。

### 修正方針

core から直接 `eprintln!` を排除し、`Diagnostic` または injectable logger に統一します。debug-only log は `crate::log::is_verbose()` を通し、default は完全に無出力にします。

### 検証

正常 compile の stderr が空であることを CLI / Node runner の回帰テストに追加します。

## RV-CORE-007: codegen が診断ではなく panic で落ちる経路を多数持つ

- 解決済: false
- 状態: open
- 優先度: P0
- 種別: bug
- 対象: `nepl-core/src/codegen_wasm.rs`, `nepl-core/src/codegen_llvm.rs`

### 根拠

- `nepl-core/src/codegen_wasm.rs:476`: unsupported extern signature で `panic!`。
- `nepl-core/src/codegen_wasm.rs:504`: unsupported function signature で `panic!`。
- `nepl-core/src/codegen_wasm.rs:1123`: string literal lookup failure で `panic!`。
- `nepl-core/src/codegen_wasm.rs:1140`: unknown variable で `panic!`。
- `nepl-core/src/codegen_llvm.rs:1391`: unknown variable で `panic!`。
- `nepl-core/src/codegen_llvm.rs:2863`: unsupported intrinsic で `panic!`。

### 問題

compiler はユーザー入力に対して診断を返すべきですが、backend は内部不整合を `panic!` で処理しています。precheck が完全でない場合、通常のコンパイルエラーではなくプロセス異常終了になります。

### 影響

CLI / web / doctest runner が compiler crash と compile error を区別できません。クラッシュ時には span も失われ、根本原因の追跡が難しくなります。

### 修正方針

codegen API を `Result<CodegenResult, Vec<Diagnostic>>` に寄せ、panic 経路をすべて `DiagnosticId::Codegen...` 付きエラーに置き換えます。precheck は「panic を防ぐ安全網」ではなく「早い診断」の位置づけにします。

### 検証

unsupported intrinsic、unknown field selector、invalid raw wasm を compile_fail として固定し、プロセスが panic しないことを確認します。

## RV-CORE-008: effect 判定が文字列包含に依存していて不健全

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/effects.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/effects.rs:57`: `marker_is_impure_io` が `text.contains(m)`。
- `nepl-core/src/effects.rs:69`: raw body effect は行文字列の contains で判定。
- `nepl-core/src/effects.rs:77`: `HirBody::Block(_)` は raw body 判定では Pure。

### 問題

raw wasm / LLVM IR の effect を文字列検索で推定しています。コメントや別名に `fd_write` が含まれれば false positive になり、逆に wrapper 名や syscall 経由は missed detection になります。

### 影響

pure 関数から外部 I/O が呼べる、または pure な raw body が impure と誤判定されるなど、effect system の信頼性が落ちます。

### 修正方針

raw body は明示 effect annotation を必須にするか、extern / intrinsic 宣言に effect を持たせて call graph で伝播します。文字列検索は診断補助に限定します。

### 検証

pure 関数内の `fd_write` wrapper、コメントに `fd_write` を含む pure raw body、syscall 経由 I/O のテストを追加します。

## RV-CORE-009: move/borrow/drop が Resource IR なしで後付け実装されている

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: architecture
- 対象: `nepl-core/src/passes/move_check.rs`, `nepl-core/src/passes/drop_insertion.rs`, `nepl-core/src/compiler.rs`

### 根拠

- `nepl-core/src/compiler.rs:374`: drop 挿入後に monomorphize し、その後 move check。
- `nepl-core/src/passes/move_check.rs:35`: 変数状態は `BTreeMap<String, Vec<VarState>>`。
- `nepl-core/src/passes/move_check.rs:616`: `AddrOf` は borrow として状態を変える。
- `nepl-core/src/passes/drop_insertion.rs:91`: scope exit で trait drop call を後付け生成。

### 問題

Resource IR がなく、HIR 木を直接走査して所有権状態を推測しています。borrow の lifetime release、field move、auto drop と user drop の整合、branch merge が局所状態で処理されており、仕様化しづらいです。

### 影響

false positive / false negative の両方が起きやすく、stdlib の owning collection と組み合わせたときに double free や use-after-move を防ぎきれません。

### 修正方針

`doc/2.1impl/compiler_structure.md` の方針どおり HIR 後に Resource IR を導入し、move、borrow、region、drop elaboration を別 pass に分けます。drop 挿入前に ownership を確定し、auto drop は Resource IR 上で挿入します。

### 検証

branch/loop/borrow/field move/drop trait の compile_fail と should_panic を整理し、Resource IR dump の snapshot test を追加します。

## RV-CORE-010: name resolution が二重化し本パイプラインに統合されていない

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `nepl-core/src/resolve.rs`, `nepl-core/src/name_resolve.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/resolve.rs:1`: DefId / export table の scaffolding。
- `nepl-core/src/name_resolve.rs:1`: 別の name resolution skeleton。
- `nepl-core/src/name_resolve.rs:48`: `resolve_names` は diagnostics 空で返すだけ。
- `nepl-core/src/typecheck.rs:7777`: 実際の値・関数探索は `typecheck.rs` 内の `Env` が担う。

### 問題

名前解決の意図は `resolve.rs` と `name_resolve.rs` にありますが、本 pipeline は `typecheck.rs` 内 Env に依存しています。DefId を持つ canonical resolution になっていません。

### 影響

import alias、overload、shadowing、qualified path の責務が分散し、同名関数や module boundary のバグが出やすくなります。

### 修正方針

`name_resolve.rs` は削除または `resolve/` へ統合し、DefId を AST/HIR lowering の入力にします。`typecheck.rs` の Env は型検査用 local binding に限定します。

### 検証

cross-file import、qualified alias、shadowing、ambiguous open import の tree test を DefId snapshot として追加します。

## RV-CORE-011: TypeExpr が span を保持せず診断位置が失われる

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: bug
- 対象: `nepl-core/src/ast.rs`, `nepl-core/src/parser.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/ast.rs:41`: `TypeExpr::span()` が常に `Span::dummy()`。
- `nepl-core/src/parser.rs`: 型式 parsing の各所で `Span::dummy()` fallback が多い。
- `nepl-core/src/typecheck.rs:5547`: call reduction limit diagnostic も `Span::dummy()`。

### 問題

型注釈や型式のエラー位置を正確に出せません。dummy span は `<unknown>:1:1` のような診断に化け、修正箇所が分からなくなります。

### 影響

ユーザー体験が悪く、compile_fail テストで span を固定しにくくなります。内部バグの triage でも、問題の型式を特定しづらいです。

### 修正方針

`TypeExpr` を enum + span wrapper にするか、各 variant に span を持たせます。parser は型式の開始・終了 span を保存し、typecheck diagnostic では必ず元の型式 span を使います。

### 検証

型注釈 mismatch、trait bound mismatch、generic arity mismatch の `diag_span` テストを追加します。

## RV-CORE-012: target/profile gate の評価が複数箇所に散っている

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `nepl-core/src/compiler.rs`, `nepl-core/src/target_precheck.rs`, `nepl-core/src/typecheck.rs`

### 根拠

- `nepl-core/src/compiler.rs:44`: target gate parser は compiler に定義。
- `nepl-core/src/target_precheck.rs:54`: `gate_allows` は compiler の parser を呼ぶ。
- `nepl-core/src/typecheck.rs:8976`: typecheck 側にも target gate helper がある。
- `nepl-core/src/target_precheck.rs:285`: precheck は codegen 前に別途実行される。

### 問題

target/profile gate の有効 statement 判定が複数箇所に散っています。未知 gate は `false` に丸められやすく、診断になる箇所と静かに無効化される箇所が分かれます。

### 影響

`#if[target=...]` を含む stdlib で、target ごとに実行される関数 body がずれる可能性があります。raw body precheck と typecheck の見ている active statements が一致しないと、backend panic に繋がります。

### 修正方針

gate evaluation を `passes/target_gate` 相当へ一本化し、AST から inactive item を明示的に除外した lowered module を作ります。未知 gate は warning ではなく diagnostic error にします。

### 検証

target gate の boolean expression、unknown gate、profile gate、raw body selection の matrix test を追加します。

## RV-CORE-013: 参照引数の関数呼び出しが一時 borrow にならず所有値を固定する

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: bug
- 対象: `nepl-core/src/passes/move_check.rs`, `tests/compiler/move_check.n.md`, `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`

### 根拠

- `nepl-core/src/passes/move_check.rs`: 通常の関数呼び出し引数は `visit_expr` で処理され、`AddrOf` が `visit_borrow` によりスコープ終端までの borrow として記録される。
- `stdlib/alloc/collections/vec.nepl`: `len_ref &v` の後に `push v ...` する doctest が、`Vec` から `Copy` を外した時点で `cannot move out of shared borrowed value` になる。
- `stdlib/alloc/collections/stack.nepl`: `peek_ref &stk` / `len_ref &stk` も同じ構造です。

### 問題

`len_ref &v` のように参照を関数呼び出しへ渡すだけの式でも、move checker は `v` をスコープ終端まで shared borrow として扱います。参照引数の呼び出しは式の評価中だけの一時 borrow であるべきですが、現状では borrow が解放されないため、非 Copy 所有値を読み取った後に移動・更新できません。

### 影響

`Vec` / `Stack` を正しく非 Copy にすると、borrow-based read API を追加しても、その後の `push` / `free` / move が拒否されます。所有権を安全にした stdlib API と現在の borrow checker が噛み合わず、`RV-STDLIB-003` の根本修正を妨げます。

### 修正方針

move checker で call target の parameter type を参照し、parameter が `&T` / `&mut T` の場合は対応する引数を一時 borrow として評価するよう修正しました。`&x` は呼び出し式の評価中だけ borrow し、呼び出し後の `x` の所有権状態を `Valid` のまま保ちます。非参照引数の by-value move と、永続的な local borrow (`let r &x`) は従来どおり区別します。

### 検証

`tests/compiler/move_check.n.md` に「非 Copy 値を参照引数へ渡した後に move できる」回帰テストを追加しました。既存の「local に保持した shared borrow 中の move は引き続き拒否される」compile_fail と合わせて確認しています。

確認済み:

- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/move-check-rv-core-013.json -j 1` (`total=14`, `passed=14`, `failed=0`)
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-CORE-014: Pair から取り出した generic collection の型が overload 解決へ伝播しない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-core/src/typecheck.rs`, `nepl-core/src/codegen_wasm.rs`, `stdlib/alloc/collections/vec.nepl`

### 根拠

- `stdlib/alloc/collections/vec.nepl::doctest#28`: `partition` の戻り値 `.Pair` から `let evens get parts 0` で取り出した後、`len<i32> evens` が `error[D3006]: no matching overload found` になる。
- 同じ箇所へ明示型注釈を付けると、過去の調査では overload error ではなく codegen の internal panic 経路へ進んだ。
- `RV-CORE-004` は overload 解決の clone 過多を扱うが、この件は `.Pair` field access 結果の型伝播と overload 候補選択の正当性問題です。

### 問題

`.Pair` のような generic tuple 相当の値から `get` で取り出した collection の型情報が、その後の overloaded function call に十分伝播していません。結果として `Vec<i32>` であるべき値に対して `len<i32>` の候補を選べず、正常な stdlib doctest が compile error になります。

### 影響

`partition` のように複数の collection を返す API を利用したコードで、明示型注釈なしに後続 API を呼べません。型注釈で回避しようとしても backend panic 経路へ進む可能性があり、診断品質と codegen 安全性の両方に影響します。

### 修正方針

`get` intrinsic / field accessor の結果型を expected type と overload argument type へ確実に反映します。候補選択時に generic tuple / `.Pair` の field 型を解決済み `TypeId` として保持し、後続の `len<i32>` へ渡る引数型が `Vec<i32>` として見えるようにします。backend panic に進む経路は `RV-CORE-007` と合わせて診断へ落とします。

### 検証

`tests/compiler` に `.Pair` から取り出した `Vec<i32>` に `len<i32>` / `get<i32>` を呼ぶ最小再現を追加します。`stdlib/alloc/collections/vec.nepl` の `partition` doctest が通ることも確認します。
