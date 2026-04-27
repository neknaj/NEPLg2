---
id: ISS-20260427T182409751Z-PURE-RAW-BODIES-CAN-CALL-RAW-MEMORY--7C283F24
title: "pure raw bodies can call raw memory helper wrappers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, tests/compiler/raw_body_precheck.n.md"
---

# ISS-20260427T182409751Z-PURE-RAW-BODIES-CAN-CALL-RAW-MEMORY--7C283F24: pure raw bodies can call raw memory helper wrappers

## 概要

pure raw body validation checks direct memory instructions and declared callee effects, but raw memory helper wrappers such as load_i32/store_i32/mem_grow are still declared Pure in stdlib/core/mem.nepl; a user raw body can call those helpers and keep a Pure surface effect.

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, tests/compiler/raw_body_precheck.n.md`

## 根拠

- `nepl-core/src/typecheck.rs` の `validate_raw_body_effect` は raw body の direct memory instruction と direct callee の宣言 effect を検査する。
- `stdlib/core/mem.nepl` の `load_i32` / `store_i32` / `mem_grow` などは移行中の public raw helper として Pure signature のまま残っている。
- そのため、user raw body が `call $store_i32` のように helper を呼ぶと、direct instruction ではなく Pure callee として扱われる。

## 問題

pure raw body validation checks direct memory instructions and declared callee effects, but raw memory helper wrappers such as load_i32/store_i32/mem_grow are still declared Pure in stdlib/core/mem.nepl; a user raw body can call those helpers and keep a Pure surface effect.

## 影響

Pure functions can hide memory mutation or memory observation behind a raw body call to a helper wrapper, bypassing the raw instruction validation and weakening effect, borrow, and memory-safety assumptions.

## 修正方針

Classify known compiler raw memory helper symbols as raw memory effects during raw body callee validation, independently of their current stdlib signature, and add compile_fail regression tests.

## 検証

tests/compiler/raw_body_precheck.n.md should reject pure raw bodies that call store_i32/load_i32/mem_grow helper wrappers with D3025.

## 対応結果

- `nepl-core/src/effects.rs` に raw memory helper symbol の分類を追加した。
- `nepl-core/src/typecheck.rs` の pure raw body validation で、`call` 先が raw memory helper の場合は通常 source では `D3025` として拒否するようにした。
- 移行中の `core/mem.nepl` など compiler-owned raw memory boundary では、従来通り raw helper call を許可する。
- `nepl-core/tests/effects.rs` と `tests/compiler/raw_body_precheck.n.md` に wasm / LLVM raw helper call の compile_fail 回帰を追加した。

## 実施した検証

- `cargo test -p nepl-core --test effects -- --nocapture`: `19 passed`
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/raw_body_precheck.n.md --no-tree -o tmp/raw-helper-callee-effect-node.json -j 1`: `total=7`, `passed=7`
- `node nodesrc/tests.js -i tests/compiler/raw_body_precheck.n.md --runner llvm --no-tree -o tmp/raw-helper-callee-effect-llvm-node.json -j 1`: local `clang --version` が見つからず未完了。Rust 側の LLVM raw body effect テストで代替し、full runner は GitHub Actions で確認する。
