---
id: ISS-20260512T114530848Z-RESOURCE-OWNER-SUMMARY-PRE-SCAN-TREA-A21564CA
title: "Resource owner summary pre-scan treats non-owning raw views as owner aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/owner_summary_raw_alias.rs, nepl-core/src/resource/owner_summary_raw_use.rs"
---

# ISS-20260512T114530848Z-RESOURCE-OWNER-SUMMARY-PRE-SCAN-TREA-A21564CA: Resource owner summary pre-scan treats non-owning raw views as owner aliases

## 概要

Owner summary raw-owner pre-scans propagate ResourceOp::RawAddressView without checking RawAddressViewKind. NonOwningProjection is a borrowed pointer view and must not participate in raw owner alias discovery.

## 対象

- `nepl-core/src/resource/owner_summary_raw_alias.rs, nepl-core/src/resource/owner_summary_raw_use.rs`

## 根拠

- `owner_summary_raw_alias.rs` と `owner_summary_raw_use.rs` は `ResourceOp::RawAddressView` を `RawAddressAlias` と同じ raw owner alias propagation として扱っていた。
- `RawAddressViewKind::NonOwningProjection` は borrowed `RegionToken` / `str_addr` 由来の non-owning pointer view であり、free obligation owner を返す・消費する根拠にはならない。
- `RawAddressViewKind::Offset` は owner-carrying raw address からの offset projection になり得るため、こちらは owner alias discovery に残す必要がある。

## 問題

Owner summary raw-owner pre-scans propagate ResourceOp::RawAddressView without checking RawAddressViewKind. NonOwningProjection is a borrowed pointer view and must not participate in raw owner alias discovery.

## 影響

A non-owning MemPtr/RegionToken projection can be treated as a possible free-obligation alias while building owner summaries, increasing false owner-transfer pressure and obscuring the MemPtr=non-owning / Storage=owner split.

## 修正方針

Classify RawAddressViewKind in the raw-owner summary pre-scan and propagate only owner-carrying offset views. Keep NonOwningProjection out of raw owner alias discovery with an enum match.

## 検証

Focused Rust regression for the raw-owner summary scan plus Resource IR owner/provenance tests.

## 対応結果

2026-05-12 に修正済み。

- `owner_summary_raw_transfer.rs` に `push_transferred_raw_owner_view_aliases` を追加し、`RawAddressViewKind` を exhaustive `match` で分類するようにした。
- `Offset` は従来通り raw owner alias discovery へ伝播する。
- `NonOwningProjection` は borrowed pointer view として raw owner alias discovery へ伝播しない。
- `owner_summary_raw_alias.rs` と `owner_summary_raw_use.rs` は `RawAddressAlias` と `RawAddressView` を同じ arm にまとめず、view kind を見て分岐する。
- 回帰テストで `NonOwningProjection` が owner alias を増やさないこと、`Offset` が owner alias を維持することを固定した。

検証:

- `cargo test -p nepl-core owner_summary_raw_transfer -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_str_addr_view -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_dealloc_through_result_wrapped_str_addr_view -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_alloc_ptr_raw_owner_return -- --nocapture`
- `cargo fmt --check -p nepl-core`
- `cargo check -p nepl-core --tests`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-non-owning-raw-view-summary-memory-safety.json -j 1 --dist web/dist`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
