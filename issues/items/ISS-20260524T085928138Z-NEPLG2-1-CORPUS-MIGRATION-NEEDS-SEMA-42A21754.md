---
id: ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754
title: "NEPLg2.1 corpus migration needs semantic generic rewrite"
area: stdlib
status: open
resolved: false
priority: P0
type: maintenance
created: 2026-05-24
updated: 2026-05-26
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

### 2026-05-24 string Result constructor postfix cleanup checkpoint

- `stdlib/alloc/string/concat.nepl`、`stdlib/alloc/string/builder/build.nepl`、`stdlib/alloc/string/storage.nepl` で、関数戻り値の `Result ... str` から型が確定する `Result<T,E>::Ok` / `Result<T,E>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- raw copy、`RegionToken<u8>` ownership、`ByteBuilder -> ByteBuf -> str` finalization の処理順や owner boundary は変更していない。
- `tmp/neplg21_string_result_constructor_smoke.neplg2` の direct `nepl-cli.exe --check --target std` で `concat_result` / `sb_build_result` / `string_from_utf8_mem_result` の postfix-free constructor shape は pass した。対象 3 file の `nodesrc/tests.js` は runnable doctest が無く `nodesrc/tests/no-runnable-doctests` になった。

### 2026-05-24 string builder Result constructor postfix cleanup checkpoint

- `stdlib/alloc/string/builder/append.nepl`、`stdlib/alloc/string/builder/reserve.nepl`、`stdlib/alloc/string/builder_ext.nepl` で、関数戻り値の `Result StringBuilder str` / `Result i32 str` から型が確定する `Result<T,E>::Ok` / `Result<T,E>::Err` を postfix なしへ移行した。
- `string_builder_into_byte_builder`、`string_builder_from_byte_builder`、`byte_builder_error_free`、invalid char/byte/slice failure cleanup の処理順は変更していない。
- `nodesrc/source_policy/stdlib_builder_owner.js` と `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` の関連 regex を NEPLg2.1 `%fn` / `%Type` 記法へ追従し、builder owner boundary の静的検査を戻した。
- `node nodesrc/test_stdlib_builder_owner_boundary.js`、`node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_string_integer_boundary.js` は pass した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は 90 件の stale NEPLg2.0 regex failure を報告したため、全体追従は `ISS-20260524T135842959Z-NEPLG2-1-SOURCE-POLICY-REGEXES-STILL-A09E0B60` として分離した。
- `tmp/neplg21_string_builder_result_constructor_smoke.neplg2` の direct `nepl-cli.exe --check --target std` で今回の builder result constructor shape は pass した。
- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/neplg21_string_builder_result_constructor.json -j 1 --dist web/dist --assert-io` は 300s command timeout。残った node プロセスは停止した。

### 2026-05-25 getting_started tutorial generic postfix cleanup checkpoint

