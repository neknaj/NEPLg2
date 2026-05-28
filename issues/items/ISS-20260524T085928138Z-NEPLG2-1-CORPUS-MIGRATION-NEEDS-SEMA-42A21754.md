---
id: ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754
title: "NEPLg2.1 corpus migration needs semantic generic rewrite"
area: stdlib
status: open
resolved: false
priority: P0
type: maintenance
created: 2026-05-24
updated: 2026-05-28
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

### 2026-05-26 unit keyword migration checkpoint

- `()` は NEPLg2.1 の unit 型・unit 値・0 引数関数 marker として残さず、`unit` keyword へ移行する。
- `fn unit T` は unit 型引数を 1 個取る関数ではなく、0 引数で `T` を返す関数として frontend が既存の空 `params` へ正規化する。
- `\unit` も通常 parameter 名ではなく、0 引数関数リテラルの marker として扱う。
- `nodesrc/neplg21_syntax_migrate.js` は `<()>`、`%()`、`fn () T`、`\()`、値式の `()` を `unit` へ変換する。ただし `#intrinsic "..." (...)` の括弧は directive の引数区切りであり、unit 値ではないため保持する。
- explicit generic postfix がまだ残る箇所では、移行完了まで transitional に `unit` を旧 type parser でも Unit として受理する。postfix そのものの撤廃はこの issue の semantic rewrite として継続する。

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

### 2026-05-26 traits_hash Result constructor / helper postfix checkpoint

- `tests/stdlib/traits_hash.n.md` で、`checks_push` の expected type `Result () str` から型が確定する `Result<(),str>::Err` を `Result::Err` へ移行した。
- `unwrap_ok<HashSet<...>, Diag> r` は、引数 `r` の型 `Result HashSet ... Diag` から型引数が確定するため `unwrap_ok r` へ移行した。
- `hashmap_update_error_owner<...> e` / `hashset_update_error_owner<...> e` は、`Err e` の payload 型と `%HashMap` / `%HashSet` local annotation が一致しているため postfix なしへ移行した。
- `use_hasher_twice<i32, StatefulHasher>` は `.K` が値引数に現れず `.H: Hasher<.K>` bound 経由でしか決まらないため、trait bound 逆推論の別 checkpoint として残した。
- `rg -n "Result<[^>]+>::(Ok|Err)|unwrap_ok<|hashmap_update_error_owner<|hashset_update_error_owner<" tests/stdlib/traits_hash.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --no-tree -o tmp/neplg21-traits-hash-postfix.json -j 1 --dist web/dist --assert-io` は 6 件中 3 pass、`doctest#1` が compile timeout after 60000ms、`doctest#5/#6` が既存の `new` overload no_match 系 compile failure。今回移行した helper call に対する型診断は出ていない。

### 2026-05-26 stdlib json Result constructor checkpoint

- `stdlib/tests/json.n.md` で、`checks_push` の expected type `Result () str` から型が確定する `Result<(),str>::Err` を `Result::Err` へ移行した。
- `is_none<str> json_as_string ...` は owner-bearing `Option str` の observer reshape であり、Result constructor checkpoint には混ぜず残した。
- subagent review でも、対象 4 箇所は `checks_push` の `TestReport -> Result () str -> TestReport` overload により型根拠が十分で、`is_none<str>` は別 checkpoint 推奨と確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/tests/json.n.md` は 0 件になった。
- `node nodesrc/test_stdlib_json_nmd_report_contract.js` は pass。
- `node nodesrc/tests.js -i stdlib/tests/json.n.md --no-tree -o tmp/neplg21-stdlib-json-result-constructors.json -j 1 --dist web/dist --assert-io` は compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 collections_diag helper postfix checkpoint

- `tests/stdlib/collections_diag.n.md` で、`checks_push` の expected type `Result () str` から型が確定する `Result<(),str>::Ok` / `Result<(),str>::Err` を `Result::Ok` / `Result::Err` へ移行した。
- `hashmap_update_error_diag<...>` / `hashset_update_error_diag<...>` と `hashmap_update_error_owner<...>` / `hashset_update_error_owner<...>` は、`Err e` の payload 型と `%HashMap` / `%HashSet` local annotation から型が決まるため postfix なしへ移行した。
- `unwrap_ok<...>` は、代入先の `%HashMap` / `%HashSet` / `%Queue` / `%RingBuffer` annotation がある行に限って postfix なしへ移行した。
- `new<i32>` / `pop<i32>` は producer 側の型引数であり、今回の helper postfix checkpoint には混ぜず残した。
- subagent review でも、constructor、owner helper、typed-local 付き `unwrap_ok` は移行推奨、queue/ringbuffer の `new<i32>` / `pop<i32>` は残すべきと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)|unwrap_ok<|hashmap_update_error_diag<|hashset_update_error_diag<|hashmap_update_error_owner<|hashset_update_error_owner<" tests/stdlib/collections_diag.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/neplg21-collections-diag-helper-postfix.json -j 1 --dist web/dist --assert-io` は 4 件すべて compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 hashset helper postfix checkpoint

- `stdlib/tests/hashset.n.md` / `stdlib/tests/hashset_str.n.md` で、`must_hs` / `must_hss` の引数型 `Result HashSet ... Diag` から型が確定する `unwrap_ok<HashSet<...>, Diag> r` を `unwrap_ok r` へ移行した。
- 同じ 2 file で、`Err e` payload 型と代入先 `%HashSet ...` local annotation から型が確定する `hashset_update_error_owner<...> e` を `hashset_update_error_owner e` へ移行した。
- subagent review でも、`i32` key 版と `str` key 版の helper postfix-free 移行は妥当であり、`new` / `insert` / `remove` など producer/update call とは分けるべきと確認した。
- focused doctest は current diff / `unwrap_ok` explicit comparison / clean HEAD comparison のすべてで `must_hs new DefaultHash32` / `must_hss new DefaultHash32` の `type.overload.no_match` と free smoke compile timeout を再現した。今回移行した helper call 固有の新規 regression ではない。
- nested producer generic call が outer helper parameter expectation を使えない問題は `ISS-20260525T233735956Z-NEPLG2-1-NESTED-PRODUCER-GENERIC-CAL-B1C7C74C` として分離した。
- `rg -n "unwrap_ok<|hashset_update_error_owner<|Result<[^>]+>::(Ok|Err)" stdlib/tests/hashset.n.md stdlib/tests/hashset_str.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-tree -o tmp/neplg21-hashset-helper-postfix-current.json -j 1 --dist web/dist --assert-io` は 4 件中 2 compile failure / 2 compile timeout。compile failure は既存 baseline と同じ `new DefaultHash32` overload no_match 系。

### 2026-05-26 cliarg Option observer checkpoint

- `stdlib/tests/cliarg.n.md` で、`cliarg_get` / `cli_raw::cliarg_get_checked` の戻り値 `Option str` から型が確定する `is_none<str>` を `is_none` へ移行した。
- `new<T>` / `push<T>` のような producer/update call は含めていない。
- `rg -n "is_none<str>" stdlib/tests/cliarg.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-tree -o tmp/neplg21-cliarg-option-observer.json -j 1 --dist web/dist --assert-io` は 180s local command timeout。partial JSON では doctest#1-#3 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 bitset helper postfix checkpoint

- `stdlib/tests/bitset.n.md` / `tests/stdlib/bitset_collections.n.md` で、代入先 `%BitSet` または `new` / `contains` の戻り値から型が確定する `unwrap_ok<BitSet, Diag>` / `unwrap_ok<bool, Diag>` を `unwrap_ok` へ移行した。
- `BitSet` の `new` / `insert` / `remove` は型引数を持たないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "unwrap_ok<BitSet|unwrap_ok<bool|is_err<|is_ok<" stdlib/tests/bitset.n.md tests/stdlib/bitset_collections.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/bitset.n.md -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/neplg21-bitset-helper-postfix.json -j 1 --dist web/dist --assert-io` は 180s local command timeout。partial JSON では `stdlib/tests/bitset.n.md` doctest#1-#3 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 adjacency_matrix helper postfix checkpoint

- `stdlib/tests/adjacency_matrix.n.md` / `tests/stdlib/adjacency_matrix_collections.n.md` で、代入先 `%AdjacencyMatrix` または `new` / `contains` の戻り値から型が確定する `unwrap_ok<AdjacencyMatrix, Diag>` / `unwrap_ok<bool, Diag>` を `unwrap_ok` へ移行した。
- `AdjacencyMatrix` の `new` / `insert` / `remove` は型引数を持たないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "unwrap_ok<AdjacencyMatrix|unwrap_ok<bool|is_err<|is_ok<" stdlib/tests/adjacency_matrix.n.md tests/stdlib/adjacency_matrix_collections.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/adjacency_matrix.n.md -i tests/stdlib/adjacency_matrix_collections.n.md --no-tree -o tmp/neplg21-adjacency-matrix-helper-postfix.json -j 1 --dist web/dist --assert-io` は 180s local command timeout。partial JSON では `stdlib/tests/adjacency_matrix.n.md` doctest#1-#3 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 sparse_set helper postfix checkpoint

- `stdlib/tests/sparse_set.n.md` / `tests/stdlib/sparse_set_collections.n.md` で、代入先 `%SparseSet` または `new` / `contains` の戻り値から型が確定する `unwrap_ok<SparseSet, Diag>` / `unwrap_ok<bool, Diag>` を `unwrap_ok` へ移行した。
- `r0 %Result bool Diag` から型が確定する `is_err<bool, Diag> r0` を `is_err r0` へ移行した。
- `SparseSet` の `new` / `insert` / `remove` は型引数を持たないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "unwrap_ok<SparseSet|unwrap_ok<bool|is_err<|is_ok<" stdlib/tests/sparse_set.n.md tests/stdlib/sparse_set_collections.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md --no-tree -o tmp/neplg21-sparse-set-helper-postfix.json -j 1 --dist web/dist --assert-io` は 180s local command timeout。partial JSON では `stdlib/tests/sparse_set.n.md` doctest#1-#2 と `tests/stdlib/sparse_set_collections.n.md` doctest#1 が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 fenwick helper postfix checkpoint

- `stdlib/tests/fenwick.n.md` で、代入先 `%Fenwick` / `%i32` または `fw::new` / `fw::sum_prefix` / `fw::sum_range` の戻り値から型が確定する `unwrap_ok<Fenwick, Diag>` / `unwrap_ok<i32, Diag>` を `unwrap_ok` へ移行した。
- `Fenwick` の `new` / `add` / `sum_*` は型引数を持たないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "unwrap_ok<Fenwick|unwrap_ok<i32|unwrap_ok<bool|is_err<|is_ok<" stdlib/tests/fenwick.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/fenwick.n.md --no-tree -o tmp/neplg21-fenwick-helper-postfix.json -j 1 --dist web/dist --assert-io` は 2 件とも compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 disjoint_set helper postfix checkpoint

- `stdlib/tests/disjoint_set.n.md` で、代入先 `%DisjointSet` / `%bool` / `%i32` または `new` / `union` / `same` / `size` の戻り値から型が確定する `unwrap_ok<...>` を `unwrap_ok` へ移行した。
- `r0 %Result i32 Diag` / `r1 %Result bool Diag` から型が確定する `is_err<i32, Diag>` / `is_err<bool, Diag>` を `is_err` へ移行した。
- `DisjointSet` の `new` / `union` / observer は型引数を持たないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "unwrap_ok<DisjointSet|unwrap_ok<bool|unwrap_ok<i32|is_err<|is_ok<" stdlib/tests/disjoint_set.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md --no-tree -o tmp/neplg21-disjoint-set-helper-postfix.json -j 1 --dist web/dist --assert-io` は 2 件とも compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 stdio/streamio Result constructor checkpoint

- `tests/stdlib/stdio_read_all.n.md` / `tests/stdlib/streamio.n.md` で、`checks_push` の expected type `Result () str` から型が確定する `Result<(),str>::Err` を `Result::Err` へ移行した。
- `stdio_read_all_bytes_result` / `stdio_write_bytes_result` / `io_bytebuf_alloc_region` / `store_u8` の match branch 内の error aggregation だけを対象にし、producer/update call や nested generic call には踏み込んでいない。
- `rg -n "Result<\\(\\),str>::(Ok|Err)" tests/stdlib/stdio_read_all.n.md tests/stdlib/streamio.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/stdio_read_all.n.md -i tests/stdlib/streamio.n.md --no-tree -o tmp/neplg21-stdio-streamio-result-constructors.json -j 1 --dist web/dist --assert-io` は 190s local command timeout。partial JSON では `tests/stdlib/stdio_read_all.n.md` doctest#1/#2 と `tests/stdlib/streamio.n.md` doctest#1 が compile timeout after 60000ms で、型診断は出ていない。

### 2026-05-26 Fenwick/DisjointSet/SegmentTree collection helper checkpoint

- subagent の独立レビューに従い、producer generic を持たない `Fenwick` / `DisjointSet` / `SegmentTree` の collection doctest helper だけを対象にした。
- `tests/stdlib/fenwick_collections.n.md` / `tests/stdlib/disjoint_set_collections.n.md` / `stdlib/tests/segment_tree.n.md` / `tests/stdlib/segment_tree_collections.n.md` で、代入先 `%Fenwick` / `%DisjointSet` / `%SegmentTree` / `%i32` / `%bool` または戻り値型から型が確定する `unwrap_ok<...>` を `unwrap_ok` へ移行した。
- `r1 %Result i32 Diag` から型が確定する `is_err<i32, Diag> r1` を `is_err r1` へ移行した。
- `new` / `replace` / `add` / `same` / `size` / `sum_range` は対象 collection では関数名側に型引数を持たないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "unwrap_ok<|is_err<|is_ok<|Result<[^>]+>::(Ok|Err)" tests/stdlib/fenwick_collections.n.md tests/stdlib/disjoint_set_collections.n.md stdlib/tests/segment_tree.n.md tests/stdlib/segment_tree_collections.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/fenwick_collections.n.md -i tests/stdlib/disjoint_set_collections.n.md -i stdlib/tests/segment_tree.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/neplg21-nongeneric-collection-helper-postfix.json -j 1 --dist web/dist --assert-io` は 250s local command timeout。partial JSON では `tests/stdlib/fenwick_collections.n.md` doctest#1-#4 が compile timeout after 60000ms で、型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 fs/pipe_collections Result constructor checkpoint

- subagent の独立レビューに従い、`checks_push` の expected type `Result () str` から型が確定する `Result<(),str>::Ok` / `Result<(),str>::Err` だけを対象にした。
- `tests/stdlib/fs.n.md` / `tests/stdlib/pipe_collections.n.md` で、該当 constructor を `Result::Ok` / `Result::Err` へ移行した。
- `pipe_collections` に残る `new<T>` / `push<T>` / `get<T>` / `unwrap_ok<...>` / update helper postfix は producer/update/nested generic 側であり、この constructor checkpoint には混ぜていない。
- `rg -n "Result<\\(\\),str>::(Ok|Err)" tests/stdlib/fs.n.md tests/stdlib/pipe_collections.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/neplg21-fs-pipe-result-constructors.json -j 1 --dist web/dist --assert-io` は 250s local command timeout。partial JSON では `tests/stdlib/fs.n.md` doctest#1-#4 が compile timeout after 60000ms で、型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 BTree error helper checkpoint

- subagent の独立レビューに従い、`Err e` payload と `%Diag` local annotation / `must_map`・`must_set` の戻り型から型が確定する BTree error helper だけを対象にした。
- `stdlib/tests/btreemap.n.md` / `stdlib/tests/btreeset.n.md` / `tests/stdlib/btree_array_cost.n.md` で、`btreemap_insert_error_diag<...>` / `btreemap_insert_error_owner<...>` / `btreeset_insert_error_diag<...>` / `btreeset_insert_error_owner<...>` を postfix なしへ移行した。
- `unwrap_ok<...> new<T>` / `sorted_array_*_new<T>` / `insert<T>` は producer/update/nested generic 側であり、この helper checkpoint には混ぜていない。
- `rg -n "btreemap_insert_error_(diag|owner)<|btreeset_insert_error_(diag|owner)<" stdlib/tests/btreemap.n.md stdlib/tests/btreeset.n.md tests/stdlib/btree_array_cost.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stdlib/btree_array_cost.n.md --no-tree -o tmp/neplg21-btree-error-helper-postfix.json -j 1 --dist web/dist --assert-io` は 250s local command timeout。partial JSON では `stdlib/tests/btreemap.n.md` doctest#1-#4 が compile timeout after 60000ms で、型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 Option i32 observer checkpoint

