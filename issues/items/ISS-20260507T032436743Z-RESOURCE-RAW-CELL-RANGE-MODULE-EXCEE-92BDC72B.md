---
id: ISS-20260507T032436743Z-RESOURCE-RAW-CELL-RANGE-MODULE-EXCEE-92BDC72B
title: "Resource raw cell range module exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/cell_state_raw_range.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T032436743Z-RESOURCE-RAW-CELL-RANGE-MODULE-EXCEE-92BDC72B: Resource raw cell range module exceeds responsibility split limit

## 概要

After syncing origin/main 18768838, source policy reports cell_state_raw_range.rs has 159 lines while the responsibility split limit is 140. The returned byte range summary change concentrated raw byte range cell-state logic in a module that is supposed to stay small and audit-focused.

## 対象

- `nepl-core/src/resource/cell_state_raw_range.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `origin/main` `18768838` を取り込んだ後、`node nodesrc/run_source_policy_regressions.js --warn-only` が `nodesrc/test_resource_checker_responsibility.js` で検出した。
- 失敗内容は `cell_state_raw_range.rs has 159 lines; responsibility split limit is 140`。
- `cell_state_raw_range.rs` は returned byte range summary の安全性を支える Resource IR の cell-state proof module であり、line limit を上げて隠すのではなく責務分割で解消する必要がある。

## 問題

After syncing origin/main 18768838, source policy reports cell_state_raw_range.rs has 159 lines while the responsibility split limit is 140. The returned byte range summary change concentrated raw byte range cell-state logic in a module that is supposed to stay small and audit-focused.

## 影響

Raw cell range availability is part of initialized-cell proof propagation. Letting this module grow past the responsibility boundary makes Resource IR memory-safety checks harder to audit and can hide future raw range regressions behind broad helper code.

## 修正方針

Split cell_state_raw_range.rs by range normalization, availability checks, and returned-byte-range summary application without raising the responsibility split limit.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only after the split, plus focused Resource IR tests for returned byte range summaries.
