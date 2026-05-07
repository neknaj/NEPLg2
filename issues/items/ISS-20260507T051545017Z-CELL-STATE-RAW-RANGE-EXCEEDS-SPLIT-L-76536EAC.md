---
id: ISS-20260507T051545017Z-CELL-STATE-RAW-RANGE-EXCEEDS-SPLIT-L-76536EAC
title: "cell_state_raw_range exceeds split limit again after realloc range transfer"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/cell_state_raw_range.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T051545017Z-CELL-STATE-RAW-RANGE-EXCEEDS-SPLIT-L-76536EAC: cell_state_raw_range exceeds split limit again after realloc range transfer

## 概要

After rebasing on origin/main f425f799, source policy warns that cell_state_raw_range.rs has 144 lines while the responsibility split limit is 140. The prior raw range module split issue is fixed, but the realloc initialized range transfer change added enough logic to exceed the boundary again.

## 対象

- `nepl-core/src/resource/cell_state_raw_range.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After rebasing on origin/main f425f799, source policy warns that cell_state_raw_range.rs has 144 lines while the responsibility split limit is 140. The prior raw range module split issue is fixed, but the realloc initialized range transfer change added enough logic to exceed the boundary again.

## 影響

Resource IR raw range proof code is memory-safety critical. Letting the central mutation module grow past its limit again weakens reviewability and can hide future initialized range regressions behind a broad CellTable helper file.

## 修正方針

Split the new realloc/range transfer responsibility out of cell_state_raw_range.rs or move a stable subset of raw range mutation into a narrower module. Keep the 140-line policy instead of raising the limit.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only without warnings.