- `tests/stdlib/string.n.md` で、`byte_at "AZ" 2` を `%Option i32` local に置き、`is_none<i32>` を `is_none` へ移行した。
- `stdlib/tests/hashmap_str.n.md` で、`hm2_got %Option i32` local から型が確定する `is_none<i32> hm2_got` を `is_none hm2_got` へ移行した。
- `byte_at` / `HashMap.get` の producer 本体や `new DefaultHash32` には触れていないため、nested producer generic 推論問題には踏み込んでいない。
- `rg -n "is_none<i32>|is_some<i32>|unwrap_ok<|Result<|Option<|Vec<" tests/stdlib/string.n.md stdlib/tests/hashmap_str.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/string.n.md -i stdlib/tests/hashmap_str.n.md --no-tree -o tmp/neplg21-option-i32-observer-postfix.json -j 1 --dist web/dist --assert-io` は 254s local command timeout。partial JSON では `tests/stdlib/string.n.md` doctest#1-#4 が compile timeout after 60000ms で、型診断は出ていない。
- `node nodesrc/run_doctest.js -i tests/stdlib/string.n.md -n 8 --dist web/dist` と `node nodesrc/run_doctest.js -i stdlib/tests/hashmap_str.n.md -n 1 --dist web/dist` も 94s local command timeout。残留 node process は停止した。

### 2026-05-26 collection API doc helper checkpoint

- subagent の独立レビューに従い、既に test fixture 側を処理済みの non-generic collection family について、stdlib API doc example 側の `unwrap_ok<...>` を `unwrap_ok` へ移行した。
- 対象は `stdlib/alloc/collections/adjacency_matrix/**` / `bitset/**` / `sparse_set/**` / `fenwick/**` / `disjoint_set/**`、`stdlib/kp/kpfenwick.nepl`、`tests/stdlib/std_test_namespace_resolution.n.md`。
- 代入先 `%AdjacencyMatrix` / `%BitSet` / `%SparseSet` / `%Fenwick` / `%DisjointSet` / `%bool` / `%i32`、または引数 `Result T E` から型が確定する helper call だけを対象にした。
- `new<T>` / `push<T>` / sort / trait-bound 逆推論 / 実装本体の constructor helper には触れていない。
- `rg -n "unwrap_ok<|is_err<|is_ok<|Result<[^>]+>::(Ok|Err)" stdlib/alloc/collections/adjacency_matrix stdlib/alloc/collections/bitset stdlib/alloc/collections/sparse_set stdlib/alloc/collections/fenwick stdlib/alloc/collections/disjoint_set stdlib/kp/kpfenwick.nepl tests/stdlib/std_test_namespace_resolution.n.md` は 0 件になった。
- 独立レビューでも、今回外した `unwrap_ok<...>` はすべて `let x %Type` / `let x %Type:` と `Result T E` 引数から型が確定するため安全であり、指定範囲に `is_ok<...>` / `is_err<...>` / `Result<...>::Ok|Err` 候補はないと確認した。
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix/api -i stdlib/alloc/collections/bitset/api -i stdlib/alloc/collections/sparse_set/api -i stdlib/alloc/collections/fenwick/api -i stdlib/alloc/collections/disjoint_set/api -i stdlib/kp/kpfenwick.nepl -i tests/stdlib/std_test_namespace_resolution.n.md --no-tree -o tmp/neplg21-collection-api-doc-helper-postfix.json -j 1 --dist web/dist --assert-io` は 254s local command timeout。partial JSON では `adjacency_matrix/api` doctest#1 系が compile timeout after 60000ms で、型診断は出ていない。残留 node process は停止した。

### 2026-05-26 neplg2_diag_outcome Result constructor checkpoint

- `tests/stdlib/neplg2_diag_outcome.n.md` で、`checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Err` 11 件を `Result::Err` へ移行した。
- outcome API の `selfhost_outcome_ok<i32,str>` / `selfhost_outcome_push_diagnostic<i32,str>` / `selfhost_outcome_result<i32,str>` は、constructor ではなく関数呼び出し側の generic evidence なのでこの checkpoint には混ぜず残した。
- subagent の独立レビューでも、対象はすべて `checks_push` overload の `Result unit str` 引数に乗っており、source string fixture や型 evidence が弱い箇所ではないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_diag_outcome.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/neplg21-diag-outcome-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 neplg2_checker Result constructor checkpoint

- `tests/stdlib/neplg2_checker.n.md` で、`checks_push` の expected type `Result unit str` または helper 戻り値 `Result unit str` から型が確定する `Result<unit,str>::Err` 12 件を `Result::Err` へ移行した。
- source string fixture 内の旧 `fn main <()->i32> ():` は、selfhost checker の入力文字列として旧構文の扱いを確認する箇所なので構文移行していない。
- doctest#4 で外側 NEPL 文字列 literal の `\()` が `lexer.string.invalid_escape` になることが分かったため、source string の中身を `\()` のまま保つために literal backslash だけを `\\()` として escape した。
- subagent の独立レビューでも、対象 12 件は producer/nested generic 推論に絡まず、`\\()` escape は source string fixture の構文移行ではなく外側文字列の root-cause test fix と確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_checker.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/neplg21-checker-result-constructors.json -j 1 --dist web/dist --assert-io` は 4 件すべて compile timeout after 60000ms。`lexer.string.invalid_escape` は解消し、型診断は出ていない。

### 2026-05-26 import spec / impl visibility Result constructor checkpoint

- `tests/stdlib/neplg2_import_spec.n.md` と `tests/stdlib/neplg2_checker_impl_visibility.n.md` で、`checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Err` 6 件を `Result::Err` へ移行した。
- `neplg2_checker_impl_visibility.n.md` の source string fixture は selfhost parser/checker の入力文字列であり、今回の Result constructor checkpoint では構文移行していない。
- subagent の独立レビューでも、対象 6 件は producer/nested generic 推論に絡まず、残すべき `Result<unit,str>::Err` はないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_import_spec.n.md tests/stdlib/neplg2_checker_impl_visibility.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_checker_impl_visibility.n.md --no-tree -o tmp/neplg21-import-spec-impl-visibility-result-constructors.json -j 1 --dist web/dist --assert-io` は 4 件すべて compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 neplg2_parser Result constructor checkpoint

- `tests/stdlib/neplg2_parser.n.md` で、helper 戻り値 `Result unit str` または `checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 11 件を `Result::Ok` / `Result::Err` へ移行した。
- source string fixture と期待 lexeme の旧 `fn add <(i32,i32)->i32> (a,b):` は、selfhost parser の入力/期待値としてこの checkpoint では構文移行していない。
- subagent の独立レビューでも、対象 11 件は producer/nested generic 推論に絡まず、残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_parser.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md --no-tree -o tmp/neplg21-parser-result-constructors.json -j 1 --dist web/dist --assert-io` は 1 件 compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 neplg2_module_loader Result constructor checkpoint

- `tests/stdlib/neplg2_module_loader.n.md` で、`checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Err` 7 件を `Result::Err` へ移行した。
- source string fixture の旧 `fn main <()->i32> ():` / `fn helper <()->i32> ():` は、module loader の入力文字列としてこの checkpoint では構文移行していない。
- subagent の独立レビューでも、対象 7 件は producer/nested generic 推論に絡まず、残すべき `Result<unit,str>::Err` はないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_module_loader.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg21-module-loader-result-constructors.json -j 1 --dist web/dist --assert-io` は 2 件すべて compile timeout after 60000ms。型診断は出ていない。

### 2026-05-26 neplg2_stdlib_map Result constructor checkpoint

- `tests/stdlib/neplg2_stdlib_map.n.md` で、`checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Err` 7 件を `Result::Err` へ移行した。
- source string fixture の旧 `fn main <()->i32> \():` / `fn util <()->i32> ():` は、stdlib map の入力文字列としてこの checkpoint では構文移行していない。
- 外側 NEPL 文字列 literal の `\()` は、source string の中身を `\()` のまま保つために literal backslash だけを `\\()` として escape した。
- subagent の独立レビューでも、対象 7 件は producer/nested generic 推論に絡まず、残すべき `Result<unit,str>::Err` はないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_stdlib_map.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_stdlib_map.n.md --no-tree -o tmp/neplg21-stdlib-map-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断や `lexer.string.invalid_escape` は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 std/test Result constructor checkpoint

- `stdlib/std/test/assertion.nepl` で、関数戻り値 `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 14 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/std/test/report.nepl` で、`finish_checks` の戻り値 `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 2 件を `Result::Ok` / `Result::Err` へ移行した。
- source string や doc comment だけの `Result<unit,str>` 表記は、この constructor cleanup checkpoint では対象外として保持した。
- subagent の独立レビューでも、対象 16 件は戻り値型または if/match branch expected type が `Result unit str` で明確で、残すべき typed constructor はないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" stdlib/std/test/report.nepl stdlib/std/test/assertion.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/test/assertion.nepl -i stdlib/std/test/report.nepl --no-tree -o tmp/neplg21-std-test-result-constructors.json -j 1 --dist web/dist --assert-io` は helper module 直指定のため `nodesrc/tests/no-runnable-doctests`。
- `node nodesrc/tests.js -i tests/stdlib/std_test_namespace_resolution.n.md --no-tree -o tmp/neplg21-std-test-namespace-result-constructors.json -j 1 --dist web/dist --assert-io` は 1 件 compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 core/mem and alloc/string Result constructor checkpoint

- `stdlib/core/mem/allocator.nepl` / `stdlib/core/mem/pointer/scalar.nepl` / `stdlib/core/mem/pointer/region.nepl` / `stdlib/core/mem/pointer/bulk.nepl` で、関数戻り値 `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 19 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/alloc/string/float/format.nepl` と `stdlib/alloc/string/utf8.nepl` で、関数戻り値 `Result unit str`、または `%Result unit str` local annotation / `set` 先変数型から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 30 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent を並列に使い、`core/mem` と `alloc/string` を別々に独立レビューした。どちらも残すべき typed constructor はないと確認した。
- doc comment/source string だけの対象 constructor は、この checkpoint には含まれていない。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" stdlib/core/mem/allocator.nepl stdlib/core/mem/pointer/scalar.nepl stdlib/core/mem/pointer/region.nepl stdlib/core/mem/pointer/bulk.nepl stdlib/alloc/string/utf8.nepl stdlib/alloc/string/float/format.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/core/mem/allocator.nepl -i stdlib/core/mem/pointer/scalar.nepl -i stdlib/core/mem/pointer/region.nepl -i stdlib/core/mem/pointer/bulk.nepl -i stdlib/alloc/string/utf8.nepl -i stdlib/alloc/string/float/format.nepl --no-tree -o tmp/neplg21-core-mem-string-result-constructors.json -j 1 --dist web/dist --assert-io` は 264s local command timeout。partial JSON では 10/19 件 pass、failed/errored 0、型診断は出ていない。
- `node nodesrc/tests.js -i stdlib/core/mem/pointer/bulk.nepl --no-tree -o tmp/neplg21-core-mem-bulk-result-constructors.json -j 1 --dist web/dist --assert-io` は 2 件 pass。
- `node nodesrc/tests.js -i stdlib/alloc/string/utf8.nepl -i stdlib/alloc/string/float/format.nepl --no-tree -o tmp/neplg21-alloc-string-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 neplg2_module_graph Result constructor checkpoint

- `tests/stdlib/neplg2_module_graph.n.md` で、`checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Err` 14 件を `Result::Err` へ移行した。
- source string fixture の旧 `fn ... <()->i32> ...` は、module graph の入力文字列としてこの checkpoint では構文移行していない。
- 外側 NEPL 文字列 literal の `\()` は、source string の中身を `\()` のまま保つために literal backslash だけを `\\()` として escape した。
- subagent の独立レビューでも、対象 14 件は `checks_push` overload の `Result unit str` 引数に乗っており、残すべき `Result<unit,str>::Err` はないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_module_graph.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg21-module-graph-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断や `lexer.string.invalid_escape` は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 proof fixture Result constructor checkpoint

- `tests/stdlib/neplg2_effect_proof.n.md` で、helper 戻り値 `Result unit str` と match branch expected type から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 44 件を `Result::Ok` / `Result::Err` へ移行した。
- `tests/stdlib/neplg2_lifetime_proof.n.md` で、helper 戻り値 `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 32 件を `Result::Ok` / `Result::Err` へ移行した。
- `tests/stdlib/neplg2_owner_proof.n.md` で、helper 戻り値 `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 72 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 3 ファイルに残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はなく、source string fixture や doc comment だけの旧構文もないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_effect_proof.n.md tests/stdlib/neplg2_lifetime_proof.n.md tests/stdlib/neplg2_owner_proof.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_effect_proof.n.md -i tests/stdlib/neplg2_lifetime_proof.n.md -i tests/stdlib/neplg2_owner_proof.n.md --no-tree -o tmp/neplg21-proof-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 borrow proof Result constructor checkpoint

- `tests/stdlib/neplg2_borrow_proof.n.md` で、helper 戻り値 `Result unit str` と if/match branch expected type から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 42 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 file に残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はなく、source string fixture や doc comment だけの旧構文もないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_borrow_proof.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_borrow_proof.n.md --no-tree -o tmp/neplg21-borrow-proof-result-constructors.json -j 1 --dist web/dist --assert-io` は 1 件 compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 neplg2_proof Result constructor checkpoint

- `tests/stdlib/neplg2_proof.n.md` で、helper 戻り値 `Result unit str`、match branch expected type、または `checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 182 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 file に残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はなく、source string fixture や doc comment だけの旧構文もないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_proof.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/neplg21-proof-main-result-constructors.json -j 1 --dist web/dist --assert-io` は 304s local command timeout。partial JSON では 5/6 件が compile timeout after 60000ms、型診断は出ていない。
- timeout 後に残留していた `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 type proof Result constructor checkpoint

- `tests/stdlib/neplg2_type_proof.n.md` で、helper 戻り値 `Result unit str` と nested match branch expected type から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 51 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 file に残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はなく、source string fixture や doc comment だけの旧構文もないと確認した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_type_proof.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_proof.n.md --no-tree -o tmp/neplg21-type-proof-result-constructors.json -j 1 --dist web/dist --assert-io` は 1 件 compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 trait proof/type arena Result constructor checkpoint