- `tutorials/getting_started/07_option.n.md` で、`%Option i32` local から型が確定する `some<i32>` / `none<i32>` と observer call の `is_some` / `is_none` example を postfix なしへ移行した。
- `tutorials/getting_started/08_result.n.md` と `09_validation_project.n.md` で、関数戻り値や match arm の expected type から型が確定する `Result<T,E>::Ok` / `Result<T,E>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `tutorials/getting_started/18_generics.n.md` で、`identity some 7` / `identity ok 1` と `%Option i32` / `%Result i32 str` annotation の組み合わせにし、observer call の explicit generic postfix を撤廃した。
- `tutorials/getting_started/20_namespace_and_methods.n.md` で、`Option<i32>::Some` / `Option<i32>::None` を `%Option i32` local annotation + `Option::Some` / `Option::None` に移行した。
- prose も `Option<T>` / `Result<T,E>` / `Option<i32>` / `Result<i32,str>` から、NEPLg2.1 の `Option .T` / `Result .T .E` / `Option i32` / `Result i32 str` 表記へ更新した。
- subagent review で `18_generics.n.md` の `%.T` 説明が generic parameter declaration と type annotation を混同していると確認したため、`fn identity <.T: Copy>` で宣言し、型式内では `.T` と参照する説明に直した。
- 対象 5 file の `Ident<...>` generic postfix / old type application prose は `rg` で 0 件になった。
- `node nodesrc/tests.js -i tutorials/getting_started/{07_option,08_result,09_validation_project,18_generics,20_namespace_and_methods}.n.md --no-tree -o tmp/neplg21-tutorial-generic-postfix.json -j 1 --dist web/dist --assert-io` は wasm compile timeout after 60000ms で `07_option` / `08_result` の 2 件まで partial error。これは既知の per-program compile-time issue と同系で、今回差分固有の型エラーは JSON に出ていない。
- `node nodesrc/tests.js -i tutorials/getting_started/18_generics.n.md --no-tree -o tmp/neplg21-tutorial-18-generics.json -j 1 --dist web/dist --assert-io` も wasm compile timeout after 60000ms。subagent review では型 evidence の形自体は妥当と確認しており、green doctest は性能 issue 側の回復後に再確認する。

### 2026-05-26 getting_started Result constructor / small Vec observer checkpoint

- `tutorials/getting_started/10_string_and_text.n.md`、`11_bytebuf_and_text_io.n.md`、`12_char_and_ascii.n.md`、`23_project_config_validator.n.md`、`24_project_byte_output.n.md` で、関数戻り値型または `checks_push` の expected type から型が確定する `Result<T,E>::Ok` / `Result<T,E>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `10_string_and_text.n.md` の `is_err<str,str>` は、入力式 `str_slice_result text 2 3` の戻り値から `Result str str` が確定するため `is_err` へ移行した。
- `13_vec_basics.n.md` では `build_numbers` の戻り値 `Result Vec i32 str`、`v0` / `v1` / `numbers` の `Vec i32` 型、`expect_item` の `&Vec i32` 引数から解ける `push<i32>` / `vec_push_error_vec<i32>` / `get<i32>` / `len<i32>` / `free<i32>` を postfix なしへ移行した。`new<i32>` は値引数がなく型根拠が弱いため残した。
- `14_collection_reads.n.md` では `filled` の value 引数 `7` と `has_at` / `values` の `Vec i32` evidence から `filled<i32>` / `get<i32>` / `len<i32>` / `free<i32>` を postfix なしへ移行し、prose の `&Vec<T>` も `&Vec .T` 表記に更新した。
- `16_drop_and_cleanup.n.md` では `new<i32>` から得た `values: Vec i32` に対する `len<i32>` / `free<i32>` を postfix なしへ移行した。`new<i32>` は型根拠が弱いため残した。
- 対象 8 file について `Result<` / `Vec<` / `&Vec<` / `is_err<` / `is_ok<` / `unwrap_ok<` / `ok<` / `err<` / `some<` / `none<` / `push<` / `get<` / `len<` / `free<` / `vec_push_error_vec<` は 0 件になった。ただし `13_vec_basics.n.md` と `16_drop_and_cleanup.n.md` の `new<i32>` は上記理由で残している。
- `node nodesrc/tests.js` による対象 doctest verification は、`10` / `11` / `12` / `13` / `14` / `16` で wasm compile timeout after 60000ms。timeout JSON に型診断は出ていないため、既存の per-program compile-time issue 側で継続確認する。

### 2026-05-26 examples BF small Vec observer checkpoint

- `examples/bf.nepl` で、関数戻り値 `Result Vec i32 StdErrorKind` と `value: i32` から型が確定する `v::filled<i32>` を `v::filled` へ移行した。
- `current_cell` / `jump_target` では、引数型 `&Vec i32` から型が確定する `v::get<i32>` を `v::get` へ移行した。
- コメント内の `Vec<i32>` は、NEPLg2.1 の prefix 型式に合わせて `Vec i32` へ更新した。
- `compile_jumps` 内の `err<Vec<i32>, str>` / `ok<Vec<i32>, str>` / `Stack<i32>` owner recovery 周辺は、`Stack` と `Result` の owner state が密に絡むため今回の小 checkpoint では残した。
- `node nodesrc/tests.js -i examples/bf.nepl --no-tree -o tmp/neplg21-example-bf-small-vec.json -j 1 --dist web/dist --assert-io` は 2 doctest とも wasm compile timeout after 60000ms。
- `target\debug\nepl-cli.exe --check -i examples\bf.nepl --target std` と、`v::filled` / `v::get` の最小 smoke check は長時間化により timeout。残留 `nepl-cli` プロセスは停止した。

