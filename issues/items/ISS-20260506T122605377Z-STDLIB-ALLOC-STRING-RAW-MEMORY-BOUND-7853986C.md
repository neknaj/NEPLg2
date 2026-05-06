---
id: ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C
title: "stdlib alloc/string raw memory boundary lacks loader capability"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, nepl-core/tests/effects.rs, stdlib/alloc/string.nepl, stdlib/alloc/string/storage.nepl, nepl-core/tests/kp.rs, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260506T122605377Z-STDLIB-ALLOC-STRING-RAW-MEMORY-BOUND-7853986C: stdlib alloc/string raw memory boundary lacks loader capability

## 概要

Stage 5 Resource IR effect gate rejects raw-memory-backed pure functions in configured stdlib alloc/string.nepl and its storage submodule because Loader grants raw_memory_boundary capability only to core/mem.nepl.

## 対象

- `nepl-core/src/loader.rs, nepl-core/tests/effects.rs, stdlib/alloc/string.nepl, stdlib/alloc/string/storage.nepl, nepl-core/tests/kp.rs, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 では、`UnsafeMemoryInPureFunction` を Resource IR gate から compiler error へ接続しつつ、compiler-owned raw-memory-boundary capability を持つ source だけを Stage 6 移行中許可として扱う方針になっている。
- `Loader::source_capabilities_for_path` は configured stdlib の `core/mem.nepl` にだけ `raw_memory_boundary` を付与していた。
- configured stdlib の `alloc/string.nepl` と `alloc/string/storage.nepl` は string / str の owned storage boundary として raw `load` / `store` / `bulk_copy` を内部で使うが、loader capability がないため `concat_result` / `from_u128_radix` / `len__str` / `string_finish_base` が `effect.pure.calls_impure` で停止していた。
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture` は、この問題の修正前に returned raw header range summary の検証地点へ到達できなかった。

## 問題

Stage 5 Resource IR effect gate rejects raw-memory-backed pure functions in configured stdlib alloc/string.nepl and its storage submodule because Loader grants raw_memory_boundary capability only to core/mem.nepl.

## 影響

Full scanner style regressions stop at PureCallsImpure before returned raw header range summaries can be verified. This also treats audited string ownership boundary helpers as public untrusted raw memory use.

## 修正方針

Grant raw_memory_boundary capability to the exact configured stdlib alloc/string.nepl and alloc/string/storage.nepl paths, keep suffix-only custom paths rejected, and cover the loader behavior with regression tests.

## 対応

- `Loader` の raw-memory boundary 判定を `configured_raw_memory_boundary_path` に集約し、configured stdlib root から導出した `core/mem.nepl`、`alloc/string.nepl`、`alloc/string/storage.nepl` の canonical path だけを許可した。
- arbitrary path suffix による `alloc/string.nepl` 偽装は許可しない。capability は loader の `stdlib_root` から計算する exact path にだけ付与される。
- configured stdlib の `alloc/string.nepl` と `alloc/string/storage.nepl` が pure helper 内で raw memory operation を使っても、Resource IR effect gate が audited boundary として許可する regression を追加した。
- configured string boundary 自体の Stage 5 blocker は解消した。`tests/stdlib/kp.n.md` の wasm doctest では `alloc/io.nepl` / `alloc/string/utf8.nepl` / `std/text.nepl` / `std/streamio/scanner/state.nepl` の追加 boundary 設計漏れも見つかったため、`ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` として分離した。

## 検証

- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_alloc_string_as_raw_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_alloc_string_storage_as_raw_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_alloc_string_raw_boundary.json --runner wasm --no-tree -j 1 --assert-io`: failed, separated to `ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52` and `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38`
