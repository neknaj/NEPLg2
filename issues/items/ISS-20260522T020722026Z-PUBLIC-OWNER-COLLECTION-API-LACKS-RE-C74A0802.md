---
id: ISS-20260522T020722026Z-PUBLIC-OWNER-COLLECTION-API-LACKS-RE-C74A0802
title: "Public owner collection API lacks Resource IR end-to-end proof"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-22
updated: 2026-05-22
target: nepl-core/tests/collection_slot_full_range.rs
---

# ISS-20260522T020722026Z-PUBLIC-OWNER-COLLECTION-API-LACKS-RE-C74A0802: Public owner collection API lacks Resource IR end-to-end proof

## 概要

Source capability now allows public owner aggregate APIs to call private collection lifecycle helpers, but Resource IR lacked an end-to-end regression that proves such APIs remain safe without exposing marker authority or relying on stdlib allowlists.

## 対象

- `nepl-core/tests/collection_slot_full_range.rs`

## 根拠

- [ISS-20260522T014532659Z-COLLECTION-SLOT-LIFECYCLE-SOURCE-CAP-8C046921](./ISS-20260522T014532659Z-COLLECTION-SLOT-LIFECYCLE-SOURCE-CAP-8C046921.md) で、owner aggregate 境界の public collection API は source capability で遮断せず、後段の typecheck / Resource IR proof に渡す方針にした。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、stdlib module allowlist ではなく Resource IR の typed proof boundary で non-Copy collection lifecycle を証明することを要求している。
- 調査時に `MemPtr<T>` と `&RegionToken<T>` を別々の引数として受け取る helper も試したが、関数単体のソースからは両者が同じ storage を指すことを証明できないため拒否された。これは検査漏れではなく、未表現の alias precondition に依存する helper 設計を静的検査が受け入れないという正しい安全側の挙動である。

## 問題

Source capability now allows public owner aggregate APIs to call private collection lifecycle helpers, but Resource IR lacked an end-to-end regression that proves such APIs remain safe without exposing marker authority or relying on stdlib allowlists.

## 影響

A later change could keep loader/source capability behavior green while regressing Resource IR cleanup, owner-transfer, or storage-dealloc proof for public collection APIs.

## 修正方針

Add a source-level regression where a public owner aggregate API extracts a private RegionToken, calls private helpers that derive non-owning MemPtr values from that token inside the helper, initializes non-Copy slots, runs certified drop traversal, and releases storage.

## 修正内容

- `public_owner_collection_api_uses_private_lifecycle_helpers` を追加し、public `OwnerCollection` API が private helper 経由で non-Copy slot initialize、full-range drop traversal、raw dealloc、storage dealloc を一続きで証明できることを固定した。
- private initialize helper は `&RegionToken<T>` だけを受け取り、helper 内で `region_ptr` から `MemPtr<T>` を導出する形にした。これにより `MemPtr = non-owning view`、`RegionToken = owner/storage anchor` の責務分割をソースコード上で証明可能にしている。
- public API は lifecycle intrinsic を直接公開せず、source capability の public-surface rejection と Resource IR の end-to-end proof の両方を満たす。

## 検証

cargo test -p nepl-core --test collection_slot_full_range public_owner_collection_api_uses_private_lifecycle_helpers -- --test-threads=1 --exact --nocapture

結果: passed