- `tests/stdlib/neplg2_trait_proof.n.md` で、helper 戻り値 `Result unit str` と match branch expected type から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 43 件を `Result::Ok` / `Result::Err` へ移行した。
- `tests/stdlib/neplg2_type_arena.n.md` で、`checks_push` の expected type `Result unit str` から型が確定する `Result<unit,str>::Err` 30 件を `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 2 files に残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はなく、source string fixture や doc comment だけの旧構文もないと確認した。
- `Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err` など `Result unit str` ではない constructor は、この checkpoint では対象外として保持した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests/stdlib/neplg2_trait_proof.n.md tests/stdlib/neplg2_type_arena.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_trait_proof.n.md -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg21-trait-type-arena-result-constructors.json -j 1 --dist web/dist --assert-io` は 244s local command timeout。partial JSON では 4/6 件が compile timeout after 60000ms、型診断は出ていない。
- timeout 後に残留していた `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 remaining unit-str Result constructor checkpoint

- `tests/stdlib/neplg2_text.n.md` / `tests/stdlib/neplg2_lexer.n.md` / `tests/compiler/intrinsic.n.md` / `stdlib/std/streamio/scanner/state.nepl` / `stdlib/neplg2/core/resolve/name_resolver.nepl` / `stdlib/neplg2/core/hir/hir.nepl` で、関数戻り値、local annotation、`checks_push` expected type、または doc comment 内 doctest の expected type から型が確定する `Result<unit,str>::Ok` / `Result<unit,str>::Err` 54 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 files に残すべき `Result<unit,str>::Ok` / `Result<unit,str>::Err` はなく、source string fixture 内の旧構文もないと確認した。
- `Result<unit,i64>::Err` や `Result<unit,StdErrorKind>::Err` など `Result unit str` ではない constructor は、この checkpoint では対象外として保持した。
- `rg -n "Result<unit,str>::(Ok|Err)|Result<\\(\\),str>::(Ok|Err)" tests stdlib tutorials doc/examples` は 0 件になった。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_text.n.md -i tests/stdlib/neplg2_lexer.n.md -i tests/compiler/intrinsic.n.md -i stdlib/std/streamio/scanner/state.nepl -i stdlib/neplg2/core/resolve/name_resolver.nepl -i stdlib/neplg2/core/hir/hir.nepl --no-tree -o tmp/neplg21-remaining-unit-str-result-constructors.json -j 1 --dist web/dist --assert-io` は 304s local command timeout。partial JSON では 5/32 件が compile timeout after 60000ms、型診断は出ていない。
- timeout 後に残留していた `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 allocator/vec capacity Result constructor checkpoint

- `stdlib/core/mem/allocator.nepl` で、`alloc` / `realloc` の戻り値 `Result i32 str` から型が確定する `Result<i32,str>::Ok` / `Result<i32,str>::Err` 5 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/alloc/collections/vec/mutation/push.nepl` で、`vec_next_capacity` の戻り値 `Result i32 StdErrorKind` から型が確定する `Result<i32, StdErrorKind>::Ok` / `Result<i32, StdErrorKind>::Err` 5 件を `Result::Ok` / `Result::Err` へ移行した。
- `Result::Err<RegionToken<.T>, VecReallocRegionError<.T>>` のような constructor generic postfix は、この checkpoint では semantic rewrite 対象として保持した。
- `rg -n "Result<i32,str>::(Ok|Err)|Result<i32, StdErrorKind>::(Ok|Err)" stdlib/core/mem/allocator.nepl stdlib/alloc/collections/vec/mutation/push.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/core/mem/allocator.nepl -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree -o tmp/neplg21-allocator-vec-capacity-result-constructors.json -j 1 --dist web/dist --assert-io` は 304s local command timeout。partial JSON では 2 件 pass、4 件 compile timeout after 60000ms、failed 0。`compile_fail` 期待の `resource.raw.memory_outside_boundary` 以外に型診断は出ていない。
- timeout 後に残留していた `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 type arena generic Result constructor checkpoint

- `stdlib/neplg2/core/ty/ty/arena.nepl` で、関数戻り値または match/if branch expected type から型が確定する `Result<SelfhostTypeArena, StdErrorKind>::Ok` / `Result<SelfhostTypeArena, StdErrorKind>::Err` / `Result<SelfhostTypeArenaAlloc, StdErrorKind>::Ok` / `Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err` / `Result<Vec<SelfhostTypeId>, StdErrorKind>::Ok` / `Result<Vec<SelfhostTypeId>, StdErrorKind>::Err` 9 件を `Result::Ok` / `Result::Err` へ移行した。
- `tests/stdlib/neplg2_type_arena.n.md` で、fixture helper の戻り値 `Result SelfhostTypeArenaAlloc StdErrorKind` から型が確定する `Result<SelfhostTypeArenaAlloc, StdErrorKind>::Err` 5 件を `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 files に残すべき typed constructor はなく、source string fixture や doc comment だけの旧構文もないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/neplg2/core/ty/ty/arena.nepl tests/stdlib/neplg2_type_arena.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/arena.nepl -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg21-type-arena-generic-result-constructors.json -j 1 --dist web/dist --assert-io` は 5 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 streamio/intrinsic Result constructor checkpoint

- `stdlib/std/streamio/scanner/state.nepl` で、関数戻り値または match/if branch expected type から型が確定する `Result<i32,str>::Ok` / `Result<i32,str>::Err` / `Result<str,str>::Ok` / `Result<str,str>::Err` / `Result<Vec<i32>,str>::Ok` / `Result<Vec<i32>,str>::Err` / `Result<StreamScanner,str>::Ok` / `Result<StreamScanner,str>::Err` 12 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/std/streamio/scanner.nepl` で、`open` の戻り値 `Result StreamScanner str` から型が確定する `Result<StreamScanner,str>::Err` 2 件を `Result::Err` へ移行した。
- `tests/compiler/intrinsic.n.md` で、local annotation `%Result unit i64` から型が確定する `Result<unit,i64>::Err` 1 件を `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 files に残すべき typed constructor はなく、compile-fail fixture 内の intrinsic regression も local annotation で型確定すると確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" tests/compiler/intrinsic.n.md stdlib/std/streamio/scanner.nepl stdlib/std/streamio/scanner/state.nepl` は 0 件になった。
- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md -i stdlib/std/streamio/scanner.nepl -i stdlib/std/streamio/scanner/state.nepl --no-tree -o tmp/neplg21-streamio-intrinsic-result-constructors.json -j 1 --dist web/dist --assert-io` は 11 件中 10 件 pass、failed 0、`stdlib/std/streamio/scanner.nepl::doctest#1` のみ compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 text/string Result constructor checkpoint

- `stdlib/std/text/convert.nepl` / `stdlib/std/text/decode.nepl` / `stdlib/std/text/validate.nepl` で、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 39 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/alloc/string/char_offsets.nepl` / `stdlib/alloc/string/slice/byte.nepl` / `stdlib/alloc/string/slice/char.nepl` / `stdlib/alloc/string/utf8.nepl` で、同じく戻り値と branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 31 件を `Result::Ok` / `Result::Err` へ移行した。
- subagent の独立レビューでも、対象 files に残すべき typed constructor はなく、doc comment や source string fixture として保持すべき旧構文もないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/std/text/convert.nepl stdlib/std/text/decode.nepl stdlib/std/text/validate.nepl stdlib/alloc/string/char_offsets.nepl stdlib/alloc/string/slice/byte.nepl stdlib/alloc/string/slice/char.nepl stdlib/alloc/string/utf8.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/std/text/convert.nepl stdlib/std/text/decode.nepl stdlib/std/text/validate.nepl stdlib/alloc/string/char_offsets.nepl stdlib/alloc/string/slice/byte.nepl stdlib/alloc/string/slice/char.nepl stdlib/alloc/string/utf8.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/text/convert.nepl -i stdlib/std/text/decode.nepl -i stdlib/std/text/validate.nepl -i stdlib/alloc/string/char_offsets.nepl -i stdlib/alloc/string/slice/byte.nepl -i stdlib/alloc/string/slice/char.nepl -i stdlib/alloc/string/utf8.nepl --no-tree -o tmp/neplg21-text-string-result-constructors.json -j 1 --dist web/dist --assert-io` は 308s local command timeout。partial JSON では 5/6 件が compile timeout after 60000ms、failed 0。型診断は出ていない。
- timeout 後に残留していた当該 `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 sha256/hash Result constructor checkpoint

- subagent と並列で `stdlib/alloc/hash/sha256/api.nepl` / `compress.nepl` / `digest.nepl` / `padding.nepl` / `round.nepl` / `schedule.nepl` / `stdlib/tests/hash.n.md` を確認し、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 125 件を `Result::Ok` / `Result::Err` へ移行した。
- `sha256_k` の定数表は戻り値 `Result i32 StdErrorKind` から全 branch の型が確定するため、64 件の `Ok` と default branch の `Err` を postfix なし constructor へ移行した。
- source string fixture や doc comment として保持すべき旧構文はなく、subagent の独立レビューでも残すべき箇所はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/alloc/hash/sha256/api.nepl stdlib/alloc/hash/sha256/compress.nepl stdlib/alloc/hash/sha256/digest.nepl stdlib/alloc/hash/sha256/padding.nepl stdlib/alloc/hash/sha256/round.nepl stdlib/alloc/hash/sha256/schedule.nepl stdlib/tests/hash.n.md` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/alloc/hash/sha256/api.nepl stdlib/alloc/hash/sha256/compress.nepl stdlib/alloc/hash/sha256/digest.nepl stdlib/alloc/hash/sha256/padding.nepl stdlib/alloc/hash/sha256/round.nepl stdlib/alloc/hash/sha256/schedule.nepl stdlib/tests/hash.n.md` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/alloc/hash/sha256/api.nepl -i stdlib/alloc/hash/sha256/compress.nepl -i stdlib/alloc/hash/sha256/digest.nepl -i stdlib/alloc/hash/sha256/padding.nepl -i stdlib/alloc/hash/sha256/round.nepl -i stdlib/alloc/hash/sha256/schedule.nepl -i stdlib/tests/hash.n.md --no-tree -o tmp/neplg21-sha256-hash-result-constructors.json -j 1 --dist web/dist --assert-io` は 1 件 compile timeout after 60000ms。型診断は出ていない。
- subagent verification: `trunk build` / `node nodesrc/test_stdlib_hash_nmd_report_contract.js` / `node nodesrc/test_stdlib_hash_string_access_boundary.js` は pass。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 pointer/lexer Option constructor checkpoint

- `stdlib/core/mem/pointer/scalar.nepl` で、`load_i32` / `load_u8` の戻り値 `Option i32` から型が確定する `Option<i32>::None` / `Option<i32>::Some` 4 件を `Option::None` / `Option::Some` へ移行した。
- `stdlib/neplg2/core/syntax/lexer/keyword.nepl` で、keyword classifier の戻り値 `Option TokenKind` から型が確定する `Option<TokenKind>::Some` / `Option<TokenKind>::None` 9 件を `Option::Some` / `Option::None` へ移行した。
- `rg -n "Option<[^>]+>::(Some|None)" stdlib/core/mem/pointer/scalar.nepl stdlib/neplg2/core/syntax/lexer/keyword.nepl` は 0 件になった。
- `rg --pcre2 -n "Option::(?!Some|None)" stdlib/core/mem/pointer/scalar.nepl stdlib/neplg2/core/syntax/lexer/keyword.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/core/mem/pointer/scalar.nepl -i stdlib/neplg2/core/syntax/lexer/keyword.nepl -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg21-pointer-lexer-option-constructors.json -j 1 --dist web/dist --assert-io` は 308s local command timeout。partial JSON では `stdlib/core/mem/pointer/scalar.nepl::doctest#1` は pass、`tests/stdlib/neplg2_lexer.n.md` 4 件は compile timeout after 60000ms。型診断は出ていない。
- timeout 後に残留していた当該 `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 streamio writer Result constructor checkpoint

- subagent と並列で `stdlib/std/streamio/writer.nepl` / `stdlib/std/streamio/writer/state.nepl` を確認し、`open` と `stream_writer_new` の戻り値 `Result StreamWriter str` から型が確定する `Result<StreamWriter,str>::Ok` / `Result<StreamWriter,str>::Err` 7 件を `Result::Ok` / `Result::Err` へ移行した。
- source string fixture や doc comment として保持すべき旧構文はなく、subagent の独立レビューでも残すべき箇所はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/std/streamio/writer.nepl stdlib/std/streamio/writer/state.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/std/streamio/writer.nepl stdlib/std/streamio/writer/state.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/streamio/writer.nepl -i stdlib/std/streamio/writer/state.nepl --no-tree -o tmp/neplg21-streamio-writer-result-constructors.json -j 1 --dist web/dist --assert-io` は 1 件 compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 fs fd/stat Result constructor checkpoint

- `stdlib/std/fs/fd.nepl` で、`fs_open_with_flags` / `fs_close` の戻り値と local `res` annotation から型が確定する `Result<i32,i32>::Ok` / `Result<i32,i32>::Err` / `Result<unit,i32>::Ok` / `Result<unit,i32>::Err` 6 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/std/fs/stat.nepl` で、`fs_path_filetype` の戻り値と local `res` annotation から型が確定する `Result<i32,i32>::Ok` / `Result<i32,i32>::Err` 4 件を `Result::Ok` / `Result::Err` へ移行した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/std/fs/fd.nepl stdlib/std/fs/stat.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/std/fs/fd.nepl stdlib/std/fs/stat.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/fs/fd.nepl -i stdlib/std/fs/stat.nepl --no-tree -o tmp/neplg21-fs-fd-stat-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 std fs Result constructor checkpoint

- `stdlib/std/fs/bytes.nepl` / `dir/open.nepl` / `dir/read_fd.nepl` / `path/entry.nepl` / `path/normalize.nepl` / `path/normalize/build.nepl` / `read/fd.nepl` / `write/fd.nepl` / `write/path.nepl` で、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 52 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/std/fs/fd.nepl` / `stdlib/std/fs/stat.nepl` は直前 checkpoint で移行済みのため、この checkpoint では追加差分なし。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/std/fs` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/std/fs` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/fs/bytes.nepl -i stdlib/std/fs/dir/open.nepl -i stdlib/std/fs/dir/read_fd.nepl -i stdlib/std/fs/path/entry.nepl -i stdlib/std/fs/path/normalize.nepl -i stdlib/std/fs/path/normalize/build.nepl -i stdlib/std/fs/read/fd.nepl -i stdlib/std/fs/write/fd.nepl -i stdlib/std/fs/write/path.nepl --no-tree -o tmp/neplg21-std-fs-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 alloc/io Result constructor checkpoint

- subagent と並列で `stdlib/alloc/io/bytebuf.nepl` / `stdlib/alloc/io/bytebuilder/storage.nepl` / `stdlib/alloc/io/bytebuilder/build.nepl` / `stdlib/alloc/io/bytebuilder/append.nepl` を確認し、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 64 件を `Result::Ok` / `Result::Err` へ移行した。
- source string fixture や doc comment として保持すべき旧構文はなく、subagent の独立レビューでも残すべき箇所はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/alloc/io/bytebuf.nepl stdlib/alloc/io/bytebuilder/storage.nepl stdlib/alloc/io/bytebuilder/build.nepl stdlib/alloc/io/bytebuilder/append.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/alloc/io/bytebuf.nepl stdlib/alloc/io/bytebuilder/storage.nepl stdlib/alloc/io/bytebuilder/build.nepl stdlib/alloc/io/bytebuilder/append.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuf.nepl -i stdlib/alloc/io/bytebuilder/storage.nepl -i stdlib/alloc/io/bytebuilder/build.nepl -i stdlib/alloc/io/bytebuilder/append.nepl --no-tree -o tmp/neplg21-alloc-io-result-constructors.json -j 1 --dist web/dist --assert-io` は 308s local command timeout。partial JSON では 5/7 件が compile timeout after 60000ms、failed 0。型診断は出ていない。
- timeout 後に残留していた当該 `node.exe` process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 stdio read Result constructor checkpoint

- `stdlib/std/stdio/read/buffer.nepl` / `stdlib/std/stdio/read/text.nepl` で、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 15 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/std/stdio/write/fd.nepl` は subagent が並列で確認中のため、この checkpoint では追加差分に含めない。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/std/stdio/read/buffer.nepl stdlib/std/stdio/read/text.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/std/stdio/read/buffer.nepl stdlib/std/stdio/read/text.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/stdio/read/buffer.nepl -i stdlib/std/stdio/read/text.nepl --no-tree -o tmp/neplg21-stdio-read-result-constructors.json -j 1 --dist web/dist --assert-io` は 5 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 stdio write Result constructor checkpoint

- subagent と並列で `stdlib/std/stdio/write/fd.nepl` を確認し、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 20 件を `Result::Ok` / `Result::Err` へ移行した。
- source string fixture や doc comment として保持すべき旧構文はなく、subagent の独立レビューでも残すべき箇所はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/std/stdio/write/fd.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/std/stdio/write/fd.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/std/stdio/write/fd.nepl --no-tree -o tmp/neplg21-stdio-write-result-constructors.json -j 1 --dist web/dist --assert-io` は 3 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 move/memory fixture Result constructor checkpoint

