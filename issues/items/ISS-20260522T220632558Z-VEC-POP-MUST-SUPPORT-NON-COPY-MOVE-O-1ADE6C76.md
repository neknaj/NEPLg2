---
id: ISS-20260522T220632558Z-VEC-POP-MUST-SUPPORT-NON-COPY-MOVE-O-1ADE6C76
title: "Vec pop must support non-Copy move-out through owner-preserving API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/mutation/pop.nepl, stdlib/alloc/collections/vec/types.nepl, nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js"
---

# ISS-20260522T220632558Z-VEC-POP-MUST-SUPPORT-NON-COPY-MOVE-O-1ADE6C76: Vec pop must support non-Copy move-out through owner-preserving API

## 概要

Vec.pop is still Copy-only even though Resource IR can prove raw load plus collection_slot_move_out. Removing Copy directly would hide the popped Option<T> owner when callers only recover the Vec field.

## 対象

- `stdlib/alloc/collections/vec/mutation/pop.nepl, stdlib/alloc/collections/vec/types.nepl, nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js`

## 根拠

- 未記入

## 問題

Vec.pop is still Copy-only even though Resource IR can prove raw load plus collection_slot_move_out. Removing Copy directly would hide the popped Option<T> owner when callers only recover the Vec field.

## 影響

Non-Copy collection payload support remains incomplete for stack-like move-out APIs, blocking self-host data structures that need to extract owned elements from Vec without dropping them.

## 修正方針

Add a Drop-capable pop overload using a private move-out proof helper and expose a VecPop consuming eliminator that passes Vec<T> and Option<T> owners together.

## 検証

Run Vec source policy, collection cleanup contract, focused Resource IR non-Copy pop tests, doctest for vec/mutation/pop.nepl, issues check/index, cargo fmt, and git diff --check.

## 2026-05-22 Agent 1 修正

`pop<T: Drop>` を追加し、`pop<T: Copy>` と同じ private `vec_pop_storage_checked<T>` に委譲する構造へ整理した。`vec_pop_storage_checked<T>` は `VecStorageInvariant` で `len` / `initialized_len` / `cap` / storage extent を確認し、要素がある場合だけ `vec_pop_move_out_initialized_slot<T>` へ tail slot の raw load と `collection_slot_move_out` proof を閉じる。public `pop` body は raw pointer operation や lifecycle marker を open-code しない。

`VecPop<T>` から owner を片方だけ取り出す API は non-Copy では危険なので、`vec_pop_with<T, R>` を追加した。これは `Vec<T>` owner と `Option<T>` owner を同じ callback に同時に渡す owner-preserving eliminator であり、caller が `Option::Some value` と更新後 `Vec` を同一 control-flow で回収できるようにする。

実装中に、Resource IR owner summary が `RegionToken<T>` raw owner leaf を通常の owner leaf として seed していないため、高階 eliminator が callback へ owner を渡して消費しても direct call summary が空になる問題を発見した。これは [ISS-20260522T224620549Z-OWNER-SUMMARY-MUST-SEED-OWNER-TOKEN--89E3E5BE](./ISS-20260522T224620549Z-OWNER-SUMMARY-MUST-SEED-OWNER-TOKEN--89E3E5BE.md) として分離し、`owner_summary_leaf` が compiler-proven owner token raw field を最初から owner leaf として扱うようにした。stdlib 関数名 allowlist は追加していない。

回帰テストとして `Vec<DropPayload>.new -> push -> push -> pop -> vec_pop_with -> free/drop` を追加し、initialized slot MoveOut と owner obligation が callback 境界を越えて閉じることを固定した。`NonCopyPayload` の compile_fail doctest は Copy/Drop どちらの overload にも合わない `type.overload.no_match` を期待する形へ更新した。

検証:

- `cargo test -p nepl-core resource::owner_summary_leaf::tests::owner_leaf_places_seed_owner_token_raw_field -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_copy_pop_moves_out_tail_slot -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_pop_moves_out_tail_slot_and_recovers_owners -- --test-threads=1 --exact --nocapture`
- `trunk build`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/mutation/pop.nepl -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/mutation/pop.nepl -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/mutation/pop.nepl -n 3 --dist web/dist`
- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
