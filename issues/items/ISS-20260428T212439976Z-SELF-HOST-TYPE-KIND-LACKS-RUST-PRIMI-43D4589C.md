---
id: ISS-20260428T212439976Z-SELF-HOST-TYPE-KIND-LACKS-RUST-PRIMI-43D4589C
title: "self-host type kind lacks Rust primitive parity"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-29
target: stdlib/neplg2/core/ty/ty.nepl
---

# ISS-20260428T212439976Z-SELF-HOST-TYPE-KIND-LACKS-RUST-PRIMI-43D4589C: self-host type kind lacks Rust primitive parity

## 概要

SelfhostTypeKind currently covers Unit/Bool/I32/I64/U8/Char/Str/Function, but the Rust type context and parser expose F32 and Never as first-class primitive kinds and handle i64/f64 as named numeric types. The self-host type layer therefore cannot model the full primitive surface used by current NEPLg2 signatures.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl`

## 根拠

- Rust 側 `TypeExpr` は `F32` と `Never` を専用 variant として持ち、`type_from_expr` も `ctx.f32()` / `ctx.never()` に正規化している。
- Rust parser は `i64` と `f64` を named type として扱い、WASM/LLVM/layout では numeric primitive として特別扱いする。
- self-host 側の `SelfhostTypeKind` と primitive registry は `F32` / `F64` / `Never` を持っておらず、後続 checker が string name に依存するか Rust surface と違う型分類を作る状態だった。

## 問題

SelfhostTypeKind currently covers Unit/Bool/I32/I64/U8/Char/Str/Function, but the Rust type context and parser expose F32 and Never as first-class primitive kinds and handle i64/f64 as named numeric types. The self-host type layer therefore cannot model the full primitive surface used by current NEPLg2 signatures.

## 影響

A self-host checker built on the current type arena would either reject valid f32/never signatures, encode them as ad hoc named types, or diverge from Rust diagnostics and overload behavior.

## 修正方針

Extend the self-host type model with explicit Rust-parity primitive coverage, define canonical/source spellings for unit/never/floating and named numeric aliases, and add parity doctests against representative signatures.

## 修正内容

- `SelfhostTypeKind` に `F32` / `F64` / `Never` を追加し、`selfhost_type_kind_tag` の finite match に arm を追加した。
- `selfhost_type_kind_canonical_name` を追加し、`unit` / `bool` / `i32` / `i64` / `u8` / `char` / `str` / `f32` / `f64` / `never` の canonical spelling を type layer で定義した。
- `neplg2/core/builtins/prelude.nepl` の primitive registry を 10 件へ拡張し、`f32` / `f64` / `never` を typed metadata として返すようにした。
- `tests/stdlib/neplg2_type_arena.n.md` の primitive arena regression に `F32` / `F64` / `Never` の stable id と kind lookup を追加した。
- `stdlib/neplg2/README.md` の S3 Type Layer 説明を named numeric primitive 対応込みに更新した。

## 検証

- `git diff --check`: pass
- `node nodesrc/tests.js -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\selfhost-type-kind-primitives-prelude.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i stdlib\neplg2\core\ty\ty.nepl -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\selfhost-type-kind-primitives.json -j 1`: total=2 passed=1 failed=1。失敗は既知の raw-memory gate による `stdlib/alloc/collections/vec.nepl` の `free__Vec` / `push__Vec` D3100。
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\selfhost-type-kind-primitives-arena.json -j 1`: total=5 failed=5。失敗は既知の `alloc/string.nepl` `concat_result` D3100 で、追加した type kind arm の compile error ではない。
