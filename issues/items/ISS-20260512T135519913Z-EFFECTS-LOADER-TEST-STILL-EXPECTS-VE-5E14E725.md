---
id: ISS-20260512T135519913Z-EFFECTS-LOADER-TEST-STILL-EXPECTS-VE-5E14E725
title: "Effects loader test still expects vec sort merge facade raw boundary"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/tests/effects.rs; nepl-core/src/loader.rs; stdlib/alloc/collections/vec/sort/merge.nepl"
---

# ISS-20260512T135519913Z-EFFECTS-LOADER-TEST-STILL-EXPECTS-VE-5E14E725: Effects loader test still expects vec sort merge facade raw boundary

## 概要

cargo test -p nepl-core --test effects fails because loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary still expects alloc/collections/vec/sort/merge.nepl to receive raw-memory-boundary capability, even though the merge file is now a facade and raw memory moved to merge/api, merge/buffer, and merge/range.

## 対象

- `nepl-core/tests/effects.rs; nepl-core/src/loader.rs; stdlib/alloc/collections/vec/sort/merge.nepl`

## 根拠

- `nepl-core/src/loader.rs` の exact raw-memory-boundary table は `alloc/collections/vec/sort/merge/api.nepl`、`merge/buffer.nepl`、`merge/range.nepl` を許可している。
- `nepl-core/tests/effects.rs` の regression は古い `alloc/collections/vec/sort/merge.nepl` facade に raw-memory-boundary capability を期待していた。
- `stdlib/alloc/collections/vec/sort/merge.nepl` は現在 raw memory implementation ではなく submodule re-export facade である。

## 問題

cargo test -p nepl-core --test effects fails because loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary still expects alloc/collections/vec/sort/merge.nepl to receive raw-memory-boundary capability, even though the merge file is now a facade and raw memory moved to merge/api, merge/buffer, and merge/range.

## 影響

The effects test target cannot be used as a clean regression suite for raw-memory-boundary changes, and stale test expectations can pressure the loader to re-grant raw capability to a facade instead of exact implementation modules.

## 修正方針

Update the loader raw-memory-boundary regression to match the current vec sort merge split: remove the facade expectation and assert the exact implementation modules that own raw memory remain covered.

## 対応記録

- `loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary` から `alloc/collections/vec/sort/merge.nepl` facade の raw-memory-boundary 期待を削除した。
- 代わりに `alloc/collections/vec/sort/merge/api.nepl`、`merge/buffer.nepl`、`merge/range.nepl` が exact raw-memory-boundary capability を受け取ることを検査するようにした。
- `nepl-core/src/loader.rs` の capability table は現行設計どおり維持し、facade へ権限を戻していない。

## 検証

- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test effects -- --nocapture`: passed