- `tests/compiler/move_effect.n.md` / `tests/compiler/move_check.n.md` / `tests/stdlib/memory_safety.n.md` で、local annotation または helper 戻り値から型が確定する `Result<...>::Ok` / `Result<...>::Err` 8 件を `Result::Ok` / `Result::Err` へ移行した。
- 対象箇所は実行される NEPL source block であり、旧構文そのものを期待する negative fixture ではないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" tests/compiler/move_effect.n.md tests/compiler/move_check.n.md tests/stdlib/memory_safety.n.md` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" tests/compiler/move_effect.n.md tests/compiler/move_check.n.md tests/stdlib/memory_safety.n.md` は 0 件になった。
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md -i tests/compiler/move_check.n.md -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/neplg21-move-memory-fixture-result-constructors.json -j 1 --dist web/dist --assert-io` は 308s local command timeout。partial JSON では 10/230 件完了、10 件 pass、failed 0、errored 0、top_issues 0。型診断は出ていない。timeout 後の残留 process は停止した。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass。

### 2026-05-26 selfhost module graph Result constructor checkpoint

- subagent と並列で `stdlib/neplg2/core/module/graph.nepl` / `stdlib/neplg2/core/module/stdlib_map.nepl` を確認し、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 49 件を `Result::Ok` / `Result::Err` へ移行した。
- source string fixture として旧構文を保持すべき箇所はなく、subagent の独立レビューでも残すべき typed constructor はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)" stdlib/neplg2/core/module/graph.nepl stdlib/neplg2/core/module/stdlib_map.nepl` は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)" stdlib/neplg2/core/module/graph.nepl stdlib/neplg2/core/module/stdlib_map.nepl` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/neplg21-selfhost-module-graph-result-constructors.json -j 1 --dist web/dist --assert-io` は 2 件とも compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `git diff --check` は pass。

### 2026-05-26 selfhost module/parser Result constructor checkpoint

- subagent と並列で `stdlib/neplg2/core/module/import_scan.nepl` / `stdlib/neplg2/core/module/import_spec.nepl` / `stdlib/neplg2/core/module/loader.nepl` / `stdlib/neplg2/core/module/vfs.nepl` / `stdlib/neplg2/core/mono/mono.nepl` / `stdlib/neplg2/core/pipeline.nepl` / `stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl` / `stdlib/neplg2/core/syntax/parser/module_parser/diagnostic.nepl` / `stdlib/neplg2/core/syntax/parser/module_parser/entry.nepl` / `stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl` を確認し、関数戻り値、local annotation、match/if branch expected type から型が確定する `Result<...>::Ok` / `Result<...>::Err` 40 件を `Result::Ok` / `Result::Err` へ移行した。
- source string fixture として旧構文を保持すべき箇所はなく、subagent の独立レビューでも残すべき typed constructor はないと確認した。
- `rg -n "Result<[^>]+>::(Ok|Err)"` 対象 10 files は 0 件になった。
- `rg --pcre2 -n "Result::(?!Ok|Err)"` 対象 10 files は 0 件になった。
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_scan.nepl -i stdlib/neplg2/core/module/import_spec.nepl -i stdlib/neplg2/core/module/loader.nepl -i stdlib/neplg2/core/module/vfs.nepl -i stdlib/neplg2/core/mono/mono.nepl -i stdlib/neplg2/core/pipeline.nepl -i stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl -i stdlib/neplg2/core/syntax/parser/module_parser/diagnostic.nepl -i stdlib/neplg2/core/syntax/parser/module_parser/entry.nepl -i stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl --no-tree -o tmp/neplg21-selfhost-module-parser-result-constructors.json -j 1 --dist web/dist --assert-io` は 6 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `git diff --check` は pass。

### 2026-05-26 five-worker Result constructor checkpoint

- typed constructor 密度の高い実コードを 5 worker の非重複 write scope に分割し、proof API / std IO-env-streamio output / HIR-check-module AST / CLI-platform-infra / collections-json-string-resolve-lexer の 5 領域を並列移行した。
- proof API 5 files で 132 件、std IO/env/streamio output 5 files で 58 件、HIR/check/module AST 5 files で 43 件、CLI/platform/infra 6 files で 42 件、collections/json/string/resolve/lexer 5 files で 28 件を移行し、合計 26 files / 303 件を `Result::Ok` / `Result::Err` へ移行した。
- `Result<[^>]+>` 形式の確認ではネスト型を拾えないため、親 agent 側で `>::(Ok|Err|Some|None)` による再確認を行い、`Result<Vec<...>>` / `Result<SelfhostOutcome<...>>` などの取りこぼし 13 件を同じ checkpoint 内で修正した。
- subagent が一時的に `cleanup/neplg21-constructor-write-scope` branch に切り替えていたが、commit 前に `feature/neplg21-syntax-migration-20260524` へ戻し、差分を元の作業 branch 上へ集約した。
- `rg -n ">::(Ok|Err|Some|None)"` 対象 26 source files は 0 件になった。
- `rg --pcre2 -n "(?<![A-Za-z0-9_])(Result|Option)::(?!(Ok|Err|Some|None)\\b)"` 対象 26 source files は 0 件になった。
- `node nodesrc/tests.js <対象 26 files> --no-tree -o tmp/neplg21-five-worker-constructor-cleanup.json -j 4 --dist web/dist --assert-io` は 14 件すべて compile timeout after 60000ms。JSON 上は timeout のみで、型診断は出ていない。
- `node nodesrc/neplg21_syntax_migrate.js --check` / `git diff --check` は pass。

### 2026-05-26 final source Result constructor checkpoint

- 実コード側に残っていた typed constructor を 5 worker の非重複 write scope に分割し、lexer tokenize / raw memory-bytebuf-fs raw / hash collections / btree-fs small modules / selfhost import-move fixture の 5 領域を並列移行した。
- lexer tokenize 17 件、raw memory / byte buffer / fs raw 15 件、hash collections 18 件、btree / fs small modules 20 件、selfhost import / move fixture 15 件を移行し、合計 18 files / 85 件を `Result::Ok` / `Result::Err` へ移行した。
- `stdlib/alloc/io/traits.nepl:35` に残る `Result<CountSink, StdErrorKind>::Ok` は `//:` comment 内のサンプルであり、実行 source ではないためこの checkpoint では保持した。
- `rg -n ">::(Ok|Err|Some|None)" stdlib tests examples -g "*.nepl" -g "*.n.md"` は、実コード 0 件、`stdlib/alloc/io/traits.nepl:35` の comment sample 1 件のみになった。
- `rg --pcre2 -n "(?<![A-Za-z0-9_])(Result|Option)::(?!(Ok|Err|Some|None)\\b)" stdlib tests examples -g "*.nepl" -g "*.n.md"` は 0 件になった。
- `node nodesrc/tests.js <対象 18 files> --no-tree -o tmp/neplg21-final-constructor-cleanup.json -j 4 --dist web/dist --assert-io` は外側 904s timeout。partial JSON は 90/151 件完了、64 pass、25 compile timeout、1 fail。1 fail は今回変更していない `tests/compiler/move_effect.n.md::doctest#26` の既存診断 mismatch で、期待 `resource.raw.memory_outside_boundary` に対し実際は `resource.cell.uninit`。今回差分の6行には含まれない。

### 2026-05-26 five-worker helper postfix checkpoint

- `some` / `none` / `ok` / `err` / `is_some` / `is_none` / `is_ok` / `is_err` の helper postfix を 5 worker の非重複 write scope に分割し、examples/KP、type/prelude、selfhost parser、CLI args、collections/diag の 5 領域を並列移行した。
- 対象26 files から helper postfix 398 件を撤廃した。`new<T>` / `push<T>` / `get<T>` / `unwrap_ok<T,E>` など、producer 側または別 helper family の推論を混ぜる箇所はこの checkpoint では残した。
- doctest comment も実行対象なので、`stdlib/neplg2/core/options.nepl` と `stdlib/alloc/diag/error/outcome.nepl` に残っていた旧 helper postfix sample を同じ checkpoint で更新した。
- `options.nepl` doctest は `some` と `and` を直接使うため、`#import "core/option" as *` と `#import "core/math" as *` を明示した。
- `rg -n "\b(some|none|ok|err|is_some|is_none|is_ok|is_err)<" <対象26 files>` は 0 件になった。
- `node nodesrc/tests.js -i stdlib/neplg2/core/options.nepl --no-tree -o tmp/neplg21-options-doctest-after-import.json -j 1 --dist web/dist --assert-io` は 1/1 pass。
- `node nodesrc/tests.js <対象26 files> --no-tree -o tmp/neplg21-helper-postfix-scope-cleanup.json -j 4 --dist web/dist --assert-io` は 33 件中 32 件 compile timeout、1 件 fail。fail は `options.nepl` doctest の import 不足で、上記 focused test により修正済み。

### 2026-05-26 collection/selfhost helper postfix parallel checkpoint

- collection constructor/helper cleanup を 5 worker の非重複 write scope に分割し、BinaryHeap/Deque/Queue/RingBuffer、Stack/List/Vec、BitSet/AdjacencyMatrix/BloomFilter/CountingBloomFilter、Fenwick/SegmentTree/SparseSet/DisjointSet、BTreeMap/BTreeSet/HashMap/HashSet を同時に処理した。
- worker 合計で collection 60 files / 210 helper postfix occurrences を撤廃した。親 agent 側では selfhost core / env 14 files / 38 occurrences を並行して移行し、合計 74 source files / 248 occurrences の `some` / `none` / `ok` / `err` / `is_*` postfix-free 化を行った。
- `rg -n "\b(some|none|ok|err|is_some|is_none|is_ok|is_err)<" stdlib/alloc/collections stdlib/neplg2/core stdlib/std/env -g "*.nepl"` は 0 件になった。
- 変更範囲に直結する source policy 18 件と `nodesrc/test_stdlib_collection_cleanup_contract.js` / `nodesrc/test_selfhost_def_id_absence.js` は pass。collection cleanup policy は constructor postfix-free と `unit` keyword view に追従し、owner-boundary assertion は弱めていない。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は完走したが、NEPLg2.1 unit / constructor cleanup に未追従の residual policy warning が 29 件残ったため、`ISS-20260526T073859722Z-NEPLG2-1-SOURCE-POLICY-UNIT-AND-CONS-F81D1534` として分離した。
- `node nodesrc/neplg21_syntax_migrate.js --check` は pass。

### 2026-05-26 diag_err / Stack accessor postfix checkpoint

- `diag_err<T>` の撤廃を 5 worker の非重複 write scope に分割し、`stdlib/kp`、collection storage、Fenwick/BitSet/DisjointSet/AdjacencyMatrix、SparseSet/SegmentTree/BloomFilter/CountingBloomFilter、Deque/BinaryHeap/Stack/Queue/RingBuffer/List を並列移行した。
- 合計 76 件の `diag_err<...>` を `diag_err` へ移行した。対象は関数戻り値または `%Result ... Diag` local/block から `T` が確定する箇所に限定し、`new<T>` / `push<T>` / `unwrap_ok<T,E>` などの producer 系 generic postfix はこの checkpoint では残した。
- `examples/rpn.nepl` / `examples/rpn_legacy.nepl` / `examples/bf.nepl` と Stack API doctest / stack collection fixture では、`Stack i32` / `Vec i32` の引数型やlocal annotationから確定する accessor/observer/free だけを postfix-free へ移行した。
- source policy は旧 `diag_err<Vec<i32>>` / `stack_pop_item<i32>` / `stack_pop_stack<i32>` を期待していた2検査を NEPLg2.1 postfix-free 期待へ更新した。owner recovery、borrowed observer、cleanup 境界の assertion は弱めていない。
- `rg -n "\bdiag_err<" stdlib tests examples -g "*.nepl" -g "*.n.md"` は 0 件になった。
- `rg -n "stk::(stack_push_error_stack|stack_pop_item|stack_pop_stack|len|get|free|is_empty)<|v::(free|replace)<i32>|stack_(pop_item|pop_stack)<i32>"` 対象 files は 0 件になった。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js -i examples/rpn.nepl -i examples/rpn_legacy.nepl -i examples/bf.nepl --no-tree -o tmp/neplg21-examples-stack-accessor-postfix.json -j 1 --dist web/dist --assert-io` は 5 件すべて compile timeout after 60000ms。型診断は出ていない。
- `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/neplg21-stack-accessor-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 6/18 件完了、6 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 Stack unwrap_ok postfix checkpoint

- Stack fixture の `unwrap_ok<Stack<i32>, ...>` を 2 worker の非重複 write scope に分割し、`stdlib/tests/stack.n.md` と `tests/stdlib/stack_collections.n.md` を並列移行した。
- `stdlib/tests/stack.n.md` で 33 件、`tests/stdlib/stack_collections.n.md` で 41 件、合計 74 件を `unwrap_ok` へ移行した。
- この checkpoint では `new<i32>` / `push<i32>` / `pop<i32>` / `get<i32>` / `len<i32>` などの producer / observer postfix は触っていない。
- Stack source policy は旧 `unwrap_ok<Stack<i32>...>` の再導入を禁止し、NEPLg2.1 の型根拠ベースの Stack fixture であることを確認する形へ更新した。
- `rg -n "unwrap_ok<Stack<i32>" stdlib/tests/stack.n.md tests/stdlib/stack_collections.n.md` は 0 件になった。
- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/neplg21-stack-unwrap-ok-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 5/18 件完了、5 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 collection unwrap_ok postfix checkpoint

