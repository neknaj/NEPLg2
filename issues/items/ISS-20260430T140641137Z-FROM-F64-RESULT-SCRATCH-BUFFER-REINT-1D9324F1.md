---
id: ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1
title: "from_f64_result scratch buffer reintroduces Resource IR moved-cell failures"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/string.nepl, stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, GitHub Actions run 25157230630 stdlib-test"
---

# ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1: from_f64_result scratch buffer reintroduces Resource IR moved-cell failures

## 概要

GitHub Actions run 25157230630 stdlib-test reports resource.cell.possibly_moved in from_f64_result__f64__Result_T_E_str_i32__pure. HashMap/HashSet doctests fail before exercising collection logic because from_f64_result allocates a six-byte scratch buffer, passes the MemPtr through string_from_mem_unchecked_result, then reads/trims/deallocates via the raw scratch address in a way Resource IR sees as MaybeMoved.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, GitHub Actions run 25157230630 stdlib-test`

## 根拠

- `gh run view 25157230630 --job 73742416567 --log-failed` で `stdlib-test` が `415 total / 232 passed / 173 failed / 10 errored` で失敗していることを確認した。
- `gh run download 25157230630 -n stdlib-tests` で取得した `stdlib-tests.json` では、`stdlib/alloc/collections/hashmap.nepl::doctest#1..#3` と `stdlib/alloc/collections/hashset.nepl::doctest#1..#6` が `from_f64_result__f64__Result_T_E_str_i32__pure` の `resource.cell.possibly_moved` で compile failure になっている。
- 該当実装は `alloc_ptr<u8> 6` で scratch を確保し、`scratch_raw = mem_ptr_addr scratch` を使って digit を書いた後、`string_from_mem_unchecked_result scratch trim` と `dealloc_raw scratch_raw 6` の境界を跨いでいる。

## 問題

GitHub Actions run 25157230630 stdlib-test reports resource.cell.possibly_moved in from_f64_result__f64__Result_T_E_str_i32__pure. HashMap/HashSet doctests fail before exercising collection logic because from_f64_result allocates a six-byte scratch buffer, passes the MemPtr through string_from_mem_unchecked_result, then reads/trims/deallocates via the raw scratch address in a way Resource IR sees as MaybeMoved.

## 影響

The stdlib-test job cannot be used as a clean collection regression signal: string f64 formatting failures mask HashMap/HashSet doctests and can encourage weakening Resource IR cell checks. This is also a regression risk for selfhost diagnostics or JSON/report paths that need numeric formatting.

## 修正方針

Do not weaken Resource IR. Redesign from_f64_result so fractional digit generation does not reuse a raw scratch owner after a conversion boundary. Prefer direct StringBuilder or owned string region construction, or keep scratch storage behind a clear borrowed view and exactly-once free path that Resource IR can prove. Add a source policy/regression that prevents reintroducing scratch_raw MaybeMoved paths in string numeric formatting.

## 検証

Use gh to confirm the affected Actions stdlib-test failure disappears. For implementation pre-commit checks, run focused string/hash collection doctests, node nodesrc/test_stdlib_string_no_unsafe_unwraps.js, node nodesrc/issues.js check, and git diff --check.

## 解決

- `from_f64_result` から 6 byte scratch buffer、`scratch_raw`、`string_from_mem_unchecked_result scratch trim`、`dealloc_raw scratch_raw 6` の所有権境界を削除した。
- 小数 6 桁は局所 `i32` digit として生成し、`from_f64_fraction_trim_len` で末尾 0 を除いた出力桁数を決めるようにした。
- 符号、整数部、小数部の連結は `StringBuilder` の `_result` API に集約し、raw pointer をまたぐ一時文字列化ではなく、builder owner の消費と `sb_build_result` の境界で `str` を確定するようにした。
- `from_f64` の値レベル doctest を追加し、`1`、`1.25`、`-0.5` の固定小数出力を確認するようにした。
- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` に、`from_f64_result` が `scratch_raw` / `alloc_ptr<u8> 6` / `string_from_mem_unchecked_result` を再導入しない source policy を追加した。

## 検証結果

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/from-f64-result-string-nepl-3.json -j 1`: `10 total / 10 passed`
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl --no-tree -o tmp/from-f64-result-hashmap.json -j 1`: `from_f64_result` failure は消え、次の既知残件 `str_split_result` owner may leak が露出した。
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashset.nepl --no-tree -o tmp/from-f64-result-hashset.json -j 1`: `from_f64_result` failure は消え、次の既知残件 `str_split_result` owner may leak が露出した。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: 新規 `alloc/string` policy は passed。既存の `owner_summary_variant_paths.rs has 637 lines; responsibility split limit is 380` は `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` 側の既知残件。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

## 追加確認

HashMap/HashSet doctest は collection 本体の確認前に `str_split_result__str_str__Result_T_E_Vec_T_str_str__pure` の owner may leak へ進む。これは `ISS-20260430T023401649Z-SELFHOST-REQ-FAILS-STRICT-OWNER-GATE-F0FF69D6` で既に追跡済みの `str_split_result` partial owned `Vec<str>` cleanup 問題であり、本 issue では `from_f64_result` の moved-cell 再発防止までを完了とする。
