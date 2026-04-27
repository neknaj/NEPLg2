---
id: ISS-20260426T213058421Z-LLVM-TARGET-TYPECHECKS-STDLIB-COLLEC-DD7EF927
title: "LLVM target typechecks stdlib collection modules with signature drift"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/typecheck.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/binary_heap.nepl, stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/deque.nepl, stdlib/alloc/collections/list.nepl"
---

# ISS-20260426T213058421Z-LLVM-TARGET-TYPECHECKS-STDLIB-COLLEC-DD7EF927: LLVM target typechecks stdlib collection modules with signature drift

## 概要

GitHub Actions run 24967172989 llvm-dual-stdlib reports D3003 return type mismatch, D3005 ambiguous overload, and D3016 stack leftovers across binary_heap, btreemap/btreeset, deque, list, queue, ringbuffer, and related collection doctests only in LLVM lowering mode.

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/binary_heap.nepl, stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/deque.nepl, stdlib/alloc/collections/list.nepl`

## 根拠

- 現在の `main` では、対象collection群のLLVM compile-only focused runでIssue記載の D3003 / D3005 / D3016 が再現しない。
- `binary_heap.nepl`, `btreemap.nepl`, `deque.nepl`, `list.nepl`, `queue.nepl`, `ringbuffer.nepl` をまとめて `--runner llvm --llvm-all --llvm-compile-only` で実行し、55件すべてpassした。
- 直近のLLVM lowering修正で `SourceMap` を保持した qualified import alias 解決、明示型引数のraw/resolved TypeId置換、zero-sized local bindingが修正済みであり、Issue記載の「LLVM lowering modeだけ型面がずれる」症状は現行状態では観測されない。

## 問題

GitHub Actions run 24967172989 llvm-dual-stdlib reports D3003 return type mismatch, D3005 ambiguous overload, and D3016 stack leftovers across binary_heap, btreemap/btreeset, deque, list, queue, ringbuffer, and related collection doctests only in LLVM lowering mode.

## 影響

WASM stdlib doctests can pass while the LLVM target sees a different typed surface, so backend parity for self-host collections is not established.

## 修正方針

Identify whether LLVM target cfg exposes different overload sets or whether lowering re-typechecks already checked modules with altered expectations; make target-specific imports and generic constraints deterministic across WASM and LLVM.

## 検証

Run llvm-dual-stdlib focused collection doctests and confirm D3003/D3005/D3016 collection signature drift is gone.

## 解決

- このissue単体での追加コード修正は不要だった。
- 先行修正済みのLLVM target lowering / typecheck経路と、別agentによるstdlib collection更新を取り込んだ現行mainで再検証し、Issue記載のcollection signature driftが解消していることを確認した。
- 残るLLVM runtime return / compile_fail diagnostic系の失敗は、それぞれ別issueで追跡する。

## 修正後検証

- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/deque.nepl -i stdlib/alloc/collections/list.nepl -i stdlib/alloc/collections/queue.nepl -i stdlib/alloc/collections/ringbuffer.nepl --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-collection-signature-drift-repro.json -j 1`: total=55, passed=55
