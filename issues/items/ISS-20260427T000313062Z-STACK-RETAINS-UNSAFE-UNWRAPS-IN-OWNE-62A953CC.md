---
id: ISS-20260427T000313062Z-STACK-RETAINS-UNSAFE-UNWRAPS-IN-OWNE-62A953CC
title: "Stack retains unsafe unwraps in owned buffer cleanup"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/stack.nepl, tests/stdlib/stack_collections.n.md, nodesrc/test_stdlib_stack_no_unsafe_unwraps.js"
---

# ISS-20260427T000313062Z-STACK-RETAINS-UNSAFE-UNWRAPS-IN-OWNE-62A953CC: Stack retains unsafe unwraps in owned buffer cleanup

## 概要

Stack.free uses uwok on dealloc_ptr for owned data/header storage, and related allocation cleanup still relies on checked cleanup paths.

## 対象

- `stdlib/alloc/collections/stack.nepl, tests/stdlib/stack_collections.n.md`

## 根拠

- `Stack.new` は 12 byte の header と既定 capacity 分の data buffer を確保し、`Stack` owner が両方を単独所有する。
- data allocation failure 時の header cleanup と、`free` の data/header cleanup は owner invariant の内側の処理である。
- しかし実装は `dealloc_ptr` の checked cleanup に依存し、`free` では `uwok` で unwrap していたため、parser/evaluator stack の通常 cleanup が unsafe helper trap に戻り得た。

## 問題

Stack.free uses uwok on dealloc_ptr for owned data/header storage, and related allocation cleanup still relies on checked cleanup paths.

## 影響

Parser/evaluator stacks for self-host can trap during cleanup and remain inconsistent with the safer Queue/Deque owner-invariant pattern.

## 修正方針

Replace owned data/header cleanup with dealloc_raw, audit allocation-failure cleanup, add free/grow regressions, and guard implementation code against unsafe unwrap helpers.

## 解決内容

- `new` の data allocation failure cleanup を `dealloc_raw mem_ptr_addr header 12` に変更した。
- `free` の owned data/header cleanup を `dealloc_raw` に変更し、doc comment も owner invariant に合わせて更新した。
- capacity grow 後の `clear` / `free` と再確保を確認する regression を追加した。
- Stack 実装に unsafe unwrap / checked deallocation が戻らない source policy guard を追加した。

## 検証

- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-tree -o tmp/stack-owned-cleanup-docs.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib/stack_collections.n.md -i stdlib/tests/stack.n.md --no-tree -o tmp/stack-owned-cleanup-focused.json -j 1`: 18/18 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-stack-owned-cleanup.json -j 4`: 296/296 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-stack-owned-cleanup.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
