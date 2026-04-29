---
id: ISS-20260429T012328323Z-RESOURCE-IR-LACKS-STORAGE-ORIGIN-FOR-549F82A4
title: "Resource IR lacks storage origin for unmanaged raw addresses"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/lower.rs, stdlib/core/mem.nepl"
---

# ISS-20260429T012328323Z-RESOURCE-IR-LACKS-STORAGE-ORIGIN-FOR-549F82A4: Resource IR lacks storage origin for unmanaged raw addresses

## 概要

Resource owner gate can distinguish Live/Moved/Freed owner obligations, but raw i32 constants and legacy unmanaged addresses have no storage origin. Treating every dealloc without a free obligation as D3100 breaks migration fixtures, while leaving NoFreeObligation shadow-only would hide real owned-storage bugs.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/lower.rs, stdlib/core/mem.nepl`

## 根拠

- `ISS-20260429T004144320Z-RESOURCE-OWNER-GATE-TREATS-RAW-POINT-216A5E25` の修正中、owner gate を D3100 に接続すると `tests/compiler/move_effect.n.md::doctest#35` の positive case が `Dealloc ... found NoFreeObligation` で失敗した。
- 該当 case は `let p <i32> 16` という固定 raw address に `LocalToken` を store/load した後で `dealloc_raw p` する移行用 fixture であり、`alloc_raw` が発行した free obligation owner ではない。
- 一方で、owned storage に対する NoFreeObligation を常に shadow-only にすると、Resource IR owner gate が本来検出すべき「owner obligation を失った dealloc/realloc」を見逃す。
- `doc/neplg2/static_check_complexity_reduction_plan.md` は `MemPtr = non-owning pointer` と `Storage/OwnedRegion = free obligation owner` の分離を完了条件にしているため、raw address が owned storage 由来か unmanaged/external 由来かを Resource IR が保持する必要がある。

## 問題

Resource owner gate can distinguish Live/Moved/Freed owner obligations, but raw i32 constants and legacy unmanaged addresses have no storage origin. Treating every dealloc without a free obligation as D3100 breaks migration fixtures, while leaving NoFreeObligation shadow-only would hide real owned-storage bugs.

## 影響

Stage 4 owner gate cannot be fully authoritative until Resource IR separates compiler-owned storage from external/unmanaged raw addresses. Stage 5/6 core/mem migration also needs this distinction to close public raw address escape without rejecting internal or legacy unmanaged storage cases incorrectly.

## 修正方針

Add a storage origin/provenance classification to Resource IR, for example OwnedStorage versus ExternalUnmanagedStorage/InternalRawBoundary. OwnerState checks should report NoFreeObligation as D3100 only for places that are expected to carry an owned free obligation; unmanaged or internal raw storage must be controlled by explicit capability/effect boundary instead of silent shadow behavior.

## 検証

Add Resource IR owner tests for owned alloc double-free/no-obligation, unmanaged fixed-address dealloc, and internal raw boundary behavior. Then enable NoFreeObligation owner diagnostics for owned storage while keeping tests/compiler/move_effect.n.md D3025 and D3100 expectations stable.

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check -- --nocapture`: 18 passed
- `cargo test -p nepl-core compiler::tests::resource_owner_gate -- --nocapture`: 3 passed
- `rustfmt --check nepl-core\src\compiler.rs nepl-core\src\resource\model.rs nepl-core\src\resource\mod.rs nepl-core\src\resource\owner_check.rs nepl-core\src\resource\storage_origin.rs nepl-core\src\resource\summary.rs nepl-core\tests\resource_ir.rs`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-storage-origin-move-effect.json -j 1`: total=110, passed=110, failed=0
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-resource-storage-origin-move-check.json -j 1`: total=52, passed=52, failed=0

## 対応結果

Resource IR の `ResourceState` に `StorageOriginEntry` / `StorageOrigin` を追加し、owner checker 内では `StorageOriginTable` として owner state とは別に追跡するようにした。`alloc_raw` / fresh owner summary は owned storage origin を発行し、`Read` / `RawAddressAlias` は free obligation を移動せず non-owning alias として origin だけを伝搬する。

`dealloc` / `realloc` は owner state が存在しない場合でも、対象 place または raw alias が owned storage origin を持つなら `NoFreeObligation` を owner diagnostic として出す。一方で、固定 raw address のように owned origin を持たない unmanaged address は owner obligation の対象外として扱い、移行 fixture の false positive を避ける。

compiler gate では `NoFreeObligation` を shadow-only にしないように戻した。これにより、owned storage origin を持つ stale alias の dealloc は D3100 へ上がり、unmanaged raw address は origin がないため D3100 にならない。

## 関連

- 親 issue: `ISS-20260425T000000Z-RV-CORE-009-58589A3F`
- 前段 issue: `ISS-20260429T004144320Z-RESOURCE-OWNER-GATE-TREATS-RAW-POINT-216A5E25`
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
