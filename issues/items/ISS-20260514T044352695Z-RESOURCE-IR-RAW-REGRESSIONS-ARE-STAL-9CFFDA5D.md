---
id: ISS-20260514T044352695Z-RESOURCE-IR-RAW-REGRESSIONS-ARE-STAL-9CFFDA5D
title: "Resource IR raw regressions are stale after explicit import and raw boundary changes"
area: compiler
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-14
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260514T044352695Z-RESOURCE-IR-RAW-REGRESSIONS-ARE-STAL-9CFFDA5D: Resource IR raw regressions are stale after explicit import and raw boundary changes

## 概要

cargo test -p nepl-core --test resource_ir raw -- --nocapture no longer provides a clean static-check regression signal. Several raw-focused Resource IR tests fail before exercising the checker because snippets rely on implicit math imports, and at least one positive compiler-pipeline assertion still compiles direct raw memory source without explicit raw-memory-boundary capability.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir raw -- --nocapture` で、raw fixture が現在の明示 import / raw boundary / `MemPtr` non-owning 方針に追従していないため失敗していた。
- `alloc_ptr` 由来の `MemPtr` を `mem_ptr_addr` で raw `i32` に落として `dealloc_raw` へ渡す旧 fixture は、現在の `MemPtr = non-owning pointer` / `OwnedRegion or Storage = free obligation owner` 方針と矛盾していた。

## 問題

cargo test -p nepl-core --test resource_ir raw -- --nocapture no longer provides a clean static-check regression signal. Several raw-focused Resource IR tests fail before exercising the checker because snippets rely on implicit math imports, and at least one positive compiler-pipeline assertion still compiles direct raw memory source without explicit raw-memory-boundary capability.

## 影響

The raw Resource IR regression filter can hide real memory-safety or owner-summary regressions behind stale fixture failures, and can pressure future changes to weaken import or raw boundary checks instead of preserving the current static-check design.

## 修正方針

Update the Rust Resource IR fixtures to import the current modules explicitly, and route raw-positive compiler-pipeline checks through an explicit raw-memory-boundary test helper. Do not relax user-source raw memory rejection or Resource IR gates.

## 検証

Run the corrected focused Resource IR raw tests, static check source policies, issue index/check, and cargo fmt check.

## 対応内容

- `Resource IR` raw fixture の source snippets に、現在の resolver で必要な `core/math` / `core/field` import を明示した。
- raw memory を直接使う positive compiler pipeline assertion は、通常 user source の raw rejection を迂回しないよう、test 専用 helper で `SourceCapabilities::raw_memory_boundary()` を明示する形に分離した。
- manual Resource IR の raw owner read fixture は、raw owner を読み出した後に元 place を再読みに行く古い期待をやめ、転送先 place を load / dealloc に使う現在の owner transfer model に合わせた。
- `alloc_ptr` owner を `mem_ptr_addr` 経由で raw `i32` に移せるという旧 fixture は、`mem_ptr_addr` が free obligation owner ではないことを固定する拒否テストへ変更した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir raw -- --nocapture` は 71/71 passing。
