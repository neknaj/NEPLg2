---
id: ISS-20260516T022823182Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-MISS-2CBBEB43
title: "owner aggregate source evidence misses prefix initializer call heads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/owner_aggregate.rs
---

# ISS-20260516T022823182Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-MISS-2CBBEB43: owner aggregate source evidence misses prefix initializer call heads

## 概要

Owner aggregate source capability only inspects the first prefix item as constructor or field evidence. A valid NEPL prefix initializer such as let boxed <OwnerBox<i32>> OwnerBox<i32> region puts the constructor after let/type annotation, so compiler-owned implementation source can lose the required owner aggregate constructor capability even though the raw source contains a constructor call.

## 対象

- `nepl-core/src/source_capability/owner_aggregate.rs`

## 根拠

- `nepl-core/src/source_capability/owner_aggregate.rs` は修正前、`PrefixExpr.items.first()` だけを `owner_aggregate_call_head_evidence` に渡していた。
- NEPL の prefix initializer では、`let boxed <OwnerBox<i32>> OwnerBox<i32> region` のように `let` と type annotation が先に現れ、その後ろの `OwnerBox<i32>` が実際の constructor call head になる。
- 同じ構文位置は raw memory source evidence 側では既に call-position scanner で扱っていたが、owner aggregate 側に同じ proof primitive がなく、source capability checker 内で構文モデルが分岐していた。

## 問題

Owner aggregate source capability only inspects the first prefix item as constructor or field evidence. A valid NEPL prefix initializer such as let boxed <OwnerBox<i32>> OwnerBox<i32> region puts the constructor after let/type annotation, so compiler-owned implementation source can lose the required owner aggregate constructor capability even though the raw source contains a constructor call.

## 影響

Stage 6 source proof becomes incomplete and can reject legitimate owner aggregate implementation modules, encouraging broader capabilities or ad-hoc allowlists to compensate. The checker also diverges from the raw memory call-head scanner, making source capability mistakes harder to audit.

## 修正方針

Use the same prefix call-head position model for owner aggregate evidence as raw memory evidence: collect constructor or field symbol evidence only when the current prefix item can begin a nested call, while preserving intrinsic field evidence and shadowing/enum-variant exclusions.

## 解決

2026-05-16 に修正した。

- `nepl-core/src/source_capability/prefix_call.rs` を追加し、source capability walker が共有する `PrefixCallHead` tracker を定義した。
- `PrefixCallHead` は expression 先頭と、`let` / `set` / `if` / `while` / `addr-of` / `deref` / type annotation / pipe の直後を nested call-head 位置として扱う。
- raw memory source evidence は既存の local scanner を削除し、共有 tracker を使うようにした。
- owner aggregate source evidence も共有 tracker を使い、constructor evidence と field accessor evidence を prefix initializer 内でも検出するようにした。
- loader regression に `let boxed <OwnerBox<i32>> OwnerBox<i32> region` と `let owner <i32> field::get v "owner"` の positive case を追加した。
- `nodesrc/test_static_check_boundary_responsibility.js` に共有 tracker と両 source capability walker の利用、ならびに owner aggregate initializer regression の監視を追加した。

この修正は stdlib module ごとの allowlist ではない。source capability は compiler-owned source に現れた prefix call-head 構文証拠を抽出するだけであり、owner-backed aggregate かどうかの semantic proof は引き続き typed typecheck / Resource IR 側で行う。

## 検証

Add loader regressions for constructor evidence after let/type annotation and for non-call uppercase arguments; run cargo test -p nepl-core owner_aggregate_boundary -- --nocapture and source policy checks.

実施済み:

- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
