---
id: ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84
title: "stdlib raw-memory-backed APIs require staged effect migration"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-05-06
target: "stdlib/core/mem.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, nepl-core/src/typecheck.rs"
---

# ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84: stdlib raw-memory-backed APIs require staged effect migration

## 概要

core/mem raw primitives are still exposed as Pure because large parts of stdlib call alloc_raw/dealloc_raw/load/store from functions whose public signatures are currently pure. Marking those primitives impure in the compiler immediately causes widespread D3025 failures in Vec, string, IO, fs, diag, and stdio helpers.

## 対象

- `stdlib/core/mem.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, nepl-core/src/typecheck.rs`

## 根拠

- `tests/compiler/move_effect.n.md` の既存正常系「pure からメモリ操作を呼べる」は、pure `compute` から `alloc_raw` / `store_i32` / `load_i32` / `dealloc_raw` を呼び、`ret: 123` で通る。
- compiler 側で raw memory boundary 内の raw primitive を `Effect::Impure` として登録する試作を行うと、`tests/compiler/move_effect.n.md` の stdlib import ケースで `stdlib/alloc/collections/vec.nepl:278` の `store<.T>`、`vec.nepl:648` の `load<.T>`、`stdlib/alloc/string.nepl`、`stdlib/std/fs.nepl`、`stdlib/std/stdio.nepl`、`stdlib/std/streamio.nepl` などが一斉に D3025 になる。
- これは compiler の effect 判定だけの問題ではなく、stdlib が raw memory backed helper を pure API として公開・利用している設計移行の問題である。

## 問題

core/mem raw primitives are still exposed as Pure because large parts of stdlib call alloc_raw/dealloc_raw/load/store from functions whose public signatures are currently pure. Marking those primitives impure in the compiler immediately causes widespread D3025 failures in Vec, string, IO, fs, diag, and stdio helpers.

## 影響

The compiler cannot close the raw memory effect boundary issue without either breaking current stdlib APIs or introducing a richer internal memory effect. Pure source can still reach allocation or raw storage through stdlib wrappers, so the effect model remains unsound for self-host planning.

## 修正方針

Stage the migration: introduce a compiler-level internal/unsafe memory effect or explicit stdlib unsafe boundary, update raw-memory-backed stdlib APIs to either be impure or wrap an internal effect safely, and only then make core/mem raw primitives externally impure by default.

## 検証

Add compile_fail tests for user pure calls to raw primitives and stdlib migration tests that show intended safe wrappers either require impure context or are proven pure through the new internal effect boundary.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-04-28 issue 整理

この issue は Stage 6 の stdlib migration parent とする。raw-memory-backed 実装を禁止する issue ではなく、raw memory discipline を public API と利用者 code へ押し出さないための移行 issue である。

compiler 側の Resource IR / effect model が先行して整うまでは、既存 stdlib API を一括 impure 化しない。移行順は `core/mem` internal/public 境界、`Vec` / `StringBuilder` の owner token 移行、collection drop contract、self-host buffer API の順にする。stdlib 側で compiler 修正が必要になった場合は core issue と混ぜず、別 issue として分離する。

## 2026-04-28 memory model 方針レビュー追記

現在の stdlib は `Vec`、`string`、`io`、`fs`、`stdio`、`streamio` などの実装で raw memory backed helper を pure API の内部から呼んでいる。内部 mutation を使うこと自体は問題ではないが、raw address や storage identity が safe surface へ漏れる場合は referential transparency を compiler が証明できない。

したがって stdlib 側の方向は「raw memory を使わない」ではなく、「raw memory を public API discipline として利用者へ押し出さない」に修正する。具体的には、内部 builder / collection storage は `InternalAlloc` と owner token に閉じ、公開 API は Copy read、move-out、drop/free obligation を型と Resource IR で区別する。

self-host 実装では、S1/S2 の文字列走査・token 配列・diagnostic からこの方針を適用する。短期的には既存 `Vec` / `StringBuilder` を使って開始できるが、compiler 本体へ raw `MemPtr` 操作を直接持ち込む実装は避ける。

## 2026-04-28 stdlib full review 追記

最新 main (`0e6ffae`) で stdlib の source policy と doctest を再確認した。

- `nodesrc/test_stdlib*.js` の source policy は全件 pass。unsafe unwrap、match decision tree、NM raw aggregate detour、StringBuilder ownership comment、diag/error compact layout policy は現状の静的検査変更に追従できている。
- `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-full-review-after-diag-policy.json -j 4`: `total=80`, `passed=80`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-full-review-after-diag-policy.json -j 4`: `total=311`, `passed=303`, `failed=7`, `errored=1`
- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --no-tree -o tmp/traits-hash-timeout-review-20260428.json -j 1`: `total=6`, `passed=6`

`tests/stdlib` の残件は、source policy の未追従ではなく、strict move checking が non-Copy owner を raw storage に置く古い fixture / API design を拒否しているものだった。

