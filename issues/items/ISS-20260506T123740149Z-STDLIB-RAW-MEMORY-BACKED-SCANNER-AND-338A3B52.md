---
id: ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52
title: "stdlib raw-memory-backed scanner and byte helpers lack Stage5 boundary plan"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/io.nepl, stdlib/alloc/string/utf8.nepl, stdlib/std/text.nepl, stdlib/std/streamio/scanner/state.nepl, tests/stdlib/kp.n.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260506T123740149Z-STDLIB-RAW-MEMORY-BACKED-SCANNER-AND-338A3B52: stdlib raw-memory-backed scanner and byte helpers lack Stage5 boundary plan

## 概要

After UnsafeMemoryInPureFunction became a hard Resource IR gate, tests/stdlib/kp.n.md still failed in wasm runner because bytebuf/text/scanner helpers used raw load/store/bulk_copy through pure APIs without a compiler-owned boundary or Stage6 safe wrapper migration.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/io.nepl, stdlib/alloc/string/utf8.nepl, stdlib/std/text.nepl, stdlib/std/streamio/scanner/state.nepl, tests/stdlib/kp.n.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 は、raw memory を public pure surface から閉じ、compiler-owned raw-memory-boundary source だけを Stage 6 移行中許可にする方針である。
- `trunk build` 後の `tests/stdlib/kp.n.md` wasm runner は `alloc/io.nepl` の `io_bytebuf_from_str_result`、`alloc/string/utf8.nepl` の `string_utf8_byte_at`、`std/text.nepl` の `text_utf8_byte_at`、`std/streamio/scanner/state.nepl` の scanner header / byte access を `effect.pure.calls_impure` で拒否していた。
- これらは単なる diagnostics の表示問題ではない。raw-memory-backed stdlib helper を public pure API として残すか、internal boundary と safe wrapper に分離するかを Stage 5/6 の設計として決める必要がある。
- stdlib 全体や path suffix を一括許可すると user source から raw memory gate を迂回できるため不可。configured stdlib の exact internal boundary だけを compiler が知る設計か、stdlib API 移行で raw operation を public surface から消す設計にする。

## 問題

After UnsafeMemoryInPureFunction became a hard Resource IR gate, tests/stdlib/kp.n.md still failed in wasm runner because bytebuf/text/scanner helpers used raw load/store/bulk_copy through pure APIs without a compiler-owned boundary or Stage6 safe wrapper migration.

## 影響

Doctests that import stream scanner or byte buffer APIs cannot compile, and the returned range summary regression remains hidden behind additional effect diagnostics.

## 修正方針

Design the exact internal/public boundary for these raw-memory-backed stdlib helpers: either add audited exact raw-memory-boundary capabilities for true internal modules or migrate public APIs to effect-aware safe wrappers without allowing arbitrary stdlib or suffix paths.

## 対応

- raw-memory-boundary path を `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` の explicit table に分離し、configured stdlib root から導出した exact path だけを許可するようにした。
- `alloc/io.nepl`、`alloc/string/utf8.nepl`、`std/text.nepl`、`std/streamio/scanner/state.nepl` を Stage 6 移行中の audited boundary として追加した。
- 既存の `core/mem.nepl`、`alloc/string.nepl`、`alloc/string/storage.nepl` も同じ table に保持し、stdlib 全体や arbitrary suffix path の許可にはしていない。
- loader regression は byte/scanner boundary module ごとに raw `i32.store` を含む configured stdlib import を通し、user 側の suffix path rejection は既存 test で維持した。
- `tests/stdlib/kp.n.md` の wasm runner では `effect.pure.calls_impure` が消え、残りは owner leak、dynamic range summary、float runtime timeout へ進んだ。これらは `ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F`、`ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38`、`ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8` で追跡する。

## 検証

- `cargo fmt --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test effects loader_ -- --nocapture`: passed, 5 tests
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_stage5_raw_boundary.json --runner wasm --no-tree -j 1 --assert-io`: `effect.pure.calls_impure` は解消。残りは別 issue に分離。
