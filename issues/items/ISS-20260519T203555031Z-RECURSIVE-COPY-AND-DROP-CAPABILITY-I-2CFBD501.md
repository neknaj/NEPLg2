---
id: ISS-20260519T203555031Z-RECURSIVE-COPY-AND-DROP-CAPABILITY-I-2CFBD501
title: "Recursive Copy and Drop capability impls can self-prove type patterns"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: "nepl-core/src/types.rs; nepl-core/tests/neplg2.rs; nepl-core/tests/drop.rs"
---

# ISS-20260519T203555031Z-RECURSIVE-COPY-AND-DROP-CAPABILITY-I-2CFBD501: Recursive Copy and Drop capability impls can self-prove type patterns

## 概要

TypeCtx capability queries for Copy and Drop call type_pattern_matches against capability impl targets without a per-query recursion stack. Recursive blanket impls such as impl<.T: Copy> Copy for .T or impl<.T: Drop> Drop for .T can re-enter the same capability query for the same target while checking the pattern bound.

## 対象

- `nepl-core/src/types.rs; nepl-core/tests/neplg2.rs; nepl-core/tests/drop.rs`

## 根拠

- `Clone` capability bound は `TypeCtx` の query stack で再帰を拒否していたが、`Copy` / `Drop` は `type_pattern_matches` から `is_copy` / `has_drop` を呼ぶ経路が独立した query stack を持たなかった。
- `impl<.T: CopyLike> CopyLike for .T` や `impl<.T: Drop> Drop for .T` は、自分自身の impl target matching を通じて同じ capability query に戻れるため、独立した証明なしに trait bound を満たしたように扱われる危険があった。

## 問題

TypeCtx capability queries for Copy and Drop call type_pattern_matches against capability impl targets without a per-query recursion stack. Recursive blanket impls such as impl<.T: Copy> Copy for .T or impl<.T: Drop> Drop for .T can re-enter the same capability query for the same target while checking the pattern bound.

## 影響

The static checker can stack-overflow or accept a circular capability proof instead of requiring an independent source proof. This weakens type safety and drop/owner safety for abstraction-heavy code.

## 修正方針

Add per-query visiting sets for Copy and Drop capability queries, thread them through type-pattern capability checks alongside the Clone query stack, and reject self-recursive proofs. Add regression tests for Copy and Drop recursive blanket impls.

## 対応結果

- `CapabilityQueryStack` を導入し、`copy` / `clone` / `drop` の訪問中 target を1つの query context で管理するようにした。
- `TypeCtx::type_pattern_matches` と impl target registry query を stack-aware にし、`pattern_var_capabilities_match` が `copy_cap` / `clone_cap` / `drop_cap` を同じ証明文脈で評価するようにした。
- `is_copy` / `has_clone` / `has_drop` は公開 API では新しい stack を作り、内部再帰では既存 stack を共有する。再入した capability target は「まだ証明できていない」として拒否する。
- これにより、stdlib 名や trait 名の allowlist ではなく、TypeCtx の capability proof registry と query stack による汎用的な循環証明拒否になる。

## 回帰テスト

- `recursive_copy_capability_impl_does_not_prove_itself`: `CopyLike` の循環 blanket impl が `Payload` の `CopyLike` bound を満たさないことを検査する。
- `recursive_drop_capability_impl_does_not_prove_itself`: `Drop` の循環 blanket impl が `Payload` の `Drop` bound を満たさないことを検査する。
- 既存の `recursive_clone_capability_impl_does_not_prove_itself` も同じ stack 経路で維持する。

## 検証

- `cargo test -p nepl-core --test neplg2 recursive_copy_capability_impl_does_not_prove_itself -- --nocapture`
- `cargo test -p nepl-core --test drop recursive_drop_capability_impl_does_not_prove_itself -- --nocapture`
- `cargo test -p nepl-core --test neplg2 recursive_clone_capability_impl_does_not_prove_itself -- --nocapture`
- `cargo test -p nepl-core --test neplg2 copy_impl -- --nocapture`
- `cargo test -p nepl-core --test drop drop_impl_rejects -- --nocapture`
