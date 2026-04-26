---
id: ISS-20260426T213057295Z-LLVM-DUAL-BACKEND-MISSES-ALLOCATOR-R-EF505BD2
title: "LLVM dual backend misses allocator runtime symbols"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/codegen_llvm.rs, nepl-core/tests/neplg2.rs"
---

# ISS-20260426T213057295Z-LLVM-DUAL-BACKEND-MISSES-ALLOCATOR-R-EF505BD2: LLVM dual backend misses allocator runtime symbols

## 概要

GitHub Actions run 24967172989 llvm-dual-tests has link_llvm_cli failures such as use of undefined value @alloc_raw__i32__i32__pure across block/generic/string/span doctests.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-core/src/runtime_helpers.rs, nodesrc/tests.js`

## 根拠

- `try_lower_entry_from_hir` は `collect_hir_signatures` から `alloc_raw` / `alloc` を見つけると fallback allocator ではなく stdlib 側 helper を `resolve_alloc_symbol` で選ぶ。
- aggregate construction / string materialization / reference scalar `AddrOf` は backend 内部で allocator call を挿入するが、`PreparedLlvmProgram::reachable_set` はユーザー HIR call graph からしか作られない。
- そのため、ユーザーコードが `alloc_raw` を直接呼ばない aggregate construction では LLVM IR に allocator call だけが出力され、allocator helper 本体と `load_i32` / `store_i32` / `mem_grow` などの依存 helper が出力対象にならない。
- fallback allocator は helper signature が見つからない時だけ使われるため、`core/mem` を import して helper signature が存在するケースほど未定義 symbol が起きやすい。

## 問題

GitHub Actions run 24967172989 llvm-dual-tests has link_llvm_cli failures such as use of undefined value @alloc_raw__i32__i32__pure across block/generic/string/span doctests.

## 影響

Programs that allocate aggregates can typecheck and lower to LLVM IR but fail at link time, so LLVM backend parity is not actionable for self-host workloads.

## 修正方針

Ensure LLVM codegen emits or links the allocator/runtime helper symbols required by lowered stdlib/core intrinsics, and add a focused LLVM compile/link test for alloc_raw-backed aggregate construction.

## 検証

node nodesrc/tests.js -i tests/compiler/block_single_line.n.md --runner llvm --llvm-all --no-tree -o tmp/llvm-alloc-runtime.json -j 1 passes the alloc_raw link cases.

## 解決

- LLVM HIR lowering 用に `backend_reachable_set` を作り、stdlib allocator helper を選んだ場合はその helper を root とする HIR call graph closure を追加するようにした。
- 追加 closure には mangled 名と base alias の両方を入れ、allocator 本体だけでなく raw LLVM helper の alias emit と依存 helper emit も同じ出力に含まれるようにした。
- aggregate construction が backend 内部で allocator call を挿入するだけのケースを Rust integration test に追加し、`alloc_raw` call と `alloc_raw` 定義、依存する `load_i32` / `store_i32` が LLVM IR に含まれることを固定した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test neplg2 llvm_allocator -- --nocapture`: 1/1 passed
- `cargo test -p nepl-core --test neplg2 llvm -- --nocapture`: 5/5 passed
- `trunk build`: pass
