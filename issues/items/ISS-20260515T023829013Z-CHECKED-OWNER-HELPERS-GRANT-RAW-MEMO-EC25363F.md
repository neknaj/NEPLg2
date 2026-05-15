---
id: ISS-20260515T023829013Z-CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO-EC25363F
title: "Checked owner helpers grant raw memory boundary authority"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/source_capability/raw_memory.rs, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260515T023829013Z-CHECKED-OWNER-HELPERS-GRANT-RAW-MEMO-EC25363F: Checked owner helpers grant raw memory boundary authority

## 概要

compiler-owned stdlib source that only calls checked public owner helpers such as alloc_ptr, alloc_region, or dealloc_region is classified as raw-memory-boundary source evidence.

## 対象

- `nepl-core/src/source_capability/raw_memory.rs, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6。
- raw memory boundary は raw primitive / raw address identity / restricted compiler memory constructor を使う compiler-owned implementation point に限定する必要がある。
- `alloc_ptr` / `alloc_region` / `dealloc_region` は checked public owner wrapper であり、それを使うだけの stdlib source へ raw intrinsic authority を付与してはいけない。

## 問題

compiler-owned stdlib source that only calls checked public owner helpers such as alloc_ptr, alloc_region, or dealloc_region is classified as raw-memory-boundary source evidence.

## 影響

safe allocation wrapper use can accidentally authorize raw intrinsics, raw body memory operations, and restricted raw/owner constructors in the same file, weakening Stage 6 source proof and memory-safety boundaries.

## 修正方針

Do not treat checked owner helper names as raw memory boundary evidence; raw authority must come from actual raw operations, raw address identity helpers, or restricted compiler memory constructors.

## 検証

Add loader regression for alloc_region-only source, update source policy, run focused loader/source capability tests and issue check.

## 解決内容

`source_capability/raw_memory.rs` から `RawOwnerBoundaryHelper` 分類を削除した。`alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `alloc_region` / `alloc_region_bytes` / `dealloc_region` は safe checked wrapper であり、raw boundary evidence にはしない。

raw boundary evidence は、actual raw operation / raw address identity helper / restricted compiler memory constructor / raw address intrinsic に限定する。これにより、compiler-owned stdlib source であっても checked allocation API を使うだけでは raw intrinsic や restricted raw/owner constructor を許可されない。

loader regression は `alloc_ptr<u8> n` だけを使う configured stdlib source が `allows_raw_memory_boundary()` にならないことを確認するよう更新した。source policy も `RawOwnerBoundaryHelper` と `"alloc_region"` literal が raw boundary evidence として残らないことを検査する。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core raw_memory_boundary --lib -- --nocapture`: 12 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/core/mem/types.nepl -i stdlib/core/mem/internal.nepl -i stdlib/core/mem/pointer/alloc.nepl -i stdlib/core/mem/pointer/region.nepl -i stdlib/core/mem/pointer/scalar.nepl -i stdlib/alloc/collections/vec/storage/api.nepl -i stdlib/alloc/collections/vec/storage/view.nepl -i stdlib/alloc/collections/vec/storage/cleanup.nepl --no-tree -o tmp/agent1-raw-boundary-safe-owner-helper-doctests.json -j 1 --dist web/dist --assert-io`: 26 passed
