---
id: ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E
title: "Resource IR RawMemoryLoadCell gate needs raw pointer summaries"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/compiler.rs"
---

# ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E: Resource IR RawMemoryLoadCell gate needs raw pointer summaries

## 概要

RawMemoryLoadCell diagnostics cannot be made authoritative yet because Resource IR CellState still loses raw cell initialization across pointer-returning helpers, MemPtr address wrappers, and projection-based raw field access. Enabling the full load-cell gate produces false D3100 before the intended D3025/raw ownership diagnostics.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/compiler.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は CellState による initialized / moved 検査を Resource IR 上へ移す計画である。
- 2026-04-29 の destructive raw cell gate 作業中に `RawMemoryLoadCell` も compiler error 化すると、`tests/compiler/move_effect.n.md` の helper-returned slot、`MemPtr` wrapper、projection field access で false D3100 が出た。
- `RawMemoryStoreCell` / `RawMemoryDeallocCell` / `RawMemoryReallocCell` / `RawMemoryFillCell` / bulk cell は focused regression を通過したが、load cell は raw pointer summary が不足しているため別の完了条件を必要とする。

## 問題

RawMemoryLoadCell diagnostics cannot be made authoritative yet because Resource IR CellState still loses raw cell initialization across pointer-returning helpers, MemPtr address wrappers, and projection-based raw field access. Enabling the full load-cell gate produces false D3100 before the intended D3025/raw ownership diagnostics.

## 影響

Stage 4 cannot fully replace old move_check for raw load before store or repeated non-Copy raw load. Keeping RawMemoryLoadCell shadow-only is necessary until function and projection summaries preserve the pointed cell state without false positives.

## 修正方針

Add Resource IR summaries for raw pointer parameter-to-return transfer, MemPtr/RegionToken address wrappers, and raw aggregate field projection loads, then enable RawMemoryLoadCell diagnostics in the compiler gate after focused move_effect and move_check regressions pass.

## 検証

Add Resource IR unit tests for helper-returned raw slot load, MemPtr wrapper raw slot load, and projection field raw load. Run cargo test -p nepl-core --test resource_ir, trunk build, and node nodesrc/tests.js for tests/compiler/move_effect.n.md and tests/compiler/move_check.n.md.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [RV-CORE-009 Resource IR parent](./ISS-20260425T000000Z-RV-CORE-009-58589A3F.md)

## 2026-04-29 調査追記

`RawMemoryLoadCell` gate を一時的に compiler error 化して `tests/compiler/move_effect.n.md` を実行したところ、最初の大きな false positive 群は Resource IR lowering ではなく CellState checker 側の二重消費だった。direct raw memory helper は Resource IR 上で `ResourceOp::Call` と `ResourceOp::RawMemory` の両方を持つため、generic `Call` が先に store value を moved にし、直後の `RawMemory::Store` が pointed cell を initialized にできていなかった。

この前提不備は `ISS-20260428T173213551Z-RESOURCE-CELLSTATE-CHECKER-CONSUMES--DD20A3D7` として分離し、修正済み。修正後に同じ一時 gate 調査を再実行すると、`move_effect.n.md` の失敗は 18 件から 11 件へ減った。残件は helper-returned slot の direct/indirect return summary、`MemPtr` / `RegionToken` の address wrapper、raw aggregate field load の offset/projection CellState に絞られる。

2026-04-29 追記:

`ISS-20260428T174427199Z-RESOURCE-CELLSTATE-RAW-ADDRESS-ALIAS-45DC270E` として、CellState 側の raw address alias が helper return、known function-value indirect call、aggregate field に渡らない問題を分離し、修正済みにした。direct helper / function-value helper / `PtrBox.ptr` style wrapper の Resource IR regression は `RawMemoryLoadCell` 診断なしで通る。

この修正で helper-returned slot と `MemPtr` / `RegionToken` style wrapper の主要な alias loss は CellState に移った。親 issue には、raw memory へ格納した aggregate の field offset/projection と、compiler field projection load と raw linear memory load の区別不足が残る。

2026-04-29 追記 2:

`ISS-20260428T175617166Z-RESOURCE-CELLSTATE-EXPRESSION-MARKER-26479BD3` として、HIR lowering が semantic `ResourceOp` の直後に出す `ResourceExprKind` marker で raw address alias を消していた問題を修正した。temporary `RawMemoryLoadCell` gate で `move_effect.n.md` は 99/110 から 101/110 へ改善し、doctest#13/#14 の helper-returned slot false D3100 は解消した。

残件は主に `MemPtr` / `RegionToken` address wrapper の raw cell initialization transfer、raw aggregate stored value の field offset/projection、通常 aggregate field projection load と raw linear memory load の分類不足である。

2026-04-29 追記 3:

`ISS-20260428T182913710Z-RESOURCE-IR-LOWERING-MISCLASSIFIES-C-A1508C51` として、通常 aggregate field access の compiler-generated `load` を Resource IR で raw memory load と誤分類していた問題を修正した。Resource IR lowering と coverage comparison の両方に `TypeCtx` 付き分類を追加し、aggregate pseudo-address からの field read は `PlaceProjection::Field` / `TupleField` の `ResourceOp::Read` へ下げる。

一時 `RawMemoryLoadCell` gate では `move_check.n.md` が 51/52 から 52/52 に改善し、`move_effect.n.md` は 101/110 から 104/110 に改善した。通常 Copy aggregate field read の false D3100 / D3101 は解消済みである。残件は 6 件で、`MemPtr` / `RegionToken` address wrapper の raw cell initialization transfer と raw memory に格納した aggregate の field offset/projection に絞られる。
