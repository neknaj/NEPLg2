---
id: ISS-20260429T133518746Z-EXPLICIT-CLONE-CLONE-ON-MEMPTR-REMAI-8C62ED37
title: "Explicit Clone::clone on MemPtr remains unresolved after monomorphize"
area: compiler
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core, stdlib/core/traits/copy.nepl, stdlib/core/mem.nepl"
---

# ISS-20260429T133518746Z-EXPLICIT-CLONE-CLONE-ON-MEMPTR-REMAI-8C62ED37: Explicit Clone::clone on MemPtr remains unresolved after monomorphize

## 概要

While refactoring std/streamio StreamWriter, using let w_init <MemPtr<u8>> Clone::clone &w compiled through type checking but failed backend codegen with backend.codegen.trait_call_unresolved: Clone<>::clone [self=MemPtr_T_u8]. MemPtr<.T> has Clone/Copy impls in core/traits/copy, so explicit clone should resolve or fail earlier with a precise diagnostic.

## 対象

- `nepl-core, stdlib/core/traits/copy.nepl, stdlib/core/mem.nepl`

## 根拠

- 未記入

## 問題

While refactoring std/streamio StreamWriter, using let w_init <MemPtr<u8>> Clone::clone &w compiled through type checking but failed backend codegen with backend.codegen.trait_call_unresolved: Clone<>::clone [self=MemPtr_T_u8]. MemPtr<.T> has Clone/Copy impls in core/traits/copy, so explicit clone should resolve or fail earlier with a precise diagnostic.

## 影響

Stdlib code that tries to avoid moving a MemPtr owner by explicit cloning can hit a backend-only unresolved trait call. This encourages awkward raw-address workarounds and weakens confidence in Copy/Clone capability checking for self-host code.

## 修正方針

Trace trait resolution and monomorphization for explicit associated trait calls on generic type constructors such as MemPtr<.T>. Ensure Clone::clone for MemPtr<u8> resolves to the stdlib impl before backend, or produce an earlier typed diagnostic if the call form is unsupported.

## 検証

Add a focused .n.md or Rust integration test that imports core/traits/copy and calls Clone::clone on a MemPtr<u8>, then run the backend/codegen path that previously reported backend.codegen.trait_call_unresolved.

## 解決

2026-04-30:

- `monomorphize` の trait impl lookup を、typecheck と同じく impl target pattern と concrete self type の照合で行うように修正した。
- `impl<.T> Clone for MemPtr<.T>` のように impl type parameter が trait argument ではなく target type 側だけに現れる場合も、`MemPtr<u8>` から `.T = u8` を推論して impl method を具体化するようにした。
- `tests/compiler/reference_codegen.n.md` に `Clone::clone &MemPtr<u8>` が backend 前に stdlib の generic impl へ解決される回帰テストを追加した。

## 解決時の検証

- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --no-tree -o tmp/memptr-explicit-clone-after.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/tests.js -i tests/compiler/generic_impl_trait_args.n.md --no-tree -o tmp/memptr-explicit-clone-generic-impl.json -j 1 --dist web/dist`: total=2, passed=2
- `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-tree -o tmp/memptr-explicit-clone-prelude-copy.json -j 1 --dist web/dist`: total=4, passed=4
