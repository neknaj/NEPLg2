---
id: ISS-20260426T213057658Z-LLVM-CODEGEN-REACHES-UNBOUND-LOCALS--5CA65A55
title: "LLVM codegen reaches unbound locals after lowering"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/codegen_llvm.rs, nepl-core/tests/neplg2.rs, tests/compiler/move_effect.n.md, stdlib/alloc/collections/adjacency_matrix.nepl"
---

# ISS-20260426T213057658Z-LLVM-CODEGEN-REACHES-UNBOUND-LOCALS--5CA65A55: LLVM codegen reaches unbound locals after lowering

## 概要

GitHub Actions run 24967172989 llvm-dual-stdlib reports D4102 unknown variable v reached llvm codegen for adjacency_matrix, bitset, bloom_filter, disjoint_set, fenwick, and segment_tree doctests; tests/compiler/move_effect also reports unknown variable u.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-core/src/monomorphize.rs, stdlib/alloc/collections/adjacency_matrix.nepl`

## 根拠

- `tests/compiler/move_effect.n.md::doctest#27::llvm` で `D4102 unknown variable 'u' reached llvm codegen` を再現した。
- `stdlib/alloc/collections/adjacency_matrix.nepl::doctest#6::llvm` で `D4102 unknown variable 'v' reached llvm codegen` を再現した。
- `u` は `let u <()> ()` のような zero-sized local、`v` は `Result<(), E>` を `match` した `Result::Ok v` の unit payload binding だった。
- LLVM lowering は `LlTy::Void` の local / payload binding を storage 不要として local 環境へ登録していなかったが、HIR 上は後続の `Var(u)` / `Var(v)` が lexical binding として残るため、codegen で D4102 になっていた。

## 問題

GitHub Actions run 24967172989 llvm-dual-stdlib reports D4102 unknown variable v reached llvm codegen for adjacency_matrix, bitset, bloom_filter, disjoint_set, fenwick, and segment_tree doctests; tests/compiler/move_effect also reports unknown variable u.

## 影響

A local binding can disappear between typecheck/monomorphize and LLVM codegen, which is a core correctness issue rather than a stdlib surface failure.

## 修正方針

Trace the HIR path that introduces these locals and ensure branch/match/destructure lowering preserves local definitions or reports a pre-codegen diagnostic when a binding is out of scope.

## 検証

Run llvm-dual focused tests for stdlib/alloc/collections/adjacency_matrix.nepl and tests/compiler/move_effect.n.md until no D4102 unknown variable diagnostics remain.

## 解決

- LLVM lowering の local 環境に、storage を持たない `LlTy::Void` binding を登録できるようにした。
- block predeclare で unit local も名前だけ登録し、`let` lowering の fallback でも unit local を登録するようにした。
- `Var` lowering は `LlTy::Void` binding を load せず、値を生成しない式として扱うようにした。
- `Result::Ok v` など enum match payload が unit の場合も、payload binding 名を zero-sized local として登録するようにした。
- `nepl-core/tests/neplg2.rs` に unit local reuse と `Result<(), E>` payload bind を同時に固定する LLVM regression test を追加した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test neplg2 llvm_unit_locals_and_payload_binds_remain_in_scope -- --nocapture`: 1/1 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/alloc/collections/bitset.nepl -i stdlib/alloc/collections/bloom_filter.nepl -i stdlib/alloc/collections/counting_bloom_filter.nepl -i stdlib/alloc/collections/disjoint_set.nepl -i stdlib/alloc/collections/fenwick.nepl -i stdlib/alloc/collections/segment_tree.nepl --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-unbound-collections-after-trunk.json -j 1`: total=41, passed=41
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-unbound-move-effect-after-unit-bind.json -j 1`: doctest#27 passed and no D4102 remained; existing return-value mismatch failures remain under the separate LLVM dual-runner issue.
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md -i ...collection files... --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-unbound-related-after-trunk.json -j 1`: no D4102 / unknown variable remained.
- `git diff --check`: pass（CRLF 変換警告のみ）