- `tests/stdlib/capacity_stack.n.md::doctest#6`: `Vec<Kind>` grow が `D3100 reallocating raw memory place containing non-Copy value: $memptr:v_data+?`
- `tests/stdlib/json_typed_values.n.md::doctest#2`: `Vec<JsonValue>` grow が同種の `D3100`
- `tests/stdlib/json_typed_values.n.md::doctest#3/#4`: structured JSON payload の raw `data` owner を再利用して `D3100 use of moved raw memory place: data`
- `tests/stdlib/fs.n.md::doctest#5/#6`: `std/test` result aggregation path で `D3100 use of moved raw memory place: popped`
- `tests/stdlib/neplg2_diag_outcome.n.md::doctest#3`: `SelfhostOutcome` result cell が `D3100 overwrite of raw memory place containing non-Copy value: $memptr:result_ptr`
- `tests/stdlib/traits_hash.n.md::doctest#5`: broad parallel run では 20s timeout したが、focused run では 6/6 pass したため、現時点では再現性ある issue として分離しない。

このため、self-host 実装開始の観点では、文字列・診断 text・NM direct serializer・source policy は前進しているが、typed JSON value、generic `Vec<T>` with enum/non-Copy payload、`std/test` aggregation、`SelfhostOutcome` cell owner は Resource IR / owned collection model の移行対象として残る。現時点で新規 issue 追加ではなく、この parent issue の Stage 6 入力として扱う。

## 2026-05-05 selfhost CLI driver ResourceIR timeout 解消後の追記

`ISS-20260505T132758518Z-RESOURCEIR-INITIALIZED-SUMMARIES-KEE-A65C9148` で ResourceIR summary の unbounded projection 増殖を止めた後、`tmp\selfhost_cli_driver_doctest2_latest.nepl` の native wasm emit は timeout ではなく約 103 秒で `resource.raw.unsafe_memory_boundary` に到達した。

代表的な診断は次の raw-memory-backed stdlib helper に集中している。

- `stdlib/alloc/string.nepl`: `concat_result`, `len`, `string_byte_at_unchecked`, `string_finish_base`, `string_from_mem_unchecked_result`, `sb_append_result`, `sb_build_result`, `from_u128_radix`
- `stdlib/alloc/collections/vec.nepl`: `get`, `push`

これは user source の raw memory 直呼びを許すべき問題ではなく、stdlib 内部実装の raw memory boundary / internal unsafe effect / public safe API の責務分割が未完了であることを示している。次の対応では、ファイル全体を安易に許可して静的検査を弱めるのではなく、compiler-owned raw boundary capability と stdlib safe wrapper の境界を再設計し、selfhost driver が codegen まで進める形にする。

## 2026-05-05 operation-only SourceCapabilities 追記

`core/mem.nepl` と `stdlib` の safe wrapper 実装を同じ raw memory boundary として扱うと、`RawAddressEscapeFromInternalAlloc`、raw cell、owner obligation まで一括で抑制される。これは user-facing safe API の内部 raw operation を許す目的を超えて、raw address identity や owner/cell 検査の漏れを隠すため不適切である。

そのため SourceMap capability を次の 2 軸へ分離した。

- `raw_memory_operations`: compiler-owned stdlib 実装ファイル内の raw load/store/copy/fill 呼び出しを許可する。
- `raw_address_escape`: raw address identity escape と full raw memory boundary を許可する。

適用範囲は次の通り。

- `stdlib/core/mem.nepl`: full raw memory boundary (`raw_memory_operations` + `raw_address_escape`)。
- `stdlib/alloc/string.nepl`、`stdlib/alloc/collections/vec.nepl`: operation-only boundary (`raw_memory_operations` のみ)。
- user source とその他 stdlib: capability なし。

compiler gate では `UnsafeMemoryInPureFunction` だけを `raw_memory_operations` で許可し、`RawAddressEscapeFromInternalAlloc` は `raw_address_escape` がある場合だけ許可する。raw cell gate と owner obligation gate は引き続き full raw memory boundary のみで除外する。これにより `String` / `Vec` の内部 raw operation は selfhost 用 safe wrapper 実装として扱える一方、raw address の外部漏洩や owner/cell 検査の欠落は抑制されない。

検証結果:

- `cargo test -p nepl-core compiler::tests::resource_effect_gate_splits_raw_operation_and_identity_escape_capabilities -- --nocapture`: pass
- `cargo test -p nepl-core loader::tests::source_capabilities_split_stdlib_raw_memory_files -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo fmt --check -p nepl-core`: pass
- `cargo build -p nepl-cli`: pass
- rebuilt `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2_latest.nepl --target std --stdlib-root stdlib --emit wasm`: `resource.raw.unsafe_memory_boundary` は出ず、stderr 0 行のまま 240 秒 timeout。

このため、今回露出していた `string.nepl` / `vec.nepl` の unsafe memory boundary blocker は解消した。selfhost driver の完走は post-check/codegen timeout の既存 issue `ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C` に戻る。

## 2026-05-05 selfhost CLI arg storage 追記

`ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C` の修正後、`tests/stdlib/selfhost_cli_driver.n.md::doctest#2` は release web dist で pass するようになった。一方、同じ file の focused run では `doctest#1/#3` が次の診断で失敗する。

- `resource.raw.unsafe_memory_boundary`: pure function `selfhost_cli_arg_at__i32_i32_i32__Option_T_str__pure` が `/stdlib/neplg2/cli/args/parse.nepl:57` で `load<str>` を実行している。

これは codegen timeout ではなく、selfhost CLI argument storage が raw-memory-backed `Vec<str>` layout を直接読み、operation-only raw memory boundary の対象にもなっていないことによる stdlib/neplg2 API migration 残件である。単に `selfhost_cli_arg_at` を impure 化するだけでは CLI option parse API 全体の surface effect を変えるため、`Vec` safe accessor / argument storage の責務分割として別途対応する。

## 2026-05-06 selfhost CLI arg storage 対応追記

`tests/stdlib/selfhost_cli_driver.n.md` の stdout report 移行を再開するにあたり、`selfhost_cli_arg_at` の raw `load<str>` を廃止した。対応は `stdlib/neplg2/cli/args/parse.nepl` の parser state machine を `data/count` ではなく `&Vec<str>/count` で走査し、要素取得を `alloc/collections/vec::get<str>` へ委譲する形で行った。

この対応により、selfhost CLI parser は raw address を保持せず、raw memory operation capability の対象外のまま pure parser として残る。raw-memory-backed storage の内部操作は `Vec` 実装側へ閉じ、CLI parser の責務は token の分類と state transition に限定される。

検証結果:

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md -o tmp/selfhost_cliarg_parser_after_notree.json -j1 --dist web/dist --no-tree`: total=10, passed=10, failed=0。
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md -o tmp/selfhost_cli_driver_after_notree.json -j1 --dist web/dist --no-tree`: total=3, passed=3, failed=0。

## 2026-05-06 alloc/io と std/text raw boundary 再確認

`ISS-20260505T152927005Z-STD-TEST-CHECKS-EXIT-CODE-CAN-BYPASS-5204DA08` の stdout report 移行検証中に、`tests/stdlib/selfhost_cli_file_io.n.md` と `tests/stdlib/text_utf8.n.md` が compile phase で `resource.raw.unsafe_memory_boundary` に到達することを再確認した。

代表診断:

- `stdlib/alloc/io.nepl:202`: pure function `io_bytebuf_from_str_result__str__Result_T_E_ByteBuf_StdErrorKind__pure` が `mem_copy` を呼ぶ。
- `stdlib/std/text.nepl:64`: pure function `text_utf8_byte_at__MemPtr_T_u8_i32__Result_T_E_i32_StdErrorKind__pure` が `load_u8` を呼ぶ。

検証結果:

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/selfhost_cli_file_io_stdout_contract.json -j1 --dist web/dist`: total=4, passed=0, failed=4。
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text_utf8_stdout_contract.json -j1 --dist web/dist`: total=9, passed=2, failed=7。

これは今回の stdout report migration の退行ではなく、operation-only raw memory boundary の適用範囲が `stdlib/alloc/string.nepl` / `stdlib/alloc/collections/vec.nepl` に限られており、`alloc/io` と `std/text` の safe wrapper 内部 raw operation にはまだ整理が届いていないことを示す。

次の対応では、`io_bytebuf_from_str_result` と `text_utf8_byte_at` を単に impure 化して利用側へ効果を漏らすのではなく、safe public API と compiler-owned raw operation boundary を分離する。`alloc/io` と `std/text` に operation-only capability を与える場合も、`raw_address_escape`、raw cell、owner obligation の検査は抑制しないことを必須条件とする。

## 2026-05-06 alloc/io と std/text operation-only boundary 対応

`ISS-20260505T154332456Z-ALLOC-IO-AND-STD-TEXT-SAFE-WRAPPERS--FA2B4CA6` で、`alloc/io` と `std/text` の safe wrapper 内 raw operation を operation-only raw memory boundary に追加した。

今回の対応では、`SourceCapabilities` の full boundary を広げていない。`raw_memory_operations` だけを許可し、`raw_address_escape` は `false` のままにしたため、raw address identity escape、raw cell state、owner obligation の診断は引き続き有効である。対象 module は `StdlibRawMemoryOperationsModule` enum として `AllocString` / `AllocVec` / `AllocIo` / `StdText` に整理した。

検証結果:

- `cargo test -p nepl-core loader::tests::source_capabilities_split_stdlib_raw_memory_files -- --nocapture`: pass。
- `cargo test -p nepl-core compiler::tests::resource_effect_gate_splits_raw_operation_and_identity_escape_capabilities -- --nocapture`: pass。
- `trunk build --release`: pass。
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/selfhost_cli_file_io_raw_boundary_after.json -j1 --dist web/dist`: total=4, passed=4。
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text_utf8_raw_boundary_after.json -j1 --dist web/dist`: total=9, passed=9。
