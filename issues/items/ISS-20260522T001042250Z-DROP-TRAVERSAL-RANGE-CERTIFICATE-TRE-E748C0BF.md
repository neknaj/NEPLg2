---
id: ISS-20260522T001042250Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-TRE-E748C0BF
title: "Drop traversal range certificate treats expression output anchor writes as preserving"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_summary_build_range_lifetime.rs, nepl-core/src/resource/collection_slot_summary_build_range_step.rs, nepl-core/src/resource/collection_slot_summary_build_range_step_expr.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T001042250Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-TRE-E748C0BF: Drop traversal range certificate treats expression output anchor writes as preserving

## 概要

Full-range collection slot drop traversal certificates classify ResourceOp::Expr(LocalRead/Call/Intrinsic/Loop) and Read as preserving without checking whether their output directly overwrites the protected storage or initialized_count anchor. The loop-step recognizer also allowed a literal-expression output to overwrite the loop index while still treating the body as a strict `i += 1` induction step. This can let the static-check proof program keep a forall initialized-range certificate across Resource IR operations that actually overwrite the proof anchor.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_range_lifetime.rs`
- `nepl-core/src/resource/collection_slot_summary_build_range_step.rs`
- `nepl-core/src/resource/collection_slot_summary_build_range_step_expr.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 開発方針: https://zenn.dev/bem130/articles/1b352797de94e7

## 問題

Full-range collection slot drop traversal certificates classify ResourceOp::Expr(LocalRead/Call/Intrinsic/Loop) and Read as preserving without checking whether their output directly overwrites the protected storage or initialized_count anchor. The loop-step recognizer also allowed a literal-expression output to overwrite the loop index while still treating the body as a strict `i += 1` induction step. This can let the static-check proof program keep a forall initialized-range certificate across Resource IR operations that actually overwrite the proof anchor.

## 影響

A future lowering or hand-written Resource IR path can reuse a drop traversal certificate after the storage/count/index anchor was overwritten by an expression output. That violates the generic Resource IR proof design and makes the checker itself harder to audit with enum/match semantics.

## 修正方針

Make the range certificate lifetime classifier reject expression/read outputs that directly overwrite the certificate storage or initialized_count anchor, while still allowing ordinary reads into temporaries and existing source-lowering classification markers. Make the loop-step recognizer reject literal-expression writes to the loop index before accepting a strict induction step. Add unit regressions for expression-output storage/count invalidation and index overwrite rejection.

## 検証

Run focused collection slot range certificate tests, nepl-core cargo check/fmt, resource responsibility monitor, and issue index check.

## 解決内容

2026-05-22 に Agent 1 が修正した。

- `collection_slot_summary_build_range_lifetime.rs` で、`ResourceOp::Expr` / `ResourceOp::Read` の output が certificate の `storage` または `initialized_count` を直接上書きする場合に certificate を失効するようにした。
- 一方で、`Read storage -> temporary` や source lowering が発行する `Expr::Call` / `Expr::LocalRead` の分類 marker は、実値生成 op ではないため不必要に失効させない。これにより既存の source-level loop traversal proof を壊さず、anchor 書き換えだけを拒否する。
- `collection_slot_summary_build_range_step.rs` で、loop index へ `LiteralI32(1)` などの expression output が入る body を `i += 1` proof として扱わないようにした。

## 回帰テスト

- `collection_slot_summary_loop_certificate_rejects_post_loop_storage_expr_output`
- `collection_slot_summary_loop_certificate_rejects_post_loop_count_expr_output`
- `collection_slot_summary_loop_induction_rejects_expr_index_overwrite`
