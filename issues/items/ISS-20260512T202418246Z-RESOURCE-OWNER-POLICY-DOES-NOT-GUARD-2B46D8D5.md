---
id: ISS-20260512T202418246Z-RESOURCE-OWNER-POLICY-DOES-NOT-GUARD-2B46D8D5
title: "Resource owner policy does not guard non-owning raw view ownership split"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nodesrc/test_resource_checker_responsibility.js; nepl-core/src/resource/owner_summary_raw_transfer.rs; nepl-core/src/resource/owner_summary_raw_view_return.rs"
---

# ISS-20260512T202418246Z-RESOURCE-OWNER-POLICY-DOES-NOT-GUARD-2B46D8D5: Resource owner policy does not guard non-owning raw view ownership split

## 概要

RawAddressViewKind::NonOwningProjection is the compiler-side boundary that keeps MemPtr projections non-owning while owner tokens carry free obligations. The current source policy only checks the raw-view enum exists and line limits; it does not assert that non-owning raw views never transfer owner aliases or that return summaries preserve projection-view identity.

## 対象

- `nodesrc/test_resource_checker_responsibility.js; nepl-core/src/resource/owner_summary_raw_transfer.rs; nepl-core/src/resource/owner_summary_raw_view_return.rs`

## 根拠

- `owner_summary_raw_transfer.rs` は `RawAddressViewKind::NonOwningProjection` を owner alias 転送不可として扱うことで、borrowed `MemPtr` / `RegionToken` projection を free obligation owner に昇格させない。
- `owner_summary_raw_view_return.rs` は `RawAddressViewOwnership::NonOwningProjection` を `OwnerNonOwningRawViewKind::ProjectionView` として summary に残し、callee / caller 境界でも projection 性を失わせない。
- 既存の `nodesrc/test_resource_checker_responsibility.js` は enum の存在と module 分割を確認していたが、この所有権境界の意味までは固定していなかった。

## 問題

RawAddressViewKind::NonOwningProjection is the compiler-side boundary that keeps MemPtr projections non-owning while owner tokens carry free obligations. The current source policy only checks the raw-view enum exists and line limits; it does not assert that non-owning raw views never transfer owner aliases or that return summaries preserve projection-view identity.

## 影響

A later Resource IR refactor could accidentally treat borrowed MemPtr/RegionToken projections as owner-carrying views again, reopening forged dealloc/realloc paths even though focused tests for a single path still pass.

## 修正方針

Extend the resource responsibility policy to require explicit exhaustive matches for raw-address owner transfer and return-summary classification, with NonOwningProjection mapped to no owner alias transfer and ProjectionView.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues

## 2026-05-13 修正

`nodesrc/test_resource_checker_responsibility.js` に brace block 抽出を追加し、次の契約を source policy として固定した。

- `push_transferred_raw_owner_view_aliases` は `raw_address_view_carries_owner_alias(kind)` を必ず通る。
- `RawAddressViewKind::Offset` は owner alias を転送する。
- `RawAddressViewKind::NonOwningProjection` は owner alias を転送しない。
- 上記 match は wildcard arm なしで分岐し、新しい raw view kind を追加した場合は Rust / policy 双方で明示的な判断を要求する。
- non-owning raw view return summary は `NonOwningProjection` を `ProjectionView` として保持し、通常 alias view と混同しない。

これにより `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner` の分離を、単体テストだけでなく source policy でも監視する。