### 2026-05-26 fs Vec str observer checkpoint

- subagent 監査により、allowlist 外の positive executable corpus にはまだ多数の explicit generic postfix が残るため、この issue は main merge blocker のまま継続すると確認した。
- `tests/stdlib/fs.n.md` の directory entry tests で、`fs_read_dir` の戻り値 `Result Vec str i32` と `entries %Vec str` local annotation から型が確定する `v::len<str>` / `v::get<str>` / `v::free<str>` を postfix なしへ移行した。
- 同じ call chain の `checks_push` は `Result () str` を受けるため、該当 block の `Result<(),str>::Err` を `Result::Err` へ移行した。
- ファイル前半の `Result<(),str>::Err` は別テストの error aggregation であり、今回の directory entry Vec checkpoint には混ぜず残した。
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/neplg21-fs-vec-str-postfix.json -j 1 --dist web/dist --assert-io` は 240s command timeout。partial JSON では変更箇所前の doctest#1-#4 が wasm compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 hash Vec i32 observer checkpoint

- `stdlib/tests/hash.n.md` で、`sha256_digest_matches_loop` の引数 `digest: &Vec i32` から型が確定する `get<i32>` を `get` へ移行した。
- `sha256_push_digest_checks` では `Result::Ok digest` の payload 型が `Vec i32` と分かるため、`len<i32> &digest` / `free<i32> digest` を postfix なしへ移行した。
- 同じ検証中に、SHA256 実装側が stale な `VecPushError<T>.vec` field を直接読んでいた問題が露出したため、`ISS-20260525T214844057Z-SHA256-STILL-READS-STALE-VECPUSHERRO-77799ACC` として分離し、accessor 経由へ修正した。
- `rg -n "get<i32>|len<i32>|free<i32>" stdlib/tests/hash.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md --no-tree -o tmp/neplg21-hash-vecpusherror-field-240s.json -j 1 --dist web/dist --assert-io` は 240s compile timeout。SHA256 の `type.field.invalid_access` は再発せず、残る full doctest green 化は `ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5` 側で継続する。

### 2026-05-26 vec_main basic observer checkpoint

- `stdlib/tests/vec.n.md` の `vec_main` 冒頭で、`&Vec i32` / `&Vec u8` または owned `Vec` 引数から型が確定する `is_empty<i32>`、`len<i32>`、`get<i32>`、`get<u8>`、`replace<i32>`、`free<i32>`、`free<u8>` を postfix なしへ移行した。
- `is_none<i32> get<i32> ...` の nested call は、型推論の探索を無駄に増やさないよう `missing %Option i32` typed local に分け、`is_none missing` へ移行した。
- `new<i32>` / `with_capacity<i32>` / `push<i32>` / `push<u8>` は producer/update 側の推論 checkpoint として別に扱うため、今回は残した。
- subagent review でも同じ範囲が推奨され、producer/update call は混ぜない方針で一致した。
- `Get-Content stdlib/tests/vec.n.md | Select-Object -First 115 | rg -n "is_empty<i32>|len<i32>|get<i32>|replace<i32>|free<i32>|get<u8>|free<u8>|is_none<i32>"` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/neplg21-vec-main-basic-observer.json -j 1 --dist web/dist --assert-io` は 120s local command timeout。partial JSON では doctest#1/#2 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。
- `target\debug\nepl-cli.exe --check -i tmp\neplg21_vec_basic_observer_smoke.neplg2 --target core` は memory allocation failure。これは `ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5` 側の compile-time / memory budget 問題として扱う。

### 2026-05-26 vec_collections observer checkpoint

