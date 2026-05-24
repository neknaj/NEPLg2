---
id: ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754
title: "NEPLg2.1 corpus migration needs semantic generic rewrite"
area: stdlib
status: open
resolved: false
priority: P0
type: maintenance
created: 2026-05-24
updated: 2026-05-24
target: "stdlib/**, tests/**, tutorials/**, doc/examples/**"
---

# ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754: NEPLg2.1 corpus migration needs semantic generic rewrite

## 概要

Existing NEPLg2 source uses angle-bracket annotations, type applications, explicit generic postfix calls, and parenthesized lambda arguments throughout the stdlib and test corpus.

## 対象

- `stdlib/**, tests/**, tutorials/**, doc/examples/**`

## 根拠

- 現在の実行対象 corpus は `stdlib/`、`tests/`、`tutorials/`、`doc/examples/` に分散しており、角括弧型注釈、型適用、generic postfix call、parenthesized lambda が広く使われている。
- subagent 調査では、`tests/compiler/typeannot.n.md`、`tests/compiler/functions.n.md`、`tests/compiler/generics.n.md`、`stdlib/core/result.nepl`、`stdlib/alloc/collections/vec/**` が代表的な高密度領域として確認された。
- `tuple_old_syntax.n.md` や compile_fail fixture には、旧構文を失敗例として残すべき箇所がある。
- `stdlib/neplg2/` は selfhost compiler 実装側の構文処理を含むため、利用コードと同じ一括置換対象にはできない。
- 設計計画: [NEPLg2.1 surface syntax migration plan](../../doc/neplg2/neplg21_syntax_migration_plan.md)

## 問題

Existing NEPLg2 source uses angle-bracket annotations, type applications, explicit generic postfix calls, and parenthesized lambda arguments throughout the stdlib and test corpus.

## 影響

A textual rewrite can migrate simple annotations, but explicit generic postfix removal requires expected-type and signature-aware decisions, especially in stdlib callbacks and compile-fail fixtures.

## 修正方針

Build an inventory and migrate executable source to NEPLg2.1 syntax using AST/token-balanced tooling plus LLM review for generic call sites and lambda/function literal boundaries.

### 分類

自動変換しやすいもの:

- balanced token で範囲を取れる `<TypeExpr>` 型注釈。
- `Vec<i32>` / `Result<i32,str>` のような型式内 generic application。
- `fn name <signature> (args):` の外形変換。
- struct field / enum payload の型注釈。

LLM/手動判断が必要なもの:

- `some<i32>` / `unwrap_ok<T,E>` / `Result::Ok<T,E>` などの明示 generic call。
- 期待型が不足しているため `%T` 注釈の追加が必要な call。
- `let f (x):` や `apply 10 (x):` の function literal と旧 tuple fixture の区別。
- 関数を返す関数型を `%fn A (fn B C)` として grouping する必要がある箇所。
- owner-preserving callback signature、borrowed predicate、effect `*` が絡む stdlib API。
- selfhost parser/compiler 実装側の source string fixture。

### 2026-05-24 checkpoint

- branch: `feature/neplg21-syntax-migration-20260524`
- frontend 親 issue: `ISS-20260524T085928069Z-NEPLG2-1-SYNTAX-MIGRATION-NEEDS-FRON-7058CE30`
- doc 親 issue: `ISS-20260524T085928137Z-README-AND-DOCS-MUST-DISTINGUISH-NEP-20719BBC`
- mechanical migration:
  - `nodesrc/neplg21_syntax_migrate.js` を追加し、balanced `<TypeExpr>` 型注釈、function/lambda 引数外形、doc/test corpus の大半を `%` / `\` 構文へ移行した。
  - migrator は string literal 内を変換対象から外し、関数を返す旧関数型は戻り値側を grouping して変換する。
  - `#intrinsic "..." <...>` は compiler-owned directive syntax として保持し、`#extern` signature は `%...` を受理する。
  - `node nodesrc/neplg21_syntax_migrate.js --check` は idempotence check として通過している。
- remaining:
  - executable/comment/doc を合わせた generic postfix 形式はまだ多数残る。これは単純削除ではなく、呼び出し先 signature と expected type を見て `%T` 注釈を補う semantic rewrite として継続する。
  - selfhost source string fixture には旧構文の期待文字列が残る。これは selfhost parser の NEPLg2.1 対応と同じ単位で更新する。

### 2026-05-24 core helper postfix removal checkpoint

