---
id: ISS-20260428T120507528Z-RESOURCE-BORROW-CHECKER-IGNORES-OVER-E9E862D3
title: "Resource borrow checker ignores overlapping place projections"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T120507528Z-RESOURCE-BORROW-CHECKER-IGNORES-OVER-E9E862D3: Resource borrow checker ignores overlapping place projections

## 概要

ResourceBorrowCheckEngine checks borrow state only for the exact place. A shared or unique borrow of an aggregate field does not block assigning, moving, or uniquely borrowing the aggregate root, and a borrow of the aggregate root does not block field mutation.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は `Place` と projection を Resource IR 上の borrow/lifetime 検査対象にする方針である。
- `BorrowTable::state` は exact place のみを検索しており、`wrapper.field` の borrow と `wrapper` の assign/move/drop を overlapping place として扱っていなかった。
- 逆に aggregate root の borrow と field mutation も exact place が異なるため見落とされる。
- Resource IR の `PlaceProjection` は prefix 構造を持つため、同一 root で片方の projection がもう片方の prefix であれば overlap と判定できる。

## 問題

ResourceBorrowCheckEngine checks borrow state only for the exact place. A shared or unique borrow of an aggregate field does not block assigning, moving, or uniquely borrowing the aggregate root, and a borrow of the aggregate root does not block field mutation.

## 影響

Borrow/lifetime checks can be bypassed by moving between aggregate and field projections. This weakens memory safety for structs, tuples, enum payloads, and self-host lowering that uses projected places.

## 修正方針

Teach BorrowTable to find active borrow states on overlapping places using projection-prefix overlap. Use that in borrow creation, read checks, and exclusive operations such as move/assign/drop.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 borrow projection overlap 対応

`BorrowTable` に active borrow state の overlapping projection 検索を追加した。同一 root で projection prefix が重なる place は同じ memory location の重なりとして扱う。

borrow 作成、read、assign / move / drop などの exclusive operation は exact place だけでなく overlapping active borrow を参照するようにした。これにより field borrow 中の aggregate assign や、aggregate unique borrow 中の field read などを Resource IR 上で検出できる。

`nepl-core/tests/resource_ir.rs` に、struct field を shared borrow した状態で aggregate root を assign する経路が `BorrowConflict(Assign)` になる回帰を追加した。