- Queue / RingBuffer / List / BinaryHeap / Deque / pipe collection fixture の `unwrap_ok<T,E>` を 5 worker の非重複 write scope に分割して並列移行した。
- Queue 23 件、RingBuffer 25 件、List 16 件、BinaryHeap/Deque 39 件、pipe collection fixture 19 件を worker が移行し、親 agent 側で Stack/List/Vec の小さな overload / compiler fixture 残件 9 件を移行した。
- 合計 24 files / 131 件の `unwrap_ok<...>` を `unwrap_ok` へ移行した。
- この checkpoint では `new<i32>` / `with_capacity<i32>` / `push<i32>` / `push_back<i32>` / `push_front<i32>` / `len<i32>` / `free<i32>` などの producer / observer postfix は触っていない。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` は、今回移行した collection fixture / doctest が旧 `unwrap_ok<...>` を再導入しないことを検査する形へ更新した。既存の owner recovery / cleanup / borrowed observer boundary の検査は弱めていない。
- `rg -n "unwrap_ok<" <対象24 files>` は 0 件になった。
- `node nodesrc/test_stdlib_collection_cleanup_contract.js` と `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` は pass した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js -i stdlib/tests/queue.n.md -i tests/stdlib/queue_collections.n.md -i stdlib/tests/ringbuffer.n.md -i tests/stdlib/ringbuffer_collections.n.md -i stdlib/tests/list.n.md -i tests/stdlib/list_collections.n.md -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md -i stdlib/tests/deque.n.md -i tests/stdlib/deque_collections.n.md -i tests/stdlib/pipe_collections.n.md -i tests/compiler/overload.n.md -i tests/compiler/neplg2.n.md --no-tree -o tmp/neplg21-collection-unwrap-ok-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15/123 件完了、15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 BTree/Hash/Bloom unwrap_ok postfix checkpoint

- BTreeMap / BTreeSet / HashMap / HashSet / BloomFilter / CountingBloomFilter の `unwrap_ok<T,E>` を 5 worker の非重複 write scope に分割して並列移行した。
- BTree API doctest 15 件、BTree test fixture 19 件、Hash API doctest 34 件、Hash rehash fixture 16 件、Bloom / CountingBloomFilter doctest・fixture 25 件を移行し、合計 25 files / 109 件の `unwrap_ok<...>` を `unwrap_ok` へ移行した。
- 対象は `let` の型注釈、`set` 先の型、または pipe の直前結果から型根拠が十分な箇所に限定した。
- この checkpoint では `new` / `with_capacity` / `insert` / `remove` / `contains` / `free` などの producer / observer postfix は触っていない。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` の NEPLg2.1 `unwrap_ok` postfix 再導入防止リストへ BTree / Hash / Bloom 対象ファイルを追加した。既存の owner recovery / cleanup / borrowed observer boundary の検査は弱めていない。
- `rg -n "unwrap_ok<(BTreeMap|BTreeSet|HashMap|HashSet|BloomFilter|CountingBloomFilter)<" <対象files>` は 0 件になった。
- `node nodesrc/test_stdlib_collection_cleanup_contract.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stdlib/btree_array_cost.n.md -i tests/stdlib/hash_collection_rehash.n.md -i stdlib/tests/bloom_filter.n.md -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md --no-tree -o tmp/neplg21-btree-hash-bloom-unwrap-ok-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15/31 件完了、15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 KP/Vec/SegmentTree unwrap_ok postfix checkpoint

- KP doctest、SegmentTree API doctest、Vec mutation/query/sort doctest、collection cleanup fixture の `unwrap_ok<T,E>` を 5 worker の非重複 write scope に分割して並列移行した。
- KP 11 件、SegmentTree 9 件、Vec mutation/query/storage 18 件、Vec sort 10 件、collection cleanup fixture 1 件を移行し、合計 19 source files / 49 件の `unwrap_ok<...>` を `unwrap_ok` へ移行した。
- 対象は `let` の型注釈、呼び出し先引数の期待型、または pipe の直前結果から型根拠が十分な箇所に限定した。
- この checkpoint では `new<T>` / `push<T>` / `replace_drop_old<T>` / `sort_merge_ret<T>` / `sum_range` / `free<T>` などの producer / observer postfix は触っていない。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` の NEPLg2.1 `unwrap_ok` postfix 再導入防止リストへ KP / SegmentTree / Vec 対象ファイルを追加した。既存の owner recovery / cleanup / borrowed observer boundary の検査は弱めていない。
- `rg -n "unwrap_ok<" stdlib/kp stdlib/alloc/collections/segment_tree stdlib/alloc/collections/vec tests/stdlib/collection_cleanup_contract.n.md tests/stdlib/sort.n.md` は 0 件になった。
- `node nodesrc/test_stdlib_collection_cleanup_contract.js` と `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js -i stdlib/kp/kpprefix.nepl -i stdlib/kp/kpgraph.nepl -i stdlib/kp/kpdsu.nepl -i stdlib/alloc/collections/segment_tree/api/cleanup.nepl -i stdlib/alloc/collections/segment_tree/api/create.nepl -i stdlib/alloc/collections/segment_tree/api/query.nepl -i stdlib/alloc/collections/segment_tree/api/update.nepl -i stdlib/alloc/collections/vec/mutation/push.nepl -i stdlib/alloc/collections/vec/mutation/pop.nepl -i stdlib/alloc/collections/vec/mutation/replace.nepl -i stdlib/alloc/collections/vec/query/predicate.nepl -i stdlib/alloc/collections/vec/query/aggregate.nepl -i stdlib/alloc/collections/vec/storage/api.nepl -i stdlib/alloc/collections/vec/sort.nepl -i stdlib/alloc/collections/vec/sort/merge.nepl -i stdlib/alloc/collections/vec/sort/merge/api.nepl -i tests/stdlib/collection_cleanup_contract.n.md -i tests/stdlib/sort.n.md --no-tree -o tmp/neplg21-kp-vec-segtree-unwrap-ok-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15/121 件完了、15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 lower-case helper postfix final sweep checkpoint

- `some` / `none` / `ok` / `err` / `is_some` / `is_none` / `is_ok` / `is_err` と、`unwrap_or` / `unwrap_err` / `uwok` / `uwerr` / `diag_err` の lower-case helper postfix 残件を、5 worker の非重複 write scope と親 agent の `stdlib/tests` 範囲に分割して並列移行した。
- 対象は `tests/compiler` の small fixture、selfhost NMD、core Option/Result doctest、bytebuf / bytebuilder / CLI args、`stdlib/tests` の List / HashMap / JSON / BloomFilter smoke である。
- `tests/compiler/list_dot_map.n.md` の `result::ok<i32, i32>` / `ok<i32, i32>` は、`let r %Result i32 i32 ...` にして型根拠を明示したうえで postfix を撤廃した。
- `nodesrc/test_neplg21_helper_postfix_cleanup.js` を追加し、`stdlib` / `tests` / `examples` / `tutorials` の実行対象 source に lower-case helper postfix が再導入されないことを source policy で監視するようにした。
- `rg -n "\b(uwok|uwerr|unwrap_err|unwrap_or|diag_err|some|none|ok|err|is_some|is_none|is_ok|is_err)<" stdlib tests examples tutorials -g "*.nepl" -g "*.n.md"` は 0 件になった。
- `Result::Ok<...>` / `Result::Err<...>` / `Option::Some<...>` / `Option::None<...>` など enum constructor 側の旧 postfix はまだ残る。これは lower-case helper family とは別 checkpoint として継続する。
- `node nodesrc/test_neplg21_helper_postfix_cleanup.js` / `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `trunk build` は pass した。
- `node nodesrc/tests.js <対象11 NMD files> --no-tree -o tmp/neplg21-helper-postfix-cleanup.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 8 件中 1 pass、7 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 enum constructor postfix checkpoint

- `Result::Ok<...>` / `Result::Err<...>` / `Option::Some<...>` / `Option::None<...>` の enum constructor postfix を 5 worker の非重複 write scope に分割して並列移行した。
- collection owner API、Vec storage/mutation、Vec transform、string integer/float、core deserialize、KP search、compiler fixture を対象にし、合計 26 files / 153 件を postfix-free 化した。
- payload constructor や producer/helper generic postfix は意図的に残した。例えば `Vec<.T>` / `VecTransformError<.T>` / `view::vec_empty<.T>` / `parse_err_to_std<bool>` は今回の enum constructor syntax migration ではない。
- stale source policy は、owner-preserving failure payload や capacity validation の検査を弱めず、旧 `Result::Err<T,E>` / `Result::Ok<T,E>` と NEPLg2.1 `Result::Err` / `Result::Ok` の両方を契約上同じ constructor boundary として扱う形へ追従した。
- `nodesrc/test_neplg21_helper_postfix_cleanup.js` は lower-case helper に加えて enum constructor postfix の再導入も禁止するようにした。
- `rg -n "\b(?:Option::Some|Option::None|Result::Ok|Result::Err)<" stdlib tests examples tutorials -g "*.nepl" -g "*.n.md"` は 0 件になった。
- `node nodesrc/run_source_policy_regressions.js --warn-only` / `node nodesrc/neplg21_syntax_migrate.js --check` / `node nodesrc/issues.js check --dir issues` / `git diff --check` は pass した。
- `trunk build` は pass した。
- `node nodesrc/tests.js <対象26 files> --no-tree -o tmp/neplg21-enum-constructor-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 8 件中 8 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 collection observer / selfhost Vec accessor postfix checkpoint

