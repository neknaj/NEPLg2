---
id: ISS-20260518T013143510Z-RESOURCE-IR-RAW-IDENTITY-ESCAPE-MISS-DB101F7A
title: "Resource IR raw identity escape misses returned raw slot aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/raw_pointer_type.rs, nepl-core/src/resource/effect_summary_pointer.rs, nepl-core/src/resource/effect_summary_pointer_seed.rs, nepl-core/tests/resource_ir.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260518T013143510Z-RESOURCE-IR-RAW-IDENTITY-ESCAPE-MISS-DB101F7A: Resource IR raw identity escape misses returned raw slot aliases

## 概要

move_effect doctest#16/#17 expect resource.raw.identity_escape for alloc_raw identity written through a slot pointer returned by a helper, but current Resource IR reports only raw boundary/effect diagnostics. Direct parameter slots and local aliases still report identity_escape, so the missing path is raw slot alias propagation through return summaries and function-value calls.

## 対象

- `nepl-core/src/resource/effect_identity.rs, nepl-core/src/resource/effect_summary_pointer.rs, tests/compiler/move_effect.n.md`

## 根拠

- `tests/compiler/move_effect.n.md::doctest#16/#17` は `resource.raw.identity_escape` を期待していたが、修正前は `resource.raw.memory_outside_boundary` と `effect.pure.calls_impure` だけが出ていた。
- 隣接する doctest#13/#14/#15 は direct parameter slot、local copy alias、store/load helper 内の raw slot では `resource.raw.identity_escape` を検出できていた。
- 差分から、raw identity tracking そのものではなく、helper が `i32` raw slot pointer を返す場合の Resource IR raw pointer return summary が alias fact を落としていたことが分かる。

## 問題

move_effect doctest#16/#17 expect resource.raw.identity_escape for alloc_raw identity written through a slot pointer returned by a helper, but current Resource IR reports only raw boundary/effect diagnostics. Direct parameter slots and local aliases still report identity_escape, so the missing path is raw slot alias propagation through return summaries and function-value calls.

## 影響

A pure function can hide raw allocation identity by receiving a raw slot pointer through a helper return before storing/loading it. Even when raw boundary diagnostics also fire for ordinary source, compiler-owned raw-boundary code could miss the identity escape proof and return an owned raw address as plain i32.

## 修正方針

Resource IR effect identity / pointer summary propagation を、direct helper と function-value call から返る raw slot alias にも適用する。`MemPtr` 判定とは別に、summary が「既に証明された raw pointer alias fact」を運べる型を判定し、plain `i32` raw address slot と aggregate 内の `i32` carrier を型構造から扱う。stdlib 関数名や module 名の列挙ではなく、Resource IR の dataflow summary と typed `TypeKind` match から証明する。

## 検証

- `cargo test -p nepl-core raw_pointer_type -- --nocapture`: passed
- `cargo test -p nepl-core returned_i32_slot_alias -- --nocapture`: 2 passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 13 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 14 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 15 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 16 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 17 --dist web/dist`: passed
