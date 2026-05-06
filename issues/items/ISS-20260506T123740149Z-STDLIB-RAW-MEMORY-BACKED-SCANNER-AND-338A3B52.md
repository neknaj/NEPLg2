---
id: ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52
title: "stdlib raw-memory-backed scanner and byte helpers lack Stage5 boundary plan"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/io.nepl, stdlib/alloc/string/utf8.nepl, stdlib/std/text.nepl, stdlib/std/streamio/scanner/state.nepl, tests/stdlib/kp.n.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52: stdlib raw-memory-backed scanner and byte helpers lack Stage5 boundary plan

## 概要

After UnsafeMemoryInPureFunction became a hard Resource IR gate, tests/stdlib/kp.n.md still fails in wasm runner because bytebuf/text/scanner helpers use raw load/store/bulk_copy through pure APIs without a compiler-owned boundary or Stage6 safe wrapper migration.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/io.nepl, stdlib/alloc/string/utf8.nepl, stdlib/std/text.nepl, stdlib/std/streamio/scanner/state.nepl, tests/stdlib/kp.n.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 は、raw memory を public pure surface から閉じ、compiler-owned raw-memory-boundary source だけを Stage 6 移行中許可にする方針である。
- `trunk build` 後の `tests/stdlib/kp.n.md` wasm runner は `alloc/io.nepl` の `io_bytebuf_from_str_result`、`alloc/string/utf8.nepl` の `string_utf8_byte_at`、`std/text.nepl` の `text_utf8_byte_at`、`std/streamio/scanner/state.nepl` の scanner header / byte access を `effect.pure.calls_impure` で拒否した。
- これらは単なる diagnostics の表示問題ではない。raw-memory-backed stdlib helper を public pure API として残すか、internal boundary と safe wrapper に分離するかを Stage 5/6 の設計として決める必要がある。
- stdlib 全体や path suffix を一括許可すると user source から raw memory gate を迂回できるため不可。configured stdlib の exact internal boundary だけを compiler が知る設計か、stdlib API 移行で raw operation を public surface から消す設計にする。

## 問題

After UnsafeMemoryInPureFunction became a hard Resource IR gate, tests/stdlib/kp.n.md still fails in wasm runner because bytebuf/text/scanner helpers use raw load/store/bulk_copy through pure APIs without a compiler-owned boundary or Stage6 safe wrapper migration.

## 影響

Doctests that import stream scanner or byte buffer APIs cannot compile, and the returned range summary regression remains hidden behind additional effect diagnostics.

## 修正方針

Design the exact internal/public boundary for these raw-memory-backed stdlib helpers: either add audited exact raw-memory-boundary capabilities for true internal modules or migrate public APIs to effect-aware safe wrappers without allowing arbitrary stdlib or suffix paths.

## 検証

- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_stage5_raw_boundary.json --runner wasm --no-tree -j 1 --assert-io`