- collection fixture の observer / cleanup / accessor 系 postfix を、Queue/RingBuffer、List、BinaryHeap/Deque、BTreeMap/BTreeSet、Vec/Sort の 5 worker 非重複 write scope に分割して並列移行した。
- 親 agent 側では worker commit と衝突しないよう再確認しながら、selfhost / std fs 内の `v::len<T>` / `v::get<T>` / `v::free<T>` / `v::vec_push_error_vec<T>` を receiver 型または local annotation から解ける postfix-free 形へ移行した。
- 合計 42 files で、`len` / `get` / `contains` / `free` / `peek` / `pop` / `pop_front` / `pop_back` / `pop_max` / collection pop accessor / `head` / `tail` / `list_transform_error_list` / `vec_partition_*` / in-place `sort_*` / `v::len` / `v::get` / `v::free` / `v::vec_push_error_vec` の後置 generic を撤廃した。
- 対象は receiver 型、local annotation、戻り値 annotation、または wrapper result の型から型根拠が明確な箇所に限定した。
- `new<T>` / `push<T>` / `with_capacity<T>` / `insert<T>` / `remove<T>` / `map<T,U>` / `filter<T>` / `partition<T>` / `take_while<T>` / `drop_while<T>` / `sort_*_ret<T>` / `reverse<T>` / `cons<T>` は、producer / mutator / transform result 系として今回の checkpoint では残した。
- subagent 報告と実体に差があった BTreeMap/BTreeSet については、親 agent 側の `rg` で残件を検出し、同じ checkpoint 内で再適用した。
- `tests/stdlib/pipe_collections.n.md` は source policy 追従中に stale な observer / cleanup / accessor postfix を検出したため、同じ checkpoint に含めた。producer / mutator / owner recovery 系の postfix はこの段階では残した。
- `rg -n "\b(len|get|contains|free|peek|pop|pop_front|pop_back|pop_max|queue_pop_item|queue_pop_queue|ringbuffer_pop_item|ringbuffer_pop_buffer|binary_heap_pop_item|binary_heap_pop_heap|deque_pop_item|deque_pop_deque|head|tail|list_transform_error_list|vec_partition_matched_len|vec_partition_rest_len|vec_partition_matched_get|vec_partition_rest_get|vec_partition_free|sort_quick|sort_heap|sort_merge|sort_is_sorted)<" <collection対象files>` は 0 件になった。
- `rg -n "\bv::(len|get|free|vec_push_error_vec)<" stdlib/neplg2 stdlib/std/fs/path stdlib/std/fs/dir -g "*.nepl"` は 0 件になった。
- `node nodesrc/neplg21_syntax_migrate.js --check` は `would update 0 file(s)` になった。
- `node nodesrc/test_neplg21_helper_postfix_cleanup.js` は pass した。
- `node nodesrc/issues.js check --dir issues` は pass した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `trunk build` は pass した。
- `git diff --check` は pass した。LF/CRLF warning のみ。
- `node nodesrc/tests.js <対象35 files> --no-tree -o tmp/neplg21-collection-observer-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15 件中 15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 Stack observer / pop result postfix checkpoint

- `stdlib/tests/stack.n.md`、`tests/stdlib/stack_collections.n.md`、`examples/rpn.nepl`、`examples/rpn_legacy.nepl`、`examples/bf.nepl`、`tests/compiler/overload.n.md` を 4 worker の非重複 write scope に分割して並列移行した。
- Stack fixture では `len<i32>` / `get<i32>` / `free<i32>` / `peek<i32>` / `pop<i32>` / `pop_top<i32>` / `clear<i32>` を postfix-free にした。
- examples では `%StackPop i32` local annotation と Stack owner 引数から型が確定する `stk::pop_top<i32>` を `stk::pop_top` へ移行した。
- overload fixture では `Stack i32` / `Vec i32` receiver から型が確定する `len<i32>` / `free<i32>` / `v::len<i32>` / `v::free<i32>` を postfix-free にした。
- `new<i32>` / `push<i32>` / `v::new<i32>` / `v::push<i32>` / `pair_with_empty<i32>` / `Show::show<i32>` は producer / generic function under test / trait method generic として今回の checkpoint では残した。
- `nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` に Stack fixture、examples、overload fixture の postfix-free 契約を追加した。owner recovery、borrowed observer、cleanup 境界の検査は弱めていない。
- `rg -n "\b(?:len|get|free|peek|pop|pop_top|clear)<" stdlib/tests/stack.n.md tests/stdlib/stack_collections.n.md` は 0 件になった。
- `rg -n "\bstk::pop_top<|\b(?:v::)?(?:len|free)<" examples/rpn.nepl examples/rpn_legacy.nepl examples/bf.nepl tests/compiler/overload.n.md` は 0 件になった。
- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/test_neplg21_helper_postfix_cleanup.js`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `trunk build` と `git diff --check` は pass した。`git diff --check` は LF/CRLF warning のみ。
- `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md -i examples/rpn.nepl -i examples/rpn_legacy.nepl -i examples/bf.nepl -i tests/compiler/overload.n.md --no-tree -o tmp/neplg21-stack-observer-pop-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15/68 件完了、15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 Stack/Queue/RingBuffer producer postfix checkpoint

- `stdlib/tests/stack.n.md` / `tests/stdlib/stack_collections.n.md`、`stdlib/tests/queue.n.md` / `tests/stdlib/queue_collections.n.md`、`stdlib/tests/ringbuffer.n.md` / `tests/stdlib/ringbuffer_collections.n.md` を 3 worker の非重複 write scope に分割して並列移行した。
- Stack fixture では `new<i32>` / `push<i32>` を postfix-free にした。
- Queue fixture では `new<i32>` / `with_capacity<i32>` / `push<i32>` を postfix-free にした。
- RingBuffer fixture では `new<i32>` / `with_capacity<i32>` / `push<i32>` を postfix-free にした。
- 対象は `%Stack i32` / `%Queue i32` / `%RingBuffer i32` local annotation、または receiver 型から型根拠が明確な箇所に限定した。
- `nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_ringbuffer_borrowed_observers.js` に、今回の対象 fixture が explicit producer / mutator postfix へ戻らないことを固定した。owner recovery、borrowed observer、cleanup 境界の検査は弱めていない。
- `rg -n "\b(new|push)<" stdlib/tests/stack.n.md tests/stdlib/stack_collections.n.md` は 0 件になった。
- `rg -n "\b(new|with_capacity|push)<" stdlib/tests/queue.n.md tests/stdlib/queue_collections.n.md stdlib/tests/ringbuffer.n.md tests/stdlib/ringbuffer_collections.n.md` は 0 件になった。
- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_ringbuffer_borrowed_observers.js`、`node nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js` は pass した。
- `node nodesrc/neplg21_syntax_migrate.js --check` は `would update 0 file(s)` になった。
- `node nodesrc/run_source_policy_regressions.js --warn-only`、`node nodesrc/issues.js check --dir issues`、`trunk build`、`git diff --check` は pass した。`git diff --check` は LF/CRLF warning のみ。
- `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md -i stdlib/tests/queue.n.md -i tests/stdlib/queue_collections.n.md -i stdlib/tests/ringbuffer.n.md -i tests/stdlib/ringbuffer_collections.n.md --no-tree -o tmp/neplg21-stack-queue-ringbuffer-producer-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15/26 件完了、15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 List/BinaryHeap/Deque producer postfix checkpoint

- `stdlib/tests/list.n.md` / `tests/stdlib/list_collections.n.md`、`stdlib/tests/binary_heap.n.md` / `tests/stdlib/binary_heap_collections.n.md`、`stdlib/tests/deque.n.md` / `tests/stdlib/deque_collections.n.md` を 3 worker の非重複 write scope に分割して並列移行した。
- List fixture では `new<i32>` / `push<i32>` / `cons<i32>` / `reverse<i32>` を postfix-free にした。
- BinaryHeap fixture では `new<i32>` / `with_capacity<i32>` / `push<i32>` を postfix-free にした。
- Deque fixture では `new<i32>` / `with_capacity<i32>` / `push_back<i32>` / `push_front<i32>` を postfix-free にした。
- 対象は `%List i32` / `%BinaryHeap i32` / `%Deque i32` local annotation、または receiver / argument 型から型根拠が明確な箇所に限定した。
- `nodesrc/test_stdlib_list_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` に、今回の対象 fixture が explicit producer / mutator postfix へ戻らないことを固定した。owner recovery、borrowed observer、cleanup 境界の検査は弱めていない。
- `rg -n "\b(new|push|cons|reverse)<" stdlib/tests/list.n.md tests/stdlib/list_collections.n.md` は 0 件になった。
- `rg -n "\b(new|with_capacity|push)<" stdlib/tests/binary_heap.n.md tests/stdlib/binary_heap_collections.n.md` は 0 件になった。
- `rg -n "\b(new|with_capacity|push_back|push_front)<" stdlib/tests/deque.n.md tests/stdlib/deque_collections.n.md` は 0 件になった。
- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_collection_cleanup_contract.js` は pass した。
- `node nodesrc/neplg21_syntax_migrate.js --check` は `would update 0 file(s)` になった。
- `node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build`、`git diff --check` は pass した。`git diff --check` は LF/CRLF warning のみ。
- `node nodesrc/tests.js -i stdlib/tests/list.n.md -i tests/stdlib/list_collections.n.md -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md -i stdlib/tests/deque.n.md -i tests/stdlib/deque_collections.n.md --no-tree -o tmp/neplg21-list-heap-deque-producer-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 15/17 件完了、15 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 BTree producer postfix checkpoint

- `stdlib/tests/btreemap.n.md`、`stdlib/tests/btreeset.n.md`、`tests/stdlib/pipe_collections.n.md` を 3 worker の非重複 write scope に分割して並列移行した。
- BTreeMap fixture では `new<i32,i32>` / `insert<i32,i32>` / `remove<i32,i32>` を postfix-free にした。
- BTreeSet fixture では `new<i32>` / `insert<i32>` / `remove<i32>` を postfix-free にした。
- `tests/stdlib/pipe_collections.n.md` では `pipe_btreemap_usage` / `pipe_btreeset_usage` section だけを対象にし、他 collection section の既存 `new<i32>` は次 checkpoint へ残した。
- 対象は `%BTreeMap i32 i32` / `%BTreeSet i32` local annotation、または pipe receiver / argument 型から型根拠が明確な箇所に限定した。
- `nodesrc/test_stdlib_btree_borrowed_observers.js` に、stdlib BTree fixture と pipe BTree section が explicit producer / mutator postfix へ戻らないことを固定した。borrowed observer、owner recovery、storage cleanup 境界の検査は弱めていない。
- `rg -n "\b(new|insert|remove)<" stdlib/tests/btreemap.n.md stdlib/tests/btreeset.n.md` は 0 件になった。
- `tests/stdlib/pipe_collections.n.md` の `pipe_btreemap_usage` / `pipe_btreeset_usage` section では、対象 postfix が 0 件になった。
- `node nodesrc/test_stdlib_btree_borrowed_observers.js`、`node nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`、`node nodesrc/test_stdlib_btreemap_report_contract.js`、`node nodesrc/test_stdlib_btreeset_report_contract.js`、`node nodesrc/test_stdlib_pipe_collections_report_contract.js` は pass した。
- worker 側で `node nodesrc/neplg21_syntax_migrate.js --check` は `would update 0 file(s)`、`git diff --check` は LF/CRLF warning のみで pass した。
- focused doctest は既存の per-program compile-time issue と同系の timeout。`stdlib/tests/btreemap.n.md` と `tests/stdlib/pipe_collections.n.md` の BTree doctest は compile timeout after 60000ms で型診断なし。

### 2026-05-26 Pipe/selfhost Vec producer postfix checkpoint

- `tests/stdlib/pipe_collections.n.md`、`tests/stdlib/selfhost_cliarg_parser.n.md`、`tests/stdlib/selfhost_cli_driver.n.md` / `tests/stdlib/fs.n.md` を 3 worker の非重複 write scope に分割して並列移行した。
- pipe fixture では `pipe_list_alias_chain`、`pipe_stack_alias_usage`、Queue / RingBuffer section の `new<i32>` / `push<i32>` を postfix-free にした。BTree section は前 checkpoint 済みで変更していない。
- selfhost CLI arg parser fixture では `%Vec str` local annotation と pipe receiver から解ける `v::new<str>` / `v::push<str>` を postfix-free にした。
- selfhost CLI driver fixture と FS fixture でも `%Vec str` local annotation と pipe receiver から解ける `v::new<str>` / `v::push<str>` を postfix-free にした。
- `v::free<str>` は producer / mutator ではないため今回対象外として残した。
- `nodesrc/test_stdlib_list_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`、`nodesrc/test_stdlib_ringbuffer_borrowed_observers.js`、`nodesrc/test_selfhost_cliarg_parser_doctest_contract.js`、`nodesrc/test_selfhost_cli_driver_report_contract.js`、`nodesrc/test_stdlib_fs_report_contract.js` に、今回対象の旧 producer / mutator postfix 再導入防止を追加した。コメント量を制限する検査ではない。
- `rg -n "\bv::(new|push)<str>|\b(new|push)<i32>" tests/stdlib/pipe_collections.n.md tests/stdlib/selfhost_cliarg_parser.n.md tests/stdlib/selfhost_cli_driver.n.md tests/stdlib/fs.n.md` は 0 件になった。
- targeted source policy 7 件は pass した。
- worker 側で `node nodesrc/neplg21_syntax_migrate.js --check` は `would update 0 file(s)`、`git diff --check` は LF/CRLF warning のみで pass した。
- selfhost/FS targeted doctest は既存の compile-time timeout または既存診断で完走せず。postfix-free 化に対する型診断は出ていない。

### 2026-05-26 Vec/sort/Stack producer postfix checkpoint

- `stdlib/tests/vec.n.md`、`tests/stdlib/vec_collections.n.md`、`tests/stdlib/sort.n.md`、`tests/stdlib/sort_simple.n.md`、`tests/stdlib/traits_order.n.md`、`tests/stdlib/selfhost_req.n.md`、`examples/rpn.nepl`、`examples/rpn_legacy.nepl`、`examples/bf.nepl` を 5 worker の非重複 write scope に分割して並列移行した。
- Vec fixture では `%Vec i32` / `%Vec u8` local annotation と receiver evidence から型が確定する `new<i32>` / `with_capacity<i32>` / `push<i32>` / `new<u8>` / `push<u8>` を postfix-free にした。
- sort fixture では `new<i32>` / `push<i32>` だけを対象にし、`sort_quick_ret<i32>` / `sort_heap_ret<i32>` / `sort_merge_ret<i32>` は owner-returning sort result 系として残した。
- Stack examples では `stk::push<i32>` を receiver / value evidence で `stk::push` へ移行し、値引数のない `stk::new<i32>` は `%Result Stack i32 Diag` typed local を置いてから `stk::new` を呼ぶ形へ移行した。
- `tests/stdlib/vec_collections.n.md` の `with_capacity<i32> neg` は match scrutinee に直接の戻り値期待型がなく、型根拠が弱いため残した。必要ならこの系統は expected type propagation / call reduction 改良 issue として分離する。
- `nodesrc/test_stdlib_vec_borrowed_observers.js`、`nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`、`nodesrc/test_selfhost_req_report_contract.js`、`nodesrc/test_stdlib_traits_order_report_contract.js` に、今回対象の旧 producer / mutator postfix 再導入防止を追加した。コメント量を制限する検査ではない。
- `rg -n "\b(new|with_capacity|push)<|stk::(?:new|push)<" <対象9 files>` は `with_capacity<i32> neg` 1 件だけが残る状態になった。
- targeted source policy、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build` は pass した。
- `node nodesrc/tests.js <対象9 files> --no-tree -o tmp/neplg21-vec-sort-stack-producer-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 16/44 件完了、16 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 Vec transform / sort_ret postfix checkpoint

- `stdlib/tests/vec.n.md`、`tests/stdlib/sort.n.md`、`tests/stdlib/traits_order.n.md`、`tests/stdlib/vec_collections.n.md`、Vec transform / sort doccomment examples、`tests/compiler/list_dot_map.n.md` を 5 worker の非重複 write scope に分割して並列移行した。
- `map<i32,i32>` / `filter<i32>` / `partition<i32>` / `take_while<i32>` / `drop_while<i32>` / `fold<i32,i32>` / `reduce<i32>` / `find<i32>` / `any<i32>` / `all<i32>` / `count<i32>` は、input `Vec i32`、callback 型、戻り値 annotation、または `%VecPartition i32` typed local から型根拠を明示して postfix-free にした。
- `sort_quick_ret<i32>` / `sort_heap_ret<i32>` / `sort_merge_ret<i32>` は、input `Vec i32` または `%Vec i32` result annotation から解ける call site と doccomment example を postfix-free にした。
- `tests/stdlib/vec_collections.n.md` の `with_capacity<i32> neg` は `%Result Vec i32 StdErrorKind` typed local を置いて `with_capacity neg` へ移行した。
- `tests/compiler/list_dot_map.n.md` では Vec map case だけを対象にし、`list::map<i32,i32>` と Result `map<i32,i32,i32>` は別 API として残した。
- `nodesrc/test_stdlib_vec_borrowed_observers.js` に、今回対象の Vec transform / traversal / sort_ret postfix 再導入防止を追加した。コメント量を制限する検査ではない。
- `rg -n "sort_(quick|heap|merge)_ret<|\b(map|filter|partition|take_while|drop_while|fold|reduce|find|any|all|count)<|with_capacity<i32>\s+neg" <対象files>` は、List / Result map の対象外 2 件だけが残る状態になった。
- targeted source policy、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build`、`git diff --check` は pass した。
- `node nodesrc/tests.js <対象14 files> --no-tree -o tmp/neplg21-vec-transform-sort-ret-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 16/61 件完了、16 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 Tutorial/core/compiler small helper postfix checkpoint

- `tutorials/getting_started/13_vec_basics.n.md` と `16_drop_and_cleanup.n.md` で、値引数のない `new<i32>` を `%Result Vec i32 StdErrorKind` typed local から解ける postfix-free `new` へ移行した。
- `stdlib/core/option.nepl` と `stdlib/core/result.nepl` の doctest で、`map` / `map_err` / `and_then` の explicit generic postfix を typed local または result annotation による型根拠へ移した。
- `tests/compiler/list_dot_map.n.md` で、List / Result / Vec の small helper call を postfix-free にした。List 型注釈は star import で `List i32` を参照し、名前空間付き type annotation を導入しない形にした。
- `tests/compiler/overload_nested_generic_push.n.md` で、nested `Result unit str` payload の `new` / `len` / `free` postfix を receiver 型または `%Vec Result unit str` local annotation から解ける形へ移行した。
- `nodesrc/test_tutorial_getting_started_current_style.js`、`nodesrc/test_core_option_doc_report_contract.js`、`nodesrc/test_core_result_doc_report_contract.js`、`nodesrc/test_neplg21_small_fixture_postfix_cleanup.js` に、今回対象の旧 generic postfix 再導入防止を追加した。コメント量を制限する検査ではない。
- targeted source policy 5 件、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build`、`git diff --check` は pass した。
- `node nodesrc/tests.js -i tutorials/getting_started/13_vec_basics.n.md -i tutorials/getting_started/16_drop_and_cleanup.n.md -i tests/compiler/list_dot_map.n.md -i tests/compiler/overload_nested_generic_push.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl --no-tree -o tmp/neplg21-small-helper-producer-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 7/18 件完了、7 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 Diagnostics/KP/cost postfix checkpoint

- `tests/stdlib/btree_array_cost.n.md` で、sorted-array BTreeMap / BTreeSet helper の explicit generic postfix を `%BTreeMap i32 i32` / `%BTreeSet i32` local annotation と borrowed receiver evidence へ移した。
- `tests/stdlib/capacity_stack.n.md` と `tests/stdlib/collections_diag.n.md` で、通常利用の Vec / Queue / RingBuffer helper postfix を撤廃した。raw memory generic API はこの checkpoint の対象外として保持した。
- `tests/stdlib/kp.n.md`、`stdlib/kp/kpsearch.nepl`、`stdlib/kp/kpprefix.nepl` で、KP / prefix sum / binary search の Vec helper postfix を撤廃し、近傍コメントの `Vec<i32>` を `Vec i32` へ更新した。
- `stdlib/alloc/diag/diag.nepl`、`stdlib/alloc/diag/error/diags.nepl`、`stdlib/alloc/diag/error/outcome.nepl` で、`Vec Diag` helper と `Outcome` doctest constructor call を postfix-free にした。generic 定義そのものは変更していない。
- `tests/compiler/neplg2.n.md` と `tests/compiler/overload.n.md` で、通常利用の List / Vec helper postfix を撤廃した。`pair_with_empty<i32>`、`Show::show<i32>`、generic 定義内の `v::new<.T>` など generic/overload 機能そのものの検査は保持した。
- `nodesrc/test_neplg21_diagnostics_kp_cost_postfix_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。既存の diag / BTree policy は postfix-free 表記へ追従し、Vec boundary と borrowed observer の検査は維持した。
- `node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、targeted policy、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js <対象11 files> --no-tree -o tmp/neplg21-diagnostics-kp-cost-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 7/121 件完了、7 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 stdlib small fixture postfix checkpoint

- `stdlib/tests/hashmap.n.md` と `stdlib/tests/hashmap_str.n.md` で、`hashmap_update_error_owner<...>` を `%HashMap ...` local annotation から解ける postfix-free call へ移行した。
- `stdlib/tests/error.n.md` で、`outcome_ok` / `outcome_err` / `result_to_outcome` の通常利用 generic postfix を撤廃し、必要な箇所には `%Outcome i32 StdErrorKind` / `%Result i32 StdErrorKind` local を型根拠として追加した。
- `stdlib/tests/bloom_filter.n.md` で、`contains<i32, DefaultHash32>` を borrowed receiver evidence から解ける `contains` へ移行した。
- `stdlib/tests/string.n.md` では prose の `Vec<str>` を `Vec str` へ更新した。raw memory generic API は今回対象外として保持した。
- `nodesrc/test_neplg21_stdlib_small_fixture_postfix_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。検査は今回移行した旧構文だけを対象にし、コメント量を制限していない。
- `node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js <対象5 files> --no-tree -o tmp/neplg21-stdlib-small-fixture-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 7/17 件完了、7 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 selfhost fixture postfix checkpoint

- `tests/stdlib/selfhost_cliarg_parser.n.md`、`tests/stdlib/neplg2_lexer.n.md`、`tests/stdlib/neplg2_type_arena.n.md`、`tests/stdlib/neplg2_diag_outcome.n.md` を 4 worker の非重複 write scope に分割して並列移行した。
- `selfhost_cliarg_parser` fixture では、`v::free<str>` を receiver 型から解ける postfix-free `v::free` へ移行した。selfhost parser が読む source string 内の旧構文期待値は対象外として保持した。
- `neplg2_lexer` fixture では、`SelfhostToken` の `unwrap` / `get` / `len` / `free` postfix を `%Option SelfhostToken` local と receiver evidence から解ける postfix-free call へ移行した。lexer input としての source string は保持した。
- `neplg2_type_arena` fixture では、`SelfhostTypeId` Vec helper の `new` / `push` / `vec_push_error_vec` / `vec_push_error_kind` / `free` postfix を撤廃した。値引数のない `new` は `%Result Vec SelfhostTypeId StdErrorKind` local を型根拠として追加した。
- `neplg2_diag_outcome` fixture では、`selfhost_outcome_*<...>` の通常利用を postfix-free にし、必要な箇所に `%Result SelfhostOutcome ... StdErrorKind` local を型根拠として追加した。
- `nodesrc/test_neplg21_selfhost_fixture_postfix_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。検査対象は今回移行した旧構文だけで、source string fixture やコメント量は制限していない。
- `node nodesrc/test_neplg21_selfhost_fixture_postfix_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js <対象4 files> --no-tree -o tmp/neplg21-selfhost-fixture-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 3 件完了、3 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-26 selfhost simple postfix checkpoint

- `tests/stdlib/selfhost_cli_driver.n.md`、`tests/stdlib/neplg2_module_loader.n.md`、`tests/stdlib/neplg2_parser.n.md`、`tests/stdlib/neplg2_module_graph.n.md`、`tests/stdlib/neplg2_stdlib_map.n.md`、`tests/stdlib/selfhost_req.n.md` を 5 worker の非重複 write scope に分割して並列移行した。
- `selfhost_cli_driver` fixture では、`v::free<str>` を `Vec str` receiver evidence から解ける postfix-free `v::free` へ移行した。embedded source string は selfhost frontend 入力として保持した。
- `neplg2_module_loader` と `neplg2_parser` fixture では、`unwrap<SelfhostModuleItem>` を `%Option SelfhostModuleItem` local に受けてから `unwrap` する形へ移行した。
- `neplg2_module_graph` と `neplg2_stdlib_map` fixture では、`unwrap<SelfhostModuleGraphEdge>` を `%Option SelfhostModuleGraphEdge` local に受けてから `unwrap` する形へ移行した。
- `selfhost_req` fixture では、`get<u8>` を `%Option u8` local に受けてから match し、`free<u8>` と `hashmap_update_error_owner<...>` を receiver / lhs annotation から解ける postfix-free call へ移行した。
- `nodesrc/test_neplg21_selfhost_simple_postfix_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。検査対象は今回移行した旧構文だけで、source string fixture やコメント量は制限していない。
- `node nodesrc/test_neplg21_selfhost_simple_postfix_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js <対象6 files> --no-tree -o tmp/neplg21-selfhost-simple-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 5 件完了、`selfhost_cli_driver` の embedded source string 由来 `lexer.string.invalid_escape` 1 件と compile timeout 4 件で、今回撤廃した postfix に対する型診断は出ていない。残留 runner は停止した。

### 2026-05-26 pipe/traits/sort postfix checkpoint

- `tests/stdlib/pipe_collections.n.md`、`tests/stdlib/traits_order.n.md`、`tests/stdlib/sort.n.md`、`tutorials/getting_started/02_test_harness.n.md`、`tutorials/getting_started/91_sort_search_prefixsum.n.md`、`tests/stdlib/selfhost_req.n.md` を 4 worker の非重複 write scope に分割して並列移行した。
- `pipe_collections` fixture では、BTreeMap / BTreeSet insert error helper と HashMap / HashSet update error owner helper の explicit generic postfix を error payload / lhs annotation から解ける postfix-free call へ移行した。
- `traits_order` fixture では、`Vec i32` receiver から型が決まる `get<i32>` と `free<i32>` を postfix-free にした。trait declaration / impl / generic function 検査は変更していない。
- `sort` fixture では、`%VecSortMergeError i32` lhs annotation から解ける `VecSortMergeError<i32>` constructor を postfix-free `VecSortMergeError` へ移行した。
- tutorial prose と `selfhost_req` comment では、`Result<unit,str>` / `Vec<i32>` / `Vec<u8>` を NEPLg2.1 表記へ更新した。実行コードや source string は変更していない。
- `nodesrc/test_neplg21_pipe_traits_sort_postfix_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。検査対象は今回移行した旧構文だけで、raw memory generic API、trait/generic 機能検査、source string、コメント量は制限していない。
- `node nodesrc/test_neplg21_pipe_traits_sort_postfix_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js <対象3 files> --no-tree -o tmp/neplg21-pipe-traits-sort-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 4 件完了、4 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-27 metadata/traits postfix checkpoint

- `tests/stdlib/collection_cleanup_contract.n.md`、`tests/stdlib/traits_serde.n.md`、`tests/stdlib/traits_hash.n.md`、`tests/compiler/generic_impl_trait_args.n.md`、`tests/compiler/prelude_copy.n.md`、`tests/compiler/move_effect.n.md`、`tests/compiler/typeannot.n.md` を 4 worker の非重複 write scope に分割して並列移行した。
- `collection_cleanup_contract` では、positive metadata observation に限定して `vec_empty` / `is_empty` / `len` / `cap` / `vec_partition_matched_len` の explicit generic postfix を receiver または lhs annotation から解ける postfix-free call へ移行した。compile_fail、negative checks、trait impl、raw/direct constructor 系は変更していない。
- `traits_serde` では、`deserialize<i32>` / `deserialize<bool>` を `%Result ... StdErrorKind` typed local に受けてから match する形へ移行し、失敗メッセージの旧表記も `deserialize i32` / `deserialize bool` に更新した。
- `traits_hash` では、通常呼び出し `use_hasher_twice<i32, StatefulHasher>` を argument evidence から解ける postfix-free call へ移行し、prose の `Hasher<.K>` を `Hasher .K` 表記へ更新した。`Hasher<.K>` trait bound 宣言そのものは generic 機能検査として保持した。
- compiler fixture の prose/comment では、自然文内の `MemPtr<...>` / `RegionToken<T>` / `Option<i32>` などを NEPLg2.1 表記へ更新した。`impl<.T>` や `size_of<T>` のように構文・intrinsic 名そのものを説明している箇所は保持した。
- `nodesrc/test_neplg21_metadata_traits_postfix_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。検査対象は今回移行した旧構文だけで、declaration line の trait bound、raw memory generic API、intrinsic、compile_fail fixture、コメント量は制限していない。
- worker 側では `node nodesrc/test_stdlib_collection_cleanup_contract.js`、`node nodesrc/test_stdlib_vec_borrowed_observers.js`、`node nodesrc/test_stdlib_traits_hash_report_contract.js`、`node nodesrc/test_stdlib_traits_serde_report_contract.js` が pass した。
- `node nodesrc/test_neplg21_metadata_traits_postfix_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js <対象3 files> --no-tree -o tmp/neplg21-metadata-traits-postfix.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 4 件完了、4 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-27 kpgraph/overload postfix checkpoint

- `stdlib/kp/kpgraph.nepl` では、BFS の `v::filled<i32>` / `v::get<i32>` / `v::replace<i32>` / `v::free<i32>` を、`Vec i32` receiver と value argument から解ける postfix-free call へ移行した。
- 同じファイルの doctest snippet と説明文も、利用者向けの現在形として `v::get` / `v::free` / `Vec i32` に更新した。
- `tests/compiler/overload.n.md` では、`overload_pair_field_from_generic_result_keeps_tuple_type` の `v::new<.T>` と `pair_with_empty<i32>` を、`let right %Vec .T` と `xs %Vec i32` の型根拠から解ける postfix-free call へ移行した。`Show::show<i32>` は `type.trait_method.type_args_unsupported` の診断 fixture として保持した。
- `nodesrc/test_neplg21_kpgraph_overload_postfix_cleanup.js` を追加し、今回撤廃した旧構文だけを検出するようにした。これはコメント量を制限する検査ではない。
- `node nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js`、`node nodesrc/test_neplg21_kpgraph_overload_postfix_cleanup.js`、`node nodesrc/test_neplg21_diagnostics_kp_cost_postfix_cleanup.js` は pass した。
- `tmp/neplg21-overload-pair-inference-smoke.neplg2` の direct `nepl-cli.exe --check --target core` で、generic call の型引数が argument evidence と typed local から解けることを確認した。
- `tests/compiler/overload.n.md::doctest#10` の focused doctest / native check は既存の compile-time 長時間化で timeout した。型診断は取得できていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 DropPayload positive lifecycle postfix checkpoint

