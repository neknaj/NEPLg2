---
id: ISS-20260428T114942018Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-B551A456
title: "Resource owner checker does not move aggregate owner projections"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T114942018Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-B551A456: Resource owner checker does not move aggregate owner projections

## 概要

After owner inputs are moved into constructed aggregate projections, ResourceOwnerCheckEngine still transfers only exact owner places. Moving, assigning, or branch-returning the aggregate value does not move owner entries stored under its field projections, leaving the owner attached to the old aggregate path.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T114547680Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-0F2678E0` で construct input の owner は aggregate output projection へ移るようになった。
- しかし `ResourceOwnerCheckEngine::transfer_owner` は exact place の owner だけを移しており、aggregate value 自体の `Move` / `Assign` / branch value transfer では field projection 配下の owner が旧 aggregate path に残っていた。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は owner state を Resource IR の `Place` と projection に紐付け、value movement と owner movement を一致させる必要がある。
- `Place` は projection list を持っているため、source prefix 配下の owner entry を target prefix 配下へ移せる。

## 問題

After owner inputs are moved into constructed aggregate projections, ResourceOwnerCheckEngine still transfers only exact owner places. Moving, assigning, or branch-returning the aggregate value does not move owner entries stored under its field projections, leaving the owner attached to the old aggregate path.

## 影響

Aggregate owner state can become detached from value movement. Code can move an aggregate and then deallocate or move the old aggregate field path, while the new aggregate value is not treated as carrying the obligation.

## 修正方針

Teach owner transfer to move live owner descendants under a source place to the corresponding target projection path. Report unavailable descendant owners when aggregate transfer touches moved or freed owner fields. Add a regression for moving a constructed aggregate and rejecting deallocation through the old field path.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 aggregate owner projection move 対応

`OwnerTable` に source prefix 配下の owner entry を列挙する処理を追加し、`ResourceOwnerCheckEngine::transfer_owner` が exact owner だけでなく descendant owner projection も target prefix 配下へ移すようにした。

これにより aggregate value を `Move` / `Assign` / branch value として移した場合、構築時に aggregate field projection へ入った owner obligation も同じ value movement に追従する。旧 aggregate field path の owner は `Moved` になり、再 dealloc / move は `OwnerUnavailable` になる。

`nepl-core/tests/resource_ir.rs` に、owner を入れた struct を move した後で旧 field path を dealloc しようとする経路の回帰を追加した。