- `tests/stdlib/vec_collections.n.md` で、`Vec i32` local または `&Vec i32` 引数から型が確定する `is_empty<i32>`、`cap<i32>`、`len<i32>`、`get<i32>`、`free<i32>` を postfix なしへ移行した。
- `with_capacity<i32>`、`new<i32>`、`push<i32>`、`sort_merge_ret<i32>` は producer/update/sort 側の推論 checkpoint として別に扱うため、今回は残した。
- `rg -n "is_empty<i32>|cap<i32>|len<i32>|get<i32>|free<i32>" tests/stdlib/vec_collections.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/neplg21-vec-collections-observers.json -j 1 --dist web/dist --assert-io` は 150s local command timeout。partial JSON では doctest#1/#2 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 small stdlib Result constructor checkpoint

- `tests/stdlib/std_test_collect.n.md` で、`assert_ok_i32` / `assert_err_i32` の引数型から `Result i32 i32` が確定する `Result<i32,i32>::Ok` / `Result<i32,i32>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `tests/stdlib/io.n.md` で、`checks_push` が `Result () str` を受ける箇所の `Result<(),str>::Err` を `Result::Err` へ移行した。
- `rg -n "Result<i32,i32>::(Ok|Err)|Result<\\(\\),str>::Err" tests/stdlib/std_test_collect.n.md tests/stdlib/io.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i tests/stdlib/io.n.md --no-tree -o tmp/neplg21-result-constructor-small-tests.json -j 1 --dist web/dist --assert-io` は 150s local command timeout。partial JSON では `std_test_collect` doctest#1/#2 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 string_char Result constructor and is_err observer checkpoint

- `tests/stdlib/string_char.n.md` で、helper の戻り値型 `Result () str` / `Result str str` から型が確定する `Result<(),str>::Err`、`Result<str,str>::Err`、`Result<str,str>::Ok` を `Result::Err` / `Result::Ok` へ移行した。
- `byte_builder_text` の error cleanup branch は、cleanup 呼び出し順を変えず constructor 表記だけを変更した。
- `is_err<char,str>`、`is_err<str,str>`、`is_err<CharUtf8Step,str>` は、入力の `str_char_at_result` / `str_slice_chars_result` / `str_next_char_result` の戻り値型から `Result` 型が確定するため `is_err` へ移行した。
- subagent review では constructor 全件が戻り値型または `bytes_check %Result () str` の expected type から安全に移行でき、残すべき弱い constructor はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)|is_err<" tests/stdlib/string_char.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/neplg21-string-char-result-constructors.json -j 1 --dist web/dist --assert-io` は 150s local command timeout。partial JSON では doctest#1/#2 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 text_utf8 Result constructor and is_err observer checkpoint

- `tests/stdlib/text_utf8.n.md` で、`checks_push` の expected type `Result () str`、helper 戻り値型 `Result () str`、typed local `ok %Result () str` から型が確定する `Result<(),str>::Ok` / `Result<(),str>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `is_err<CharUtf8Step,StdErrorKind>` は、入力式 `text_utf8_decode_next ...` の戻り値 `Result CharUtf8Step StdErrorKind` から型が確定するため `is_err` へ移行した。
- raw region 系の branch は `store_u8` / `region_ptr_at` 失敗時に `dealloc_region` 後 `checks_push` へ進む順序を維持し、constructor 表記だけを変えた。
- subagent review でも、現行差分全体は expected type が見えており、cleanup 順に影響しないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)|is_err<" tests/stdlib/text_utf8.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/neplg21-text-utf8-result-constructors.json -j 1 --dist web/dist --assert-io` は 150s local command timeout。partial JSON では doctest#1/#2 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 traits_serde Result constructor checkpoint

- `tests/stdlib/traits_serde.n.md` で、`checks_push` の expected type `Result () str` から型が確定する `Result<(),str>::Ok` / `Result<(),str>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `deserialize<i32>` / `deserialize<bool>` は match 入力側の型根拠を別に確認する必要があるため、今回の constructor checkpoint には混ぜず残した。
- `rg -n "Result<[^>]+>::(Ok|Err)|is_err<|is_ok<" tests/stdlib/traits_serde.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/traits_serde.n.md --no-tree -o tmp/neplg21-traits-serde-result-constructors.json -j 1 --dist web/dist --assert-io` は `doctest#1/#2` とも compile timeout after 60000ms。型診断は出ていない。

## 検証

Run stdlib/source policy tests, trunk build, and nodesrc CLI JSON tests after migration.
