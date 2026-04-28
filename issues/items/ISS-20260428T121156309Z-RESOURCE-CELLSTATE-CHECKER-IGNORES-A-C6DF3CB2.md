---
id: ISS-20260428T121156309Z-RESOURCE-CELLSTATE-CHECKER-IGNORES-A-C6DF3CB2
title: "Resource CellState checker ignores aggregate projection state"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T121156309Z-RESOURCE-CELLSTATE-CHECKER-IGNORES-A-C6DF3CB2: Resource CellState checker ignores aggregate projection state

## 概要

Resource initialized/moved checking reads CellTable by exact place only. After an aggregate is initialized, moving or dropping a field projection does not affect the aggregate root, and moving the aggregate root does not affect descendant projections.

## 対象

- `nepl-core/src/resource/check.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は `CellState` と `Place` による initialized / moved state の検査を Resource IR 上で行う方針である。
- `CellTable::state` は exact place のみを検索していたため、`wrapper` が initialized でも `wrapper.field` は uninitialized と扱われ、逆に `wrapper.field` を move しても `wrapper` は initialized のままだった。
- `BorrowTable` / `OwnerTable` 側は projection prefix を扱い始めており、cell state だけが aggregate root と descendant projection を別々の memory region のように扱うと Resource IR の安全性が一貫しない。
- `PlaceProjection` は prefix 構造を持つため、ancestor の non-initialized state は descendant を使用不能にし、descendant の non-initialized state は aggregate root を部分 move/drop 済みとして扱える。

## 問題

Resource initialized/moved checking reads CellTable by exact place only. After an aggregate is initialized, moving or dropping a field projection does not affect the aggregate root, and moving the aggregate root does not affect descendant projections.

## 影響

Self-host and stdlib code can use or return an aggregate after a non-Copy field has been moved, or read a field after the aggregate has been moved. This weakens Stage 4 Resource IR memory-safety checks.

## 修正方針

Make CellTable availability projection-aware: ancestor and descendant non-initialized states must make overlapping places unavailable, while initialized aggregate ancestors can initialize projections unless an explicit projection state overrides them.

## 検証

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `trunk build`
- `node nodesrc/issues.js check`
- `rustfmt --check nepl-core\src\resource\check.rs nepl-core\tests\resource_ir.rs`
- `git diff --check`

## 2026-04-28 Stage 4 cell projection state 対応

`CellTable::availability_state` を追加し、exact place だけでなく ancestor / descendant projection の状態を含めて initialized / moved / dropped / maybe moved を判定するようにした。

ancestor が initialized であれば descendant projection は initialized として扱う。ただし ancestor または descendant に `Moved` / `Dropped` / `MaybeMoved` / `Uninit` が明示されている場合は、その overlapping place の使用を拒否する。aggregate root を再初期化した場合は descendant の古い projection state を消すため、過去の field move が新しい aggregate value へ漏れない。

`merge_paths` も exact state ではなく projection-aware availability を合流するようにした。これにより、一方の branch で field を明示的に初期化し、もう一方では aggregate ancestor の initialized state に依存する場合でも不必要に `MaybeMoved` へ落とさない。

`nepl-core/tests/resource_ir.rs` に、constructed aggregate の field read 正常系、field move 後の aggregate return 拒否、aggregate move 後の旧 field read 拒否を追加した。
