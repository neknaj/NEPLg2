---
id: ISS-20260515T073602740Z-STACKPOP-OWNER-RESULT-LACKS-PUBLIC-A-58C543F4
title: "StackPop owner result lacks public accessors after owner aggregate field gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/alloc/collections/stack/api.nepl, examples/bf.nepl, examples/rpn.nepl, examples/rpn_legacy.nepl, stdlib/tests/stack.n.md, tests/stdlib/stack_collections.n.md, nodesrc/test_stdlib_stack_no_unsafe_unwraps.js"
---

# ISS-20260515T073602740Z-STACKPOP-OWNER-RESULT-LACKS-PUBLIC-A-58C543F4: StackPop owner result lacks public accessors after owner aggregate field gate

## 概要

owner-backed aggregate field projection is now correctly rejected outside compiler-owned stdlib implementation sources, but StackPop<T> only exposes its stack and item through struct fields. Examples and stack doctests therefore directly call field::get / field::get_ref on StackPop<T>, producing type.owner_aggregate.field_access_restricted instead of using a stable public API.

## 対象

- `stdlib/alloc/collections/stack/api.nepl, examples/bf.nepl, examples/rpn.nepl, examples/rpn_legacy.nepl, stdlib/tests/stack.n.md, tests/stdlib/stack_collections.n.md, nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`

## 根拠

- GitHub Actions examples job で `examples/bf.nepl` / `examples/rpn.nepl` / `examples/rpn_legacy.nepl` が `type.owner_aggregate.field_access_restricted` により compile failure になった。
- 失敗箇所はいずれも `StackPop<i32>` に対する `field::get_ref ... "item"` と `field::get ... "stack"` であり、強化後の owner aggregate field gate では利用側からの直接 projection を拒否するのが正しい。
- `StackPop<T>` は更新後 `Stack<T>` owner を含むため、compiler gate を緩めるのではなく stack module の実装境界に意図的な accessor を置く必要がある。

## 問題

owner-backed aggregate field projection is now correctly rejected outside compiler-owned stdlib implementation sources, but StackPop<T> only exposes its stack and item through struct fields. Examples and stack doctests therefore directly call field::get / field::get_ref on StackPop<T>, producing type.owner_aggregate.field_access_restricted instead of using a stable public API.

## 影響

GitHub Actions examples bf/rpn/rpn_legacy fail under the strengthened static check, and user code is pushed toward forbidden field projection on an owner-preserving result type. Weakening the compiler gate would reopen owner extraction paths; the root fix is to expose intentional StackPop destructors/observers from the stack module.

## 修正方針

Add stack_pop_item(&StackPop<T>) -> Option<T> and stack_pop_stack(StackPop<T>) -> Stack<T> accessors inside the stack implementation boundary, update Stack.pop and all StackPop callers in examples/doctests to use them, and add source policy checks forbidding direct StackPop field projection outside the implementation module.

## 検証

Run stack source policy, focused stack doctests, affected examples bf/rpn/rpn_legacy through nodesrc/tests.js, issues check, and trunk build.

## 修正内容

- `stdlib/alloc/collections/stack/api.nepl` に `stack_pop_item(&StackPop<T>) -> Option<T>` と `stack_pop_stack(StackPop<T>) -> Stack<T>` を追加した。
- `pop` 自体も `StackPop` field を直接読むのではなく、同じ accessor を通して `item` を読み、返却 stack owner を解放する形へ揃えた。
- `examples/bf.nepl` / `examples/rpn.nepl` / `examples/rpn_legacy.nepl` と Stack doctest fixture は `StackPop` field projection をやめ、公開 accessor だけを使うようにした。
- `nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` に、accessor の存在と examples/doctests での直接 `StackPop` field projection 禁止を追加した。

## 回帰テスト

- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`: passed。
- `trunk build`: passed。
- `node nodesrc/tests.js -i examples/bf.nepl -i examples/rpn.nepl -i examples/rpn_legacy.nepl --no-tree -o tmp/agent1-stack-pop-accessors-examples.json -j 1 --dist web/dist`: total=5, passed=5。
- `node nodesrc/run_test.js` へ渡した最小 StackPop accessor smoke: passed, compile_ms 約 10120ms。
- `node nodesrc/issues.js check`: passed。

## 残リスク

- `stdlib/tests/stack.n.md` / `tests/stdlib/stack_collections.n.md` の `std/test` ベース doctest は compile phase timeout により focused suite が完走しなかった。この timeout は `ISS-20260515T080145702Z-STACK-STD-TEST-DOCTESTS-EXCEED-WASM--4870E145` として分離し、compiler/static-check の計算量問題として扱う。
