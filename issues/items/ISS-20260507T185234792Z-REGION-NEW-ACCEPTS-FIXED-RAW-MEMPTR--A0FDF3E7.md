---
id: ISS-20260507T185234792Z-REGION-NEW-ACCEPTS-FIXED-RAW-MEMPTR--A0FDF3E7
title: "region_new accepts fixed raw MemPtr without owned provenance"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/storage_origin.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260507T185234792Z-REGION-NEW-ACCEPTS-FIXED-RAW-MEMPTR--A0FDF3E7: region_new accepts fixed raw MemPtr without owned provenance

## 概要

region_new copies the raw address alias from MemPtr into RegionToken, but the Resource IR does not mark that RegionToken raw field as requiring owned storage provenance. A fixed raw address can therefore be wrapped into MemPtr, promoted to RegionToken, and passed to dealloc_region without resource.owner.no_free_obligation.

## 対象

- `nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `region_new` の Resource IR lowering は `MemPtr.raw` から `RegionToken.ptr.raw` への `RawAddressAlias` だけを生成しており、`RegionToken` 側の raw field が owned storage provenance を要求することを IR 上に残していなかった。
- `StorageOriginTable` は exact place / ancestor origin だけをコピーしており、`RegionToken` value move や `Read` を挟むと `token.ptr.raw` 配下の origin が whole value の移動に追従しなかった。
- `dealloc_region` の owner summary consumption は whole `RegionToken` またはその projection を消費するため、`RegionToken.ptr.raw` 配下の owned origin を whole token 側からも検出する必要があった。

## 問題

region_new copies the raw address alias from MemPtr into RegionToken, but the Resource IR does not mark that RegionToken raw field as requiring owned storage provenance. A fixed raw address can therefore be wrapped into MemPtr, promoted to RegionToken, and passed to dealloc_region without resource.owner.no_free_obligation.

## 影響

Safe source can manufacture an owner-token-shaped RegionToken from an arbitrary raw address and reach checked deallocation without the compiler proving a free obligation. This weakens the MemPtr = non-owning pointer / RegionToken = owner token split required by the static check complexity reduction plan.

## 修正方針

Represent region_new's output raw field as an explicit owner storage obligation in Resource IR and make owner checking reject deallocation when no transferable owner exists. Keep non-owning view propagation distinct from owned storage origin.

## 対応

- `ResourceOp::StorageOrigin { target, origin, span }` を追加し、Resource IR dump と各静的検査 pass の match に明示的に接続した。
- `region_new` の lowering は、既存の raw address alias 生成後に `RegionToken.ptr.raw` へ `StorageOrigin::Owned` を付与する。これにより `RegionToken` が owner-token-shaped value であることを Resource IR 上の provenance obligation として表す。
- `StorageOriginTable` は prefix 配下の origin を value move / read / assign に追従できるようにし、`RegionToken` 全体を動かしても `token.ptr.raw` の owned origin が失われないようにした。
- owner checker は whole value の配下に owned origin がある場合も free obligation 要求として扱い、転送可能 owner がない `region_new (mem_ptr_wrap 16)` 由来 token の `dealloc_region` を `resource.owner.no_free_obligation` で拒否する。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_fixed_mem_ptr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir region_token_forged -- --nocapture`: 5 passed
- `cargo test -p nepl-core --test resource_ir region_ptr -- --nocapture`: 11 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 241 passed
- `cargo fmt --check -p nepl-core`: passed
- remote main `b9a10e24` rebase 後 `cargo check -p nepl-core`: passed
- remote main `b9a10e24` rebase 後 `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_fixed_mem_ptr -- --nocapture`: passed
- remote main `b9a10e24` rebase 後 `cargo test -p nepl-core --test resource_ir unknown_callback -- --nocapture`: 5 passed
- remote main `b9a10e24` rebase 後 `cargo test -p nepl-core --test resource_ir region_token_forged -- --nocapture`: 5 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-region-new-fixed-raw-memory-safety.json -j 1 --dist web/dist`: 22 passed
- remote main `b9a10e24` rebase 後 `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-region-new-fixed-raw-memory-safety-rebased.json -j 1 --dist web/dist`: 22 passed
- `node nodesrc/issues.js index`: total=618, open=10, resolved=608
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
