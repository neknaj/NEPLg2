---
id: ISS-20260521T041844157Z-COLLECTION-SLOT-LIFECYCLE-CAPABILITY-1DE754DB
title: "Collection slot lifecycle capability needs exact stdlib use-site regression"
area: compiler
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/loader.rs, nepl-core/src/source_capability/**"
---

# ISS-20260521T041844157Z-COLLECTION-SLOT-LIFECYCLE-CAPABILITY-1DE754DB: Collection slot lifecycle capability needs exact stdlib use-site regression

## 概要

collection slot lifecycle intrinsics are the bridge from stdlib source to generic Resource IR slot proof. Existing coverage checked typed primitive separation, but did not explicitly lock the boundary to the exact intrinsic span and did not mirror the raw-memory user-source rejection case. That leaves future source capability refactors room to accidentally turn the primitive into file-wide authority or user-source authority.

## 対象

- `nepl-core/src/loader.rs, nepl-core/src/source_capability/**`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を stdlib module allowlist ではなく generic Resource IR proof boundary へ載せることを要求している。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、静的検査の authority を enum / match / typed proof boundary に集約し、表面文字列や file-wide capability に戻さない方針を定めている。
- 実装確認で、`IntrinsicExpr` が intrinsic 名の span を保持しておらず、collection slot lifecycle capability が `#intrinsic ... (args)` 全体の span に結び付いていることが分かった。

## 問題

collection slot lifecycle intrinsics are the bridge from stdlib source to generic Resource IR slot proof. Existing coverage checked typed primitive separation, but did not explicitly lock the boundary to the exact intrinsic span and did not mirror the raw-memory user-source rejection case. That leaves future source capability refactors room to accidentally turn the primitive into file-wide authority or user-source authority.

## 影響

If the boundary spreads beyond the exact stdlib use site, non-Copy collection payload support could bypass the intended compiler-owned proof gate and reintroduce shallow move/drop/free unsoundness.

## 修正方針

Add source capability regression coverage for exact collection slot intrinsic use-site authority, primitive separation, and rejection of identical user-source text outside the configured stdlib root.

## 修正内容

- `IntrinsicExpr` に `name_span` を追加し、parser が intrinsic name string literal の span を保持するようにした。
- source capability walker は intrinsic expression span と intrinsic name literal span の両方を proof event へ渡すようにした。
- collection slot lifecycle evidence だけを intrinsic name literal span に結び付けた。raw builtin evidence と owner/field 系 evidence は既存の expression span 境界を維持し、raw-memory boundary や compiler memory field gate を巻き込まないようにした。
- typecheck の collection slot lifecycle gate は `name_span` を参照するようにし、source proof と typecheck gate の use-site が一致するようにした。
- loader regression で、collection slot lifecycle capability が exact use-site にだけ付き、同一 file 内の unrelated span へ広がらず、同じ source text でも configured stdlib 外では拒否されることを固定した。

## 検証

- `cargo test -p nepl-core collection_slot_lifecycle_boundary -- --test-threads=1`
- `cargo test -p nepl-core --test effects raw_memory_intrinsic_in_core_mem_source_is_allowed_during_migration -- --test-threads=1 --exact`
- `cargo test -p nepl-core --test char char_cast_intrinsics_emit_llvm_as_i32_noops -- --test-threads=1 --exact`
- `cargo check -p nepl-core`
