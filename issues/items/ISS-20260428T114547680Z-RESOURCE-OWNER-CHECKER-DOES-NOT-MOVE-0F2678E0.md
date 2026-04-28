---
id: ISS-20260428T114547680Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-0F2678E0
title: "Resource owner checker does not move owners into constructed aggregates"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T114547680Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-0F2678E0: Resource owner checker does not move owners into constructed aggregates

## 概要

ResourceOwnerCheckEngine ignores ResourceOp::Construct. When a live owner is used as an aggregate input, the owner remains attached to the original input place instead of moving into an output projection. The original owner can then be deallocated or moved again after construction.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は `OwnerState` を Resource IR の `Place` に結びつけ、aggregate construction でも owner token / free obligation が落ちないことを要求している。
- `ResourceOwnerCheckEngine` は `DeclareLocal` / `Read` / `Move` / call return では owner を移しているが、`ResourceOp::Construct` を無視していた。
- そのため `alloc -> struct { ptr }` のような構築後も owner が入力 place に残り、構築済み aggregate と元 pointer の両方が同じ obligation を持つかのように扱われた。
- Resource IR には `AggregateKind` と `PlaceProjection` があるため、構築された output の field / tuple field / enum payload projection へ owner を移せる。

## 問題

ResourceOwnerCheckEngine ignores ResourceOp::Construct. When a live owner is used as an aggregate input, the owner remains attached to the original input place instead of moving into an output projection. The original owner can then be deallocated or moved again after construction.

## 影響

Owner/free-obligation checking is not tied to Resource IR aggregate construction. This permits owner duplication patterns through structs, tuples, or enum payloads and keeps Stage 4 owner checks dependent on flat pointer shapes.

## 修正方針

Handle ResourceOp::Construct in the owner checker by transferring each live owner input into a deterministic output projection for the constructed aggregate. Add a regression that deallocating the original input after construction is rejected as a moved owner.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 aggregate construct owner transfer 対応

`ResourceOwnerCheckEngine` が `ResourceOp::Construct` を処理し、各 input の live owner を構築 output の deterministic projection へ移すようにした。struct は `Field`、tuple は `TupleField`、enum は `EnumPayload` を使う。

これにより aggregate へ移した owner の元 input は `Moved` になり、構築後に元 pointer を `Dealloc` / move しようとする経路を `OwnerUnavailable` として検出できる。

`nepl-core/tests/resource_ir.rs` に、raw allocation owner を struct に構築した後で元 pointer を dealloc しようとする経路の回帰を追加した。