- `tests/stdlib/collection_cleanup_contract.n.md` の `vec_push_accepts_drop_payload` positive path で、`new<DropPayload>` / `push<DropPayload>` / `len<DropPayload>` / `free<DropPayload>` を postfix-free call へ移行した。
- `new` は `let v0 %Vec DropPayload`、`push` は receiver `v0` と `(DropPayload 7)`、`len` / `free` は `v1: Vec DropPayload` から型が決まる。
- compile_fail / negative contract / raw boundary / trait impl の明示型引数は、診断目的を変えないため保持した。
- `nodesrc/test_neplg21_metadata_traits_postfix_cleanup.js` に今回の positive DropPayload lifecycle だけを検出する限定 pattern を追加した。コメント量を制限する検査ではない。
- `node nodesrc/test_neplg21_metadata_traits_postfix_cleanup.js` と `node nodesrc/test_stdlib_collection_cleanup_contract.js` は pass した。
- `node nodesrc/run_doctest.js -i tests/stdlib/collection_cleanup_contract.n.md -n 6 --dist web/dist` は外側 timeout。型診断は出ていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 SHA256 Vec helper postfix checkpoint

- `stdlib/alloc/hash/sha256/{api,compress,digest,padding,schedule}.nepl` で、`Vec i32` receiver / return annotation / value argument から型が決まる `new<i32>` / `with_capacity<i32>` / `push<i32>` / `get<i32>` / `len<i32>` / `free<i32>` を postfix-free call へ移行した。
- SHA256 module comment の `Vec<i32>` は NEPLg2.1 の `Vec i32` 表記へ更新した。
- raw memory / intrinsic API ではなく、`Vec i32` に閉じた通常 stdlib helper call だけを対象にした。
- `nodesrc/test_neplg21_sha256_postfix_cleanup.js` を追加し、今回撤廃した旧構文だけを検出するようにした。コメント量を制限する検査ではない。
- `node nodesrc/test_neplg21_sha256_postfix_cleanup.js`、`node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_hash_nmd_report_contract.js` は pass した。
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md --no-tree -o tmp/neplg21-sha256-postfix.json -j 1 --dist web/dist --assert-io` は compile timeout after 60000ms。型診断は出ていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 Vec facade doc postfix checkpoint

- `stdlib/alloc/collections/vec.nepl` の facade doc / doctest 例で、通常の利用例として残っていた `Vec<.T>`、`new<i32>`、`with_capacity<i32>`、`get<i32>`、`len<i32>`、`clear<i32>`、`free<i32>` を NEPLg2.1 表記へ移行した。
- すべて `%Vec i32` local / block result annotation / receiver evidence から型が決まる使用例であり、旧構文そのものの説明ではない。
- `nodesrc/test_neplg21_vec_doc_postfix_cleanup.js` を追加し、今回撤廃した old facade doc syntax だけを検出するようにした。コメント量を制限する検査ではない。
- `node nodesrc/test_neplg21_vec_doc_postfix_cleanup.js`、`node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`、`node nodesrc/test_stdlib_vec_borrowed_observers.js`、`node nodesrc/test_stdlib_vec_pop_doc_report_contract.js` は pass した。
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/neplg21-vec-doc-postfix.json -j 1 --dist web/dist --assert-io` は 3 doctest とも compile timeout after 60000ms。型診断は出ていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 core traits postfix checkpoint

- `stdlib/core/traits/deserialize.nepl` の doctest を、`deserialize<i32>` ではなく `%Result i32 StdErrorKind` typed local で復元先型を与える形へ移行した。
- `parse_err_to_std<bool>` / `parse_err_to_std<i32>` / `parse_err_to_std<i64>` / `parse_err_to_std<i128>` / `parse_err_to_std<f32>` / `parse_err_to_std<f64>` は、入力 `to_* s` の `Result T i32` と impl の戻り値型から `.T` が決まるため postfix-free call へ移行した。
- `stdlib/core/traits/hash.nepl` と `serialize.nepl` の doccomment で、利用者向けの prose に残っていた `Hasher<.K>`、`Hasher<MyKey>`、`Hash<i32>` 系、`Serialize<T, F>` を NEPLg2.1 表記へ更新した。
- `trait Hasher<.K: HashKey>`、`impl<.K: HashKey> Hasher<.K> for DefaultHash32`、`pub fn hash_with <.K: HashKey,.H: Hasher<.K>>` は、現行の declaration / impl / bound 構文なのでこの checkpoint では保持した。
- subagent の独立調査でも、`parse_err_to_std` は引数側の型から解ける安全な移行候補であり、trait declaration / impl / bound は残す対象と確認した。
- `nodesrc/test_neplg21_core_traits_postfix_cleanup.js` を追加し、今回撤廃した旧構文だけを検出するようにした。コメント量や doccomment の増加を制限する検査ではない。
- `node nodesrc/test_neplg21_core_traits_postfix_cleanup.js`、`node nodesrc/test_core_traits_doc_report_contract.js`、`node nodesrc/test_stdlib_traits_hash_report_contract.js`、`node nodesrc/test_stdlib_traits_serde_report_contract.js`、`node nodesrc/neplg21_syntax_migrate.js --check` は pass した。
- `target\debug\nepl-cli.exe --check -i tmp\neplg21_core_traits_deserialize_smoke.neplg2 --target core` は 124s timeout。残留 `node` / `nepl-cli` process は停止した。型診断は出ていないため、full smoke は performance issue 側で継続確認する。

### 2026-05-27 selfhost/TUI Vec helper postfix checkpoint

- `stdlib/platforms/wasix/tui/text/wrap.nepl` と `text.nepl` で、prose の `Vec<str>` を `Vec str` に更新し、`v::vec_empty<str>` / `v::push<str>` / `v::vec_push_error_vec<str>` / `v::new<str>` を postfix-free call へ移行した。
- `v::new` は値引数がなく型根拠が弱いため、`let initial %Result Vec str StdErrorKind v::new` の typed local を置いてから match する形にした。
- `stdlib/neplg2/core/infra/text.nepl` では、line start table の prose を `Vec i32` に更新し、`v::push<i32>` / `v::filled<i32>` を `Vec i32` receiver / value evidence から解ける postfix-free call へ移行した。
- `stdlib/neplg2/core/mono/mono.nepl` と `stdlib/neplg2/core/module/vfs.nepl` では、prose の `Option<SelfhostMonoInstanceId>` / `Vec<SelfhostVirtualFile>` を NEPLg2.1 表記にし、`v::new` は typed local、`v::push` / `v::vec_push_error_kind` は receiver / value evidence で postfix-free にした。
- 3 worker に TUI text、selfhost source text、selfhost mono/VFS を非重複 write scope として割り当て、親 agent が stale policy を NEPLg2.1 表記へ追従した。
- `nodesrc/test_neplg21_selfhost_tui_vec_postfix_cleanup.js` を追加し、今回撤廃した旧構文だけを検出するようにした。コメント量や doccomment の増加を制限する検査ではない。
- `node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`、`node nodesrc/test_selfhost_source_text_no_recursive_line_map.js`、`node nodesrc/test_selfhost_mono_instance_absence.js`、`node nodesrc/test_selfhost_cli_file_io_boundary.js`、`node nodesrc/test_neplg21_selfhost_tui_vec_postfix_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check` は pass した。
- `node nodesrc/tests.js -i stdlib/neplg2/core/mono/mono.nepl -i stdlib/neplg2/core/module/vfs.nepl --no-tree -o tmp/worker3-selfhost-mono-vfs.json -j 1` は両 doctest compile timeout。型診断は出ていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 collection storage Vec helper postfix checkpoint

- `adjacency_matrix` / `bitset` / `bloom_filter` / `counting_bloom_filter` の `Vec u8` storage helper で、`vec::get<u8>` / `vec::replace<u8>` / `vec::filled<u8>` / `vec::free<u8>` を receiver / value / return evidence から解ける postfix-free call へ移行した。
- `disjoint_set` / `fenwick` / `sparse_set` / `segment_tree` の `Vec i32` storage helper で、`vec::get<i32>` / `vec::replace<i32>` / `vec::filled<i32>` / `vec::free<i32>` / `vec::len<i32>` を postfix-free call へ移行した。
- prose の `Vec<u8>` / `Vec<i32>` も NEPLg2.1 の `Vec u8` / `Vec i32` 表記へ更新した。
- raw memory / intrinsic API / generic `.T` storage / `Option<.T>` slot storage はこの checkpoint の対象外として保持した。
- 3 worker に `Vec u8` storage、DisjointSet/Fenwick、SparseSet/SegmentTree を非重複 write scope として割り当て、親 agent が no-unsafe policy の stale call spelling を postfix-free contract へ追従した。
- `nodesrc/test_neplg21_collection_storage_vec_postfix_cleanup.js` を追加し、今回撤廃した旧構文だけを検出するようにした。コメント量や doccomment の増加を制限する検査ではない。
- targeted policy: `test_stdlib_adjacency_matrix_no_unsafe_unwraps.js`、`test_stdlib_bitset_no_unsafe_unwraps.js`、`test_stdlib_bloom_filter_no_unsafe_unwraps.js`、`test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`、`test_stdlib_disjoint_set_no_unsafe_unwraps.js`、`test_stdlib_fenwick_no_unsafe_unwraps.js`、`test_stdlib_sparse_set_no_unsafe_unwraps.js`、`test_stdlib_segment_tree_no_unsafe_unwraps.js` は pass した。
- `node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check` は pass した。
- SparseSet / SegmentTree API doctest 直接実行は 240s timeout。partial JSON は compile timeout のみで型診断は出ていない。

### 2026-05-27 collection wrapper Vec helper postfix checkpoint

