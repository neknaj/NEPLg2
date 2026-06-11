---
id: ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7
title: "MemoKey and MemoValue need structural purity rules"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-12
target: "nepl-core/src/types.rs; nepl-core/src/typecheck; stdlib/std; stdlib/neplg2/core/ty"
---

# ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7: MemoKey and MemoValue need structural purity rules

## 概要

Phase 1 memo_call can use Copy-like key/value types, but TypeCtx::is_copy alone is too broad because function values, references, raw pointers, owner tokens, public mutable state, and external handles must not become MemoKey or MemoValue.

## 対象

- `nepl-core/src/types.rs; nepl-core/src/typecheck; stdlib/std`

## 根拠

- `doc/neplg2/private_effect_memoization_purity_design.md` の Phase 1 方針に従う。
- `memo_call` は private cache backend が入るまでは `MemoKey&Copy` / `MemoValue&Copy` かつ Drop なしの構造値だけを受け入れ、identity や resource lifecycle が観測可能な値を拒否する必要がある。
- ordinary `Copy` は低レベル境界の軽量 handle にも付与されるため、`MemoKey` / `MemoValue` trait と Phase 1 structural predicate は function value、reference、raw memory view、owner token を追加で拒否する。

## 問題

Phase 1 memo_call can use Copy-like key/value types, but TypeCtx::is_copy alone is too broad because function values, references, raw pointers, owner tokens, public mutable state, and external handles must not become MemoKey or MemoValue.

## 影響

If MemoKey or MemoValue is treated as a simple Copy alias, memo_call can cache values whose identity or behavior is externally observable, breaking the Pure contract.

## 修正方針

Define structural MemoKey and MemoValue rules that require pure Eq/Hash/Clone/Drop where applicable and explicitly reject function values, references, raw pointers, owner tokens, public mutable state, external resources, unknown effect values, and non-Copy/Drop values in Phase 1.

## 2026-05-31 checkpoint

- `stdlib/core/traits/memo.nepl` に `MemoKey` / `MemoValue` trait を追加し、`memo_call` の public signature を `.K: MemoKey&Copy, .V: MemoValue&Copy` にした。
- `memo_call` Phase 1 predicate は `ctx.is_copy`、`ctx.has_drop`、compiler memory type check、`MemoKey` / `MemoValue` trait bound を組み合わせる。key 側は `unit`、`i32`、`u8`、`bool`、`char` と、それらだけで構成される recursive structural Copy aggregate を受け入れる。value 側は同じ範囲に加えて `f32` も受け入れる。
- compiler-known primitive gate が参照する `MemoKey` / `MemoValue` trait definition は `stdlib/core/traits/memo.nepl` の source identity を確認する。
- accepted regression として、user-defined `Pair` に `MemoKey` / `MemoValue` / `Clone` / `Copy` impl があり、field が `i32` だけで構成される場合に `memo_call @same_pair` が通ることを確認した。
- rejected regression として、Copy だが `MemoKey` がない struct、Copy かつ `MemoKey` だが `MemoValue` がない struct、non-Copy struct、`str`、`f32` key、`f32` field を持つ structural key、function value、reference、`MemPtr i32`、`RegionToken i32`、unresolved generic function value を追加した。
- `unit` keyword が trait impl method signature の一部経路で fresh type variable になっていたため、type expression lowering で intrinsic type name として扱うようにした。これにより `MemoKey for unit` / `MemoValue for unit` の標準 impl が signature mismatch なしで検査される。
- unit key/value の regression を追加し、`%fn (unit) i32` の grouped unary unit argument と `%fn unit i32` の zero-argument function marker を区別して固定した。
- この checkpoint では private cache backend がまだないため、cache algorithm correctness、official external handle marker、`MemoKey` / `MemoValue` impl の semantic validation は継続して設計する。

## 検証

Accepted tests should cover primitive scalar/unit/structural Copy values; rejected tests should cover function keys, impure Eq/Hash/Clone/Drop, references, raw pointers, owner tokens, mutable/public state, external handles, and non-Copy values.

Current Phase 1 regression is covered by `cargo test -p nepl-core function_memo_call --test functions -- --nocapture`.

## 2026-06-12 selfhost predicate checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait.nepl` を追加し、selfhost compiler 側でも `MemoKey` / `MemoValue` の Phase 1 predicate を持つようにした。主 API は `Result unit SelfhostMemoTraitRejectKind` であり、`bool` helper はこの typed result から派生する補助に留めた。

現 selfhost predicate は `unit`、`bool`、`i32`、`u8`、`char` を `MemoKey` と `MemoValue` の両方で受理し、`f32` は `MemoValue` だけで受理する。`f32` key、`I64`、`F64`、`str`、`never`、`error`、function type、missing TypeId、generic parameter は enum reason 付きで拒否する。

Rust 実装の Phase 1 は structural Copy aggregate acceptance まで持つが、selfhost の現行 `SelfhostTypeArena` は named / applied type の field layout、trait impl evidence、Drop / Copy proof を持たない。そのため、この checkpoint では `NamedLayoutUnknown` / `AppliedLayoutUnknown` として fail-closed にする。aggregate acceptance は、type constructor layout evidence と trait solver が入った後にこの issue の後続 slice として接続する。