- `stdlib/core/option.nepl` と `stdlib/core/result.nepl` の helper 本体で、戻り値期待型から解決できる `Option<.T>::Some` / `Result<.T,.E>::Ok` / `some<.U>` / `err<.U,.E>` などを postfix なしへ移行した。
- 呼び出し側の `map<i32,i32>` / `and_then<i32,i32,str>` などはまだ撤廃していない。直接 smoke では、呼び出し側 postfix を外した `map a inc` / `and_then r f` は stack extra values で失敗するため、expected type 伝播または call candidate reduction 改良が必要である。
- `node nodesrc/tests.js -i stdlib/tests/option.n.md -i stdlib/tests/result.n.md --no-tree -o tmp/neplg21-option-result-smoke.json -j 1 --assert-io` は doctest compile timeout になったため、stdin direct `nepl-cli --check --target core` で helper 内部の型推論を確認した。

### 2026-05-24 call-site postfix removal blocker checkpoint

- 呼び出し側 postfix 撤廃の blocker は、`map a inc` の末尾 `inc` が未適用関数として open call に残り、後ろから優先する call reduction がそこで停止して外側の unresolved overloaded `map` まで戻れないことだった。
- `find_outer_function_consumer` は unresolved overload を常に除外していたため、`Option.map` / `Result.map` のように第 2 引数が関数型である候補を持つ外側 call を、関数値引数の consumer として扱えなかった。
- unresolved overload についても候補 signature の該当引数位置が function type なら外側 call を reduction 対象に戻すようにし、最終的な overload selection は既存の引数型・期待戻り値制約で行う。
- 回帰として、`let mapped %Option i32 map opt inc` と `let res1 %Result i32 str and_then res0 positive_double` が postfix type args なしで通る Rust test を追加した。

### 2026-05-24 Option/Result and_then call-site checkpoint

- `stdlib/tests/option.n.md` と `stdlib/tests/result.n.md` で、外側 consumer と `%Result` 注釈から解決できる `and_then<...>` call site を `and_then` へ移行した。
- `some<i32>` / `unwrap<i32>` / `is_none<i32>` / `unwrap_ok<i32,i32>` は、この checkpoint では外側 consumer または入力型 evidence として残した。
- focused direct `nepl-cli --check --target core` では、今回の option/result 呼び出し形を含む最小 source が pass した。
- `node nodesrc/run_doctest.js -i stdlib/tests/{option,result}.n.md -n 1 --dist web/dist` は 240s timeout になったため、既知の stdlib doctest 長時間化として残し、この差分固有の型推論失敗は確認されていない。

### 2026-05-24 Option/Result test fixture postfix reduction checkpoint

- `stdlib/tests/option.n.md` と `stdlib/tests/result.n.md` で、payload / local annotation / function return annotation から具体化できる `some<T>` / `none<T>` / `ok<T,E>` / `err<T,E>` / `unwrap*<...>` の postfix を追加で撤廃した。
- `is_none none`、`is_err ok 5`、`is_ok err 7` のように constructor 側だけでは type parameter が片側未確定になる observer call は、外側 `is_*<...>` を consumer evidence として残した。
- codegen smoke では、`is_none none` / `is_err ok 5` / `is_ok err 7` まで同時に postfix なしへすると未具体化 generic function が wasm codegen に到達するため、`ISS-20260524T123402690Z-GENERIC-CALLS-WITH-UNCONSTRAINED-TYP-DD4E3093` で型推論/診断改善として分離した。
- focused direct `nepl-cli --check --target core` と selected codegen smoke では、今回残した consumer evidence つきの option/result removed forms が pass した。

### 2026-05-24 Option/Result test fixture final postfix cleanup checkpoint

- `ISS-20260524T123402690Z-GENERIC-CALLS-WITH-UNCONSTRAINED-TYP-DD4E3093` により、未具体化 generic call は codegen ではなく type diagnostic で止まるようになった。
- `stdlib/tests/option.n.md` では `none` を `%Option i32` typed local に置いてから `is_none` / `is_some` に渡し、observer 側の `is_*<i32>` consumer evidence を撤廃した。
- `stdlib/tests/result.n.md` では既存の `%Result i32 i32` typed local `r1` / `e1` を `is_err` / `is_ok` の入力として再利用し、`is_err<i32,i32> ok 5` / `is_ok<i32,i32> err 7` を撤廃した。
- `rg -n "<" stdlib/tests/option.n.md stdlib/tests/result.n.md` は 0 件になった。
- direct `nepl-cli.exe --check` smoke で、更新後の option/result fixture 形はどちらも pass した。

