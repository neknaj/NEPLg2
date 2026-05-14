---
id: ISS-20260514T190700987Z-SELF-HOST-CLI-ARGS-PARSER-READS-VEC--5193AE7F
title: "Self-host CLI args parser reads Vec str storage through raw memory"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/neplg2/cli/args/parse.nepl, nodesrc/test_selfhost_cli_args_no_owner_field_reads.js, tests/stdlib/selfhost_cliarg_parser.n.md"
---

# ISS-20260514T190700987Z-SELF-HOST-CLI-ARGS-PARSER-READS-VEC--5193AE7F: Self-host CLI args parser reads Vec str storage through raw memory

## 概要

stdlib/neplg2/cli/args/parse.nepl borrows Vec<str> but immediately converts it to data_mem_ptr/mem_ptr_addr and load<str> for indexing. This brings raw Vec storage identity into the self-host CLI parser even though str is Copy and Vec.get<str> can perform checked reads.

## 対象

- `stdlib/neplg2/cli/args/parse.nepl, nodesrc/test_selfhost_cli_args_no_owner_field_reads.js, tests/stdlib/selfhost_cliarg_parser.n.md`

## 根拠

- `selfhost_cli_parse_args` / `selfhost_cli_parse_argv` が `v::data_mem_ptr<str>` を `mem_ptr_addr` へ変換し、`selfhost_cli_arg_at` が `load<str>` と `size_of<str>` で `Vec<str>` storage を直接走査していた。
- `str` は `Copy` なので、parser の read-only observation は `Vec` の public observer `v::get<str>` で表現できる。
- raw storage 走査を外すと parser caller 側の `Vec<str>` owner obligation が表面化したため、parser / driver doctest 側で入力 `Vec<str>` を明示的に解放する必要があった。

## 問題

stdlib/neplg2/cli/args/parse.nepl borrows Vec<str> but immediately converts it to data_mem_ptr/mem_ptr_addr and load<str> for indexing. This brings raw Vec storage identity into the self-host CLI parser even though str is Copy and Vec.get<str> can perform checked reads.

## 影響

The self-host parser depends on the transitional Vec storage layout and raw-memory-boundary evidence, weakening Stage 6's goal that self-host compiler layers use safe public collection observers rather than raw storage discipline.

## 修正方針

Change selfhost_cli_arg_at/next_value/parse_loop to carry a borrowed Vec<str> and use v::get<str>. Remove core/mem/raw imports from parse.nepl and update source policy to forbid raw Vec storage reads in the self-host CLI args parser.

## 解決内容

- `stdlib/neplg2/cli/args/parse.nepl` から `core/mem` / `core/mem/internal` / `core/mem/raw` import を削除した。
- `selfhost_cli_arg_at` / `selfhost_cli_next_value` / `selfhost_cli_parse_loop` は `&Vec<str>` を受け取り、index read は `v::get<str>` に限定した。
- `selfhost_cli_parse_args` / `selfhost_cli_parse_argv` は raw data pointer を抽出せず、borrowed `Vec<str>` と `v::len<str>` の count を parser loop へ渡す。
- `tests/stdlib/selfhost_cliarg_parser.n.md` と self-host CLI option / driver doctest の入力 `Vec<str>` owner を、parse 成功・失敗の両 branch で解放するようにした。
- `nepl-core/tests/resource_ir.rs` に、`Vec.get<str>` の Copy read を `Option<str>` field へ返しても `Vec` storage owner を移動しない Resource IR 回帰を追加した。
- focused parser suite は通過したが、10 doctest 合計で約 130 秒かかり、各 case の `run_ms` は 5-10ms、`compile_ms` は約 10-17s だった。この性能問題は `ISS-20260514T193353066Z-SELFHOST-CLI-ARG-PARSER-DOCTEST-SUIT-CF8C1BA8` に分離した。

## 検証

- `node nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`: pass
- `node nodesrc/test_selfhost_cli_args_types_split.js`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_accepts_vec_get_copy_str_option_return -- --nocapture`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/cli/args/parse.nepl --no-tree -o tmp/agent1-selfhost-cli-args-vec-observer-parse-module-final.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/neplg2/cli/args/options.nepl --no-tree -o tmp/agent1-selfhost-cli-args-vec-observer-options-module-2.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md --no-tree -o tmp/agent1-selfhost-cli-args-vec-observer-tests-final.json -j 1 --dist web/dist --assert-io`: total=10, passed=10
- `stdlib/neplg2/cli/driver.nepl` / `tests/stdlib/selfhost_cli_driver.n.md` は parser input owner cleanup を更新済み。ただし focused run は既知の `ISS-20260514T150128082Z-JSON-BUILDERS-STILL-DEPEND-ON-NON-CO-493D5962` により `alloc/encoding/json` 側で compile fail するため、この issue の完了条件には含めない。
