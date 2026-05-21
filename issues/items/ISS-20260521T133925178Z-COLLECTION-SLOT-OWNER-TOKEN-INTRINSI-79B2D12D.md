---
id: ISS-20260521T133925178Z-COLLECTION-SLOT-OWNER-TOKEN-INTRINSI-79B2D12D
title: "Collection slot owner-token intrinsic anchors can move storage owners"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/typecheck/prefix_check.rs, nepl-core/tests/effects.rs"
---

# ISS-20260521T133925178Z-COLLECTION-SLOT-OWNER-TOKEN-INTRINSI-79B2D12D: Collection slot owner-token intrinsic anchors can move storage owners

## 概要

collection slot lifecycle intrinsics が `RegionToken<T>` values を proof anchor として受け入れていた。intrinsic argument は source expression であるため、non-Copy owner token を値渡しすると、proof marker のつもりで storage owner を move してしまう。

## 対象

- `nepl-core/src/typecheck/prefix_check.rs, nepl-core/tests/effects.rs`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、collection lifecycle を stdlib helper allowlist ではなく compiler-issued owner / Resource IR / generic proof boundary へ接続することを要求している。
- [ISS-20260521T132527293Z-SOURCE-LEVEL-COLLECTION-DROP-TRAVERS-24EF497F](./ISS-20260521T132527293Z-SOURCE-LEVEL-COLLECTION-DROP-TRAVERS-24EF497F.md) の source-level cleanup fixture では、`RegionToken<T>` を値渡し anchor にすると marker 呼び出し自体が owner move になり、後続 traversal / dealloc marker が false `Moved` diagnostics になった。
- proof marker は ownership transfer ではなく storage identity を参照する semantic boundary なので、owner token anchor は `&RegionToken<T>` に限定する必要がある。

## 問題

`CollectionSlotLifecycleAnchor` は owner token か raw pointer かだけを持ち、owner token が参照から来たのか値から来たのかを区別していなかった。このため次の境界が曖昧になっていた。

- `collection_slot_drop_traversal<T>(storage)` が proof marker であるにもかかわらず、`storage: RegionToken<T>` を consume する source expression になり得る。
- `collection_slot_storage_dealloc(storage)` / `collection_slot_storage_relocate(old, new)` でも同じく marker が owner move を発生させ得る。
- Resource IR 側では proof marker が owner state と混ざり、cleanup helper の正当性を検査する以前に false `Moved` diagnostics が出る。

## 影響

Compiler-owned stdlib cleanup helpers can accidentally encode proof markers as owner-consuming operations. This creates false diagnostics in Resource IR and leaves the proof boundary semantically ambiguous: marker anchors should not move owner tokens.

## 修正方針

compiler-memory anchor が reference 経由か値経由かを `OwnerTokenAnchorAccess` enum として保持し、owner-token collection slot lifecycle anchor は `Borrowed` のみ許可する。raw `MemPtr<T>` anchor は slot-offset lifecycle marker 用に維持する。

## 検証

cargo test -p nepl-core --test effects collection_slot -- --test-threads=1

## 対応

`CollectionSlotLifecycleAnchor::OwnerToken` に `OwnerTokenAnchorAccess::{Borrowed, ByValue}` を持たせ、typecheck 境界で owner-token lifecycle target の by-value access を `IntrinsicArgTypeMismatch` として拒否した。storage relocate は old/new 両方の owner-token anchor を borrowed に限定する。

`MemPtr<T>` anchor は raw pointer anchor として維持したため、slot-offset lifecycle marker の source-level raw proof path は壊していない。tests では storage dealloc / storage relocate / drop traversal の borrowed owner token anchor を typecheck で許可し、by-value owner token anchor を拒否する回帰を追加した。
