---
id: ISS-20260428T112207828Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-A173C06B
title: "Resource borrow checker loses borrow token returns through unknown callbacks"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T112207828Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-A173C06B: Resource borrow checker loses borrow token returns through unknown callbacks

## 概要

ResourceBorrowCheckEngine propagates borrow token return summaries for direct calls and known function values, but an IndirectCall whose callee has no known FunctionValue alias is ignored. An unknown callback can legally return one of its arguments, so an active borrow token passed to that callback can be returned into a fresh place and escape through the caller return without ResourceBorrowOperation::ReturnValue diagnostics.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T111052658Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-F9979F31` で direct call、`ISS-20260428T111459572Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-E16E4C63` で known function value の borrow token return summary は追加された。
- しかし callback parameter や higher-order value など、`ResourceOp::IndirectCall` の callee が known `FunctionValue` alias を持たない場合は、borrow checker が return token の可能性を一切伝播していなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は function / callback 境界でも borrow state を落とさないことを要求している。
- unknown callback は保守的に「戻り値型と一致する引数を返し得る」と扱う必要がある。

## 問題

ResourceBorrowCheckEngine propagates borrow token return summaries for direct calls and known function values, but an IndirectCall whose callee has no known FunctionValue alias is ignored. An unknown callback can legally return one of its arguments, so an active borrow token passed to that callback can be returned into a fresh place and escape through the caller return without ResourceBorrowOperation::ReturnValue diagnostics.

## 影響

Borrow/lifetime checking remains dependent on knowing the exact callback target. Self-host and stdlib higher-order helper code can hide active borrow token escape behind a callback parameter, weakening the Stage 4 Resource IR lifetime boundary.

## 修正方針

For IndirectCall with no known callee aliases, conservatively treat the output as possibly any active borrow token argument. Preserve precise summaries for known aliases, and add a Resource IR regression for unknown callback-mediated token return escape.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 unknown callback borrow token return 対応

`ResourceBorrowCheckEngine::propagate_indirect_call_return_token` で、callee が known function alias を持たない場合の保守的な fallback を追加した。unknown callback は任意の同型引数を返し得るため、output と型が一致する active borrow token 引数を output へ伝播する。

known function value の場合は従来通り computed borrow token return summary を使う。これにより known callee の精度を維持しながら、unknown callback parameter 経由の borrow token return escape を塞ぐ。

`nepl-core/tests/resource_ir.rs` に unknown callback が borrow token を返す経路の検出回帰と、戻り値型が異なる場合は token として扱わない正常系を追加した。
