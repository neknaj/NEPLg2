---
id: ISS-20260507T155231568Z-SELFHOST-HIR-RANGES-ENCODE-EMPTY-STA-8B562D49
title: "Selfhost HIR ranges encode empty state with negative first index"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_range_payload.js"
---

# ISS-20260507T155231568Z-SELFHOST-HIR-RANGES-ENCODE-EMPTY-STA-8B562D49: Selfhost HIR ranges encode empty state with negative first index

## 概要

SelfhostHirChildRange and SelfhostHirParamRange store first/count as flat i32 pairs. Empty ranges are represented as (-1, 0), and callers then test negative first indexes to avoid table access.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl`
- `nodesrc/test_selfhost_hir_range_payload.js`

## 根拠

- `SelfhostHirChildRange` と `SelfhostHirParamRange` が flat `first/count` pair で、空範囲を `(-1, 0)` として表していた。
- `selfhost_hir_module_get_child` / `selfhost_hir_module_get_param` は negative first index を後段で検査し、payload の有無を型ではなく値で判定していた。
- HIR lowering と静的検査が child/parameter range を共有するため、空範囲と nonempty range の区別は enum variant として表す必要がある。

## 問題

SelfhostHirChildRange and SelfhostHirParamRange store first/count as flat i32 pairs. Empty ranges are represented as (-1, 0), and callers then test negative first indexes to avoid table access.

## 影響

HIR lowering and later static checks can accidentally propagate or read negative range starts as ordinary payload. Empty-vs-nonempty range handling is not forced through enum match coverage, leaving the self-host HIR model with invalid sentinel state.

## 修正方針

Split child and parameter ranges into Empty and Range enum payloads. Range payloads carry first/count only for nonempty ranges, and all accessors/table lookups must match the range variant before reading payload.

## 検証

Add a source policy rejecting flat HIR range structs, -1 empty range constructors, direct range field access in table lookups, and missing Empty/Range match handling. Run focused HIR doctests, issue check, and source policy regressions.

## 対応結果

- `SelfhostHirChildRange` を `Empty` / `Range <SelfhostHirChildRangeItems>` の enum payload に変更した。
- `SelfhostHirParamRange` を `Empty` / `Range <SelfhostHirParamRangeItems>` の enum payload に変更した。
- `selfhost_hir_child_range_empty` / `selfhost_hir_param_range_empty` は `-1` sentinel ではなく `Empty` variant を返す。
- `selfhost_hir_child_range_first` / `count` と `selfhost_hir_param_range_first` / `count` は range variant を `match` して値を返す。
- `selfhost_hir_module_get_child` / `get_param` は `Empty` を即 `None` にし、`Range` payload だけを table lookup に使う。
- `nodesrc/test_selfhost_hir_range_payload.js` を追加し、flat range struct、negative empty range、direct range field read、Empty/Range match 不足の再導入を source policy で拒否する。

## 検証結果

- `node nodesrc/test_selfhost_hir_range_payload.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/hir/hir.nepl --no-tree -o tmp/agent1-selfhost-hir-range-payload.json -j 1 --dist web/dist`: total=3, passed=3
