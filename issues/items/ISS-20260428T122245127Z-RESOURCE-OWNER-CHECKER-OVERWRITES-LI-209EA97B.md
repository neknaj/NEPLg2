---
id: ISS-20260428T122245127Z-RESOURCE-OWNER-CHECKER-OVERWRITES-LI-209EA97B
title: "Resource owner checker overwrites live owners on assignment"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T122245127Z-RESOURCE-OWNER-CHECKER-OVERWRITES-LI-209EA97B: Resource owner checker overwrites live owners on assignment

## 概要

ResourceOwnerCheckEngine handles ResourceOp::Assign by transferring the value owner into the target, but it does not account for an existing live owner already stored in the target or its projections. OwnerTable::set_state can overwrite that live free obligation.

## 対象

- `nepl-core/src/resource/check.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は owner token / free obligation を Resource IR の `Place` に結びつけ、value movement と obligation movement を一致させる方針である。
- `ResourceOwnerCheckEngine` の `Assign` は `transfer_owner(value, target)` を呼ぶだけで、target 側に既存 live owner があるかを事前に見ていなかった。
- `OwnerTable::set_state` は同じ `Place` の state を更新するため、target exact owner や target descendant owner projection が live のまま新しい owner を代入すると、古い storage id が owner table から消える。
- aggregate owner projection transfer の修正により `wrapper.field` の obligation は追えるようになったが、assign overwrite 時の古い `wrapper.field` obligation を診断しなければ、projection 対応が逆に leak を隠す経路になる。

## 問題

ResourceOwnerCheckEngine handles ResourceOp::Assign by transferring the value owner into the target, but it does not account for an existing live owner already stored in the target or its projections. OwnerTable::set_state can overwrite that live free obligation.

## 影響

Self-host or stdlib code can assign a new owning pointer or owning aggregate over an old one without deallocating or dropping the old storage. The checker then loses the old free obligation and misses a leak.

## 修正方針

Before assignment transfer, detect live owner obligations at the target and descendant projections that are not the value being transferred, report them as leaked or maybe leaked, and clear those stale obligations before writing the new owner state.

## 検証

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `trunk build`
- `node nodesrc/issues.js check`
- `rustfmt --check nepl-core\src\resource\check.rs nepl-core\tests\resource_ir.rs`
- `git diff --check`

## 2026-04-28 Stage 4 owner assignment overwrite 対応

`ResourceOwnerCheckEngine` の `Assign` で owner transfer を実行する前に、target exact place と descendant projection 配下の live / maybe-freed owner を列挙するようにした。

代入 value と overlap しない既存 owner は、代入によって free obligation が失われるため `OwnerLeaked` / `OwnerMaybeLeaked` としてその場で診断し、stale state は `Moved` にする。これにより新しい owner を target へ移しても、古い storage id が `OwnerTable::set_state` によって無言で上書きされない。

`nepl-core/tests/resource_ir.rs` に、raw pointer local へ新しい owner を assign してから dealloc しても古い owner leak が残る回帰と、owner 入り aggregate を別 aggregate で assign したときに旧 field owner leak が報告される回帰を追加した。
