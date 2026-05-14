---
id: ISS-20260514T150630865Z-RESOURCE-OWNER-RAW-VIEW-POLICY-DRIFT-6883F812
title: "Resource owner raw view policy drift after transfer kind split"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-15
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/owner_summary_raw_transfer.rs"
---

# ISS-20260514T150630865Z-RESOURCE-OWNER-RAW-VIEW-POLICY-DRIFT-6883F812: Resource owner raw view policy drift after transfer kind split

## 概要

Resource checker source policy still requires the old raw_address_view_carries_owner_alias helper even though owner_summary_raw_transfer.rs now classifies raw view owner transfer through RawOwnerAliasTransferKind. The policy fails and no longer monitors the richer NonOwningProjection/NonOwning/OwnerAlias branches.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/owner_summary_raw_transfer.rs`

## 根拠

- `nodesrc/test_resource_checker_responsibility.js` が `owner_summary_raw_transfer.rs` の旧 helper `raw_address_view_carries_owner_alias` を要求して失敗していた。
- 現行実装は `RawOwnerAliasTransferKind` により、`NonOwningProjection`、既存 non-owning view からの `Offset`、owner-carrying `Offset` を分けている。
- 同じ policy 実行で、後続 Resource IR module の追加後に `summary_index.rs` などの監視漏れと、owner summary / worklist / i32 condition helper の責務集中も露出した。

## 問題

Resource checker source policy still requires the old raw_address_view_carries_owner_alias helper even though owner_summary_raw_transfer.rs now classifies raw view owner transfer through RawOwnerAliasTransferKind. The policy fails and no longer monitors the richer NonOwningProjection/NonOwning/OwnerAlias branches.

## 影響

A future Resource IR refactor could weaken the MemPtr non-owning pointer and owner-token separation without the source policy catching the exact enum branch that changed.

## 修正方針

Update the source policy to require raw_owner_alias_transfer_kind and the RawOwnerAliasTransferKind match branches, and add focused Rust tests for offset-from-non-owning transfer behavior.

## 検証

node nodesrc/test_resource_checker_responsibility.js; cargo test -p nepl-core owner_summary_raw_transfer -- --nocapture; node nodesrc/issues.js check --dir issues

## 2026-05-15 修正

- raw view owner alias policy を旧 `raw_address_view_carries_owner_alias` 監視から、現行の `raw_owner_alias_transfer_kind` / `RawOwnerAliasTransferKind` 監視へ更新した。
- `NonOwningProjection` は projection view として伝播し、`NonOwning` 由来の offset は non-owning view として伝播し、owner-carrying `Offset` だけが raw owner alias を転送することを source policy と Rust test の両方で固定した。
- `owner_summary_raw_alias` / `owner_summary_raw_use` の recursive walk、`owner_summary_update` の canonicalization、`summary_worklist` の初期順序、`initialized_alias_i32_condition` の query context / relation condition を責務ごとに分割した。
- `summary_index.rs` と今回追加した helper / test module を resource responsibility line limit の監視対象へ追加し、未監視 module が残らないことを policy で確認した。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --lib owner_summary -- --nocapture`: 6/6 pass
- `cargo test -p nepl-core --lib summary_worklist -- --nocapture`: 1/1 pass
- `cargo test -p nepl-core --lib initialized_alias_i32 -- --nocapture`: 2/2 pass