### 2026-05-24 String byte/find postfix cleanup checkpoint

- `stdlib/tests/string.n.md` の `byte_at` / `find` result を `%Option i32` typed local に置き、`unwrap_or<i32>` / `is_none<i32>` を postfix なしの `unwrap_or` / `is_none` へ移行した。
- `std/test` pipe argument 内の nested call から typed local へ分けることで、semantic evidence を明示しながら call reduction の探索も小さくした。
- `builder_result` の `%Result str str` local annotation で戻り値が固定される `Result<str,str>::Err e` 4 箇所も `Result::Err e` へ移行した。
- direct `nepl-cli.exe --check` smoke で、`byte_at` / `find` の更新後 local binding 形は pass した。
- direct `nepl-cli.exe --check` smoke で、postfix-free `builder_result` の nested match 形は pass した。
- `std/test` つき完全 `string_byte_at` smoke は旧 postfix 形でも 180s timeout するため、この checkpoint 固有の regression ではない既知の長時間化として扱った。

### 2026-05-24 core char Result constructor postfix cleanup checkpoint

- `stdlib/core/char.nepl` の `Result<char,str>::Ok` / `Result<char,str>::Err` / `Result<i32,str>::Ok` / `Result<i32,str>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `char_from_i32_result` の `cast code` は overload ambiguous になるため、generic postfix ではなく `%char cast code` の値 ascription で戻り値型 evidence を明示した。
- direct `nepl-cli.exe --check --target core` smoke で、`char_from_i32_result` と `char_to_u8_result` の match 利用はどちらも pass した。

### 2026-05-24 JSON accessor postfix cleanup checkpoint

- `stdlib/alloc/encoding/json/access.nepl` の `json_as_bool` / `json_as_number` / `json_as_string` / `json_as_array` / `json_as_object` 本体で、戻り値型から解ける `some<T>` / `none<T>` を postfix なしへ移行した。
- 同ファイルの doctest 例に残っていた `is_some<bool>` も `is_some` へ移行した。
- direct `nepl-cli.exe --check` smoke で、bool/number/string accessor と array/object accessor の postfix-free return は pass した。

### 2026-05-24 string access/search Option postfix cleanup checkpoint

- `stdlib/alloc/string/access.nepl` の `byte_at` 本体で、戻り値 `%Option i32` から確定できる `some<i32>` / `none<i32>` を postfix なしへ移行した。
- `stdlib/alloc/string/byte_index.nepl` の `checked_string_byte_index` / `checked_string_byte_at` / `string_bytes_cmp` 本体で、戻り値型から確定できる `some<T>` / `none<T>` を postfix なしへ移行した。
- `stdlib/alloc/string/find.nepl` の `find` 本体で、戻り値 `%Option i32` と `out %Option i32` local から確定できる `some<i32>` / `none<i32>` を postfix なしへ移行した。
- doctest コメント内の `is_none<i32>` 例は、nested generic consumer に頼らず `%Option i32` local を置いて `is_none` に渡す形へ移行した。
- `node nodesrc/tests.js` の対象 file runner では `access.nepl` が 1/1 pass、`byte_index.nepl` が 5/5 pass。`find.nepl` は std/test 付き doctest が compile timeout after 60000ms になったため、direct `nepl-cli.exe --check --target core` smoke で `byte_at` / `checked_string_byte_at` / `string_bytes_cmp` / `find` の postfix-free shape を確認した。

### 2026-05-24 JSON Copy Option observer postfix cleanup checkpoint

- `stdlib/tests/json.n.md` の `json_as_bool` / `json_as_number` 結果を `%Option bool` / `%Option i32` local に置き、Copy payload に限って `is_none<bool>` / `is_none<i32>` を postfix なしの `is_none` へ移行した。
- `json_as_string` の `Option str` observer 2 箇所は、owner-bearing payload を Copy payload checkpoint に混ぜないため残した。
- `node nodesrc/tests.js -i stdlib/tests/json.n.md --no-tree -o tmp/json-generic-postfix.json -j 1 --dist web/dist --assert-io` は compile timeout after 60000ms。`tmp/neplg21_json_is_none_copy_smoke.neplg2` の direct `nepl-cli.exe --check --target std` で今回の call shape は pass した。

## 検証

Run stdlib/source policy tests, trunk build, and nodesrc CLI JSON tests after migration.