- `list` / `stack` / `queue` / `ringbuffer` / `deque` / `binary_heap` / `btreeset` / `btreemap` / `hashset` / `hashmap` の wrapper / storage 層で、`vec::get<Option<...>>` / `vec::replace<Option<...>>` / `vec::filled<Option<...>>` / `vec::free<Option<...>>` などの Vec helper postfix を撤廃した。
- `list` の `vec::new` / `vec::with_capacity`、btree/hash storage の `vec::filled` は、値引数だけでは型根拠が弱い箇所に `%Result Vec ... StdErrorKind` または `%Option ...` local を置いてから postfix-free call へ移行した。
- `BinaryHeap<.T>` / `HashSet<.T,.H>` / `HashMap<.K,.V,.H>` などの constructor / type form と、`pub fn f <.T>` の generic declaration は現行構文として保持した。
- 5 worker に list、small linear collections、binary_heap、btree、hash storage を非重複 write scope として割り当て、親 agent が stale source policy の call spelling を NEPLg2.1 へ追従した。
- `nodesrc/test_neplg21_collection_wrapper_vec_postfix_cleanup.js` を追加し、今回対象ファイルの旧 `vec::...<...>` helper call だけを検出するようにした。コメント量や doccomment の増加を妨げる検査ではない。
- `node nodesrc/test_neplg21_collection_wrapper_vec_postfix_cleanup.js` と、list / binary_heap / queue_deque / ringbuffer / stack / btree / hashset / hashmap の関連 source policy は pass した。
- `node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- worker focused doctest では list と btree 系で compile timeout が出たが、partial JSON では型診断は出ていない。full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 selfhost / stdlib Vec alias postfix checkpoint

- `stdlib/neplg2/cli/**` の doccomment examples、`stdlib/neplg2/core/{hir,ty,module,resolve,infra,syntax}/**`、`stdlib/std/fs/**` に残っていた `v::new<T>` / `v::push<T>` / `v::replace<T>` / `v::with_capacity<T>` / `v::pop<T>` / `v::vec_empty<T>` を postfix-free call へ移行した。
- zero-argument の `v::new` / `v::vec_empty` / `v::with_capacity` など、型根拠が弱い箇所では `%Result Vec ... StdErrorKind` や `%Vec ...` local を置き、型注釈で選択する NEPLg2.1 方針に合わせた。
- 4 worker に CLI doc examples、selfhost HIR/TY、selfhost module/resolve/syntax/diag、std/fs を非重複 write scope として割り当てた。
- `nodesrc/test_neplg21_selfhost_stdlib_v_postfix_cleanup.js` を追加し、今回対象ファイルの旧 `v::...<...>` helper call だけを検出するようにした。コメント量や doccomment の増加を妨げる検査ではない。
- `node nodesrc/test_neplg21_selfhost_stdlib_v_postfix_cleanup.js`、`node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`、selfhost HIR/TY/CLI/module/diag/parser の focused policies は pass した。
- `node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- worker focused doctest では CLI examples が compile timeout。型診断は出ていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 selfhost prose / outcome / lexer postfix checkpoint

- `stdlib/neplg2/core/infra/outcome.nepl` で、doccomment の `Result<T,E>` / `SelfhostOutcome<T,E>` を NEPLg2.1 の `Result .T .E` / `SelfhostOutcome .T .E` 表記へ更新した。
- `selfhost_outcome_new` / `selfhost_outcome_from_result` / `selfhost_outcome_ok` / `selfhost_outcome_err` / `selfhost_outcome_free` などは、引数型・戻り値型・明示 local annotation から型が決まる箇所だけ postfix-free call へ移行した。
- `.E` が値から出にくい doctest と `selfhost_outcome_stage0` では、`%Result SelfhostOutcome i32 str StdErrorKind` local を置いてから `selfhost_outcome_ok` を呼ぶ形にした。
- `pub struct SelfhostOutcome<.T, .E>` は現行の generic declaration syntax なので保持した。
- `stdlib/neplg2/core/syntax/lexer/{tokenize,indent}.nepl` では、`push<T>` / `drop_last<T>` / `vec_push_error_vec<T>` を receiver / value / error payload evidence から解ける postfix-free call へ移行した。
- `new<SelfhostToken>` / `new<i32>` は値引数がないため、`%Result Vec SelfhostToken StdErrorKind` / `%Result Vec i32 StdErrorKind` typed local を置いてから `match` する形にした。
- selfhost CLI/module/syntax と stdlib fs の doccomment に残っていた `Vec<...>` / `RegionToken<u8>` / `MemPtr<u8>` などの prose 型表記を prefix 型式へ更新した。実コードの `alloc_region<u8>` など raw memory generic call はこの checkpoint の対象外として保持した。
- 4 worker に outcome、lexer、selfhost prose、stdlib fs prose を非重複 write scope として割り当て、親 agent が限定 regression を追加した。
- `nodesrc/test_neplg21_selfhost_prose_type_postfix_cleanup.js` を追加し、今回撤廃した旧構文だけを検出するようにした。コメント量や doccomment の増加を妨げる検査ではない。
- `lex_stack_drop_top` の source policy は、public Vec owner API を通る契約を維持したまま、旧 `drop_last<i32>` ではなく postfix-free `drop_last stack` を確認する形へ追従した。
- focused policy: `test_selfhost_diag_outcome_report_contract.js`、`test_selfhost_diag_split_contract.js`、`test_selfhost_lexer_report_contract.js`、`test_selfhost_lexer_split_contract.js`、`test_selfhost_cli_args_doc_report_contract.js`、`test_selfhost_import_spec_report_contract.js`、`test_selfhost_module_graph_report_contract.js`、`test_selfhost_token_split_contract.js`、`test_stdlib_fs_no_unsafe_unwraps.js`、`test_stdlib_fs_nmd_report_contract.js` は pass した。
- `node nodesrc/test_neplg21_selfhost_prose_type_postfix_cleanup.js` と `node nodesrc/neplg21_syntax_migrate.js --check` は pass した。
- `node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- worker focused doctest では outcome / CLI examples が compile timeout。型診断は出ていないため、full doctest green 化は performance issue 側で継続確認する。

### 2026-05-27 selfhost import scan doc unwrap checkpoint

- `stdlib/neplg2/core/module/import_scan.nepl` の doctest helper `record_at` に残っていた `unwrap<SelfhostImportRecord>` を postfix-free `unwrap` へ移行した。
- `record_at` の戻り値型 `SelfhostImportRecord` と `v::get records idx` の入力 `&Vec SelfhostImportRecord` から `Option SelfhostImportRecord` が決まるため、明示 type args を残す必要がない。
- `nodesrc/test_neplg21_selfhost_prose_type_postfix_cleanup.js` に、selfhost doccomment 内の `unwrap<Selfhost...>` 系 helper postfix を検出する pattern を追加した。コメント量や doccomment の増加を妨げる検査ではない。
- `rg -n "\\b[A-Za-z_][A-Za-z0-9_:]*<[A-Za-z_.]" stdlib/neplg2 -g "*.nepl"` は、現行 declaration syntax の `pub struct SelfhostOutcome<.T, .E>` だけになった。
- `node nodesrc/test_neplg21_selfhost_prose_type_postfix_cleanup.js`、`node nodesrc/test_selfhost_import_spec_report_contract.js`、`node nodesrc/test_selfhost_module_graph_report_contract.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。

### 2026-05-27 stdlib prose type notation checkpoint

- `stdlib/alloc/collections/**`、`stdlib/core/**`、`stdlib/alloc/string/**`、`stdlib/alloc/io/**`、`stdlib/std/text/**`、`stdlib/std/test/**`、`tests/stdlib/**` の doccomment prose / markdown prose に残っていた旧 `Type<...>` 説明を NEPLg2.1 prefix 型式へ更新した。
- 4 worker に Vec prose、非 Vec collection prose、core/string/io/text/test prose、tests/stdlib prose を非重複 write scope として割り当てた。
- 変更対象は `.nepl` の `//:` prose と `.n.md` の fenced block 外本文・見出しに限定した。`//:|` doctest code、実コード、compile_fail / negative fixture、raw memory generic API、intrinsic、現行 generic declaration syntax は保持した。
- `nodesrc/test_neplg21_prose_type_notation_cleanup.js` を追加し、`nodesrc/run_source_policy_regressions.js` へ組み込んだ。検査は prose の旧型適用表記だけを検出し、コメント量や doccomment の増加を妨げない。
- `node nodesrc/test_neplg21_prose_type_notation_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、targeted doc/source policy、`trunk build`、`node nodesrc/run_source_policy_regressions.js --warn-only` は pass した。
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/neplg21-prose-type-notation-cleanup.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 18/67 件完了、12 pass、compile timeout 5 件、`memory_safety.n.md::doctest#14` の既存 raw memory case で `resource.cell.uninit` 1 件。今回の prose-only 変更に伴う型診断は出ていない。

### 2026-05-27 streamio scanner postfix checkpoint

- `stdlib/std/streamio/scanner.nepl` と `stdlib/std/streamio/scanner/state.nepl` で、`Vec i32` cursor storage に閉じた `vec::free<i32>` / `vec::get<i32>` / `vec::replace<i32>` / `vec::filled<i32>` を postfix-free call へ移行した。
- `nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_streamio_scanner_boundary.js` は、typed cursor storage boundary を維持したまま NEPLg2.1 の call spelling へ追従した。
- `nodesrc/test_neplg21_streamio_scanner_postfix_cleanup.js` を追加し、今回対象ファイルの旧 `vec::...<i32>` helper call だけを検出するようにした。コメント量や doccomment 増加は制限しない。
- subagent 分類では、`tests/stdlib/**` の残件は raw memory boundary / compile_fail / source string fixture が主で、今回すぐに通常 API 移行へ回す対象はなかった。`stdlib/alloc/collections/vec/**` の positive doctest と、`stdlib/std/**` / `stdlib/nm/**` の prose-only 残件は次以降の安全な分割候補として残した。
- `node nodesrc/test_neplg21_streamio_scanner_postfix_cleanup.js`、streamio targeted source policy、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build` は pass した。
- `node nodesrc/tests.js -i stdlib/std/streamio/scanner.nepl -i stdlib/std/streamio/scanner/state.nepl --no-tree -o tmp/neplg21-streamio-scanner-postfix.json -j 1 --dist web/dist --assert-io` は `scanner.nepl::doctest#1` の compile timeout after 60000ms。型診断は出ていない。

### 2026-05-27 prose tail type notation checkpoint

- `stdlib/std/**`、`stdlib/nm/**`、`stdlib/neplg2/README.md` に残っていた prose-only の旧 `Type<...>` 型表記を NEPLg2.1 prefix 型式へ更新した。
- 2 worker に std raw owner 型説明と nm/selfhost README 型説明を非重複 write scope として割り当てた。実コード、`//:|` doctest code、raw memory generic call、HTML literal、generic declaration は変更していない。
- `Result<unit,str>`、`RegionToken<u8>`、`MemPtr<u8>`、`Vec<Node>`、`Vec<Inline>`、`Option<SelfhostMonoInstanceId>`、`Vec<SelfhostMonoInstanceRecord>` を prefix 型式へ移行した。
- `nodesrc/test_neplg21_prose_type_notation_cleanup.js` の対象を `stdlib/std` と `stdlib/nm` 全体、`stdlib/neplg2/README.md` へ拡張した。検査は prose の旧型適用表記だけを検出し、コメント量や doccomment 増加を制限しない。
- read-only subagent が `doc/examples/**` の `%fn*` 残件を source example migration 候補として分類した。これは prose-only ではないため次 checkpoint へ分離する。
- `node nodesrc/test_neplg21_prose_type_notation_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build` は pass した。
- `node nodesrc/tests.js -i stdlib/std/test.nepl -i stdlib/std/env/cliarg/raw.nepl -i stdlib/std/env/cliarg/cstr.nepl -i stdlib/std/stdio/write/fd.nepl -i stdlib/std/stdio/read/buffer.nepl -i stdlib/platforms/wasix/tui/tty.nepl -i stdlib/nm/html_gen.nepl -i stdlib/nm/parser/document.nepl -i stdlib/nm/README.n.md --no-tree -o tmp/neplg21-prose-tail-type-notation.json -j 1 --dist web/dist --assert-io` は外側 timeout。partial JSON は 5/14 件完了、5 件 compile timeout after 60000ms、型診断なし。残留 runner は停止した。

### 2026-05-27 doc examples impure fn checkpoint

- `doc/neplg2/neplg21_syntax_migration_plan.md` の正規表記に合わせ、`doc/examples/**` に残っていた旧 draft `%fn*` / `fn*` を `%impure fn` / `impure fn` へ移行した。
- 2 worker に `01_basics` / `07_modules` と `05_io_and_resources` を非重複 write scope として割り当てた。
- `05_io_and_resources.nepl` の multi-argument function example は `%impure fn str fn str Result unit IoError` とし、2 引数目以降を表す nested pure `fn str ...` は保持した。
- `nodesrc/test_neplg21_doc_examples_impure_fn_cleanup.js` を追加し、`doc/examples/*.nepl` に旧 draft spelling が戻らないようにした。検査は `%fn*` / `fn*` だけを検出し、コメント量や説明追加を制限しない。
- `rg -n "%fn\\*|fn\\*" doc/examples -g "*.nepl"` は 0 件、`node nodesrc/test_neplg21_doc_examples_impure_fn_cleanup.js`、`node nodesrc/neplg21_syntax_migrate.js --check`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`trunk build` は pass した。
- `node nodesrc/tests.js -i doc/examples/01_basics.nepl -i doc/examples/05_io_and_resources.nepl -i doc/examples/07_modules.nepl --no-tree -o tmp/neplg21-doc-examples-impure-fn.json -j 1 --dist web/dist --assert-io` は `nodesrc/tests/no-runnable-doctests`。対象 `doc/examples` は runner 上 runnable doctest として収集されなかった。

### 2026-05-28 collection cleanup contract postfix checkpoint

- `tests/stdlib/collection_cleanup_contract.n.md` の collection cleanup 契約 fixture で、receiver / 引数 / 戻り値型から確定できる public collection API 呼び出しを generic postfix なしへ移行した。
- `new` / `with_capacity` のように値引数だけでは payload 型が出ない producer は、`%Result Vec ... StdErrorKind` local を置いてから返す形にし、関数名ではなく型注釈で選択する NEPLg2.1 方針に合わせた。
- Drop payload を扱う positive fixture は、`free` が Drop cleanup overload を通るため `main` を `%impure fn unit i32` にした。effect 検査を弱めず、契約例の側を正しい effect boundary へ合わせた。
- `HashMap` 内部の storage accessor 呼び出しに残っていた `hashmap_storage_states<...>` / `hashmap_storage_keys<...>` / `hashmap_storage_values<...>` を、storage receiver と期待型から解決する形へ整理した。
- `bloom_filter/mutation.nepl` と `counting_bloom_filter/mutation.nepl` は `&Vec u8` を公開型シグネチャで使うため、`alloc/collections/vec` import を明示し、prefix type arity preload が facade 経由の偶然に依存しないようにした。
- overload selection は、構造的に候補が materialize された後、全候補が trait bound 不足だけで拒否された場合に `type.trait_bound.unsatisfied` を報告するようにした。arity mismatch や argument type mismatch は従来どおり `type.overload.no_match` 系へ残す。
- subagent 2 件で、fixture / stdlib import / compiler diagnostic の切り分けと、trait-bound-only overload rejection の診断方針を独立レビューした。
- `nodesrc/test_neplg21_collection_cleanup_contract_postfix_cleanup.js` を追加し、今回撤廃した旧 postfix call だけを fenced executable fixture から検出するようにした。コメント量や doccomment 増加を妨げる検査ではない。
- 検証: `cargo fmt -p nepl-core --check`、`cargo check -p nepl-core`、`cargo check --manifest-path nepl-web/Cargo.toml`、`cargo test -p nepl-core --test neplg2 overload_selection_reports_trait_bound_when_all_candidates_fail_bounds -- --nocapture`、`trunk build`、`node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/neplg21-collection-cleanup-contract-postfix-20260528-final.json -j 1 --dist web/dist --assert-io` は pass。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は exit 0。既存 warn-only 警告として documentation gap、Vec cleanup stale NEPLg2.0 regex、typecheck/driver.rs line split limit が残る。

## 検証

Run stdlib/source policy tests, trunk build, and nodesrc CLI JSON tests after migration.
