---
id: ISS-20260428T111052658Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-F9979F31
title: "Resource borrow checker loses borrow token bindings returned by helpers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T111052658Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-F9979F31: Resource borrow checker loses borrow token bindings returned by helpers

## 概要

The borrow checker now rejects returning an active borrow token only when the returned place is still bound in BorrowTable. ResourceOp::Call is ignored, so a helper like borrow_id(t): t can return the token into a fresh place and the return escape check no longer recognizes it as an active borrow token.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T110657405Z-RESOURCE-BORROW-CHECKER-ALLOWS-BORRO-EC3BEA97` で active borrow token の direct return は拒否できるようになった。
- しかし `ResourceBorrowCheckEngine` は `ResourceOp::Call` を無視しており、`fn borrow_id(t): t` のような helper return で token binding が fresh output に伝播しなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は borrow / lifetime を形骸化させず、function boundary でも resource state を維持する必要がある。
- Resource IR には direct call target と argument / output place が残っているため、parameter-to-return token summary を計算できる。

## 問題

The borrow checker now rejects returning an active borrow token only when the returned place is still bound in BorrowTable. ResourceOp::Call is ignored, so a helper like borrow_id(t): t can return the token into a fresh place and the return escape check no longer recognizes it as an active borrow token.

## 影響

Borrow lifetime escape diagnostics depend on local expression shape. Helper functions used by self-host lowering or stdlib wrappers can hide active borrow tokens across function boundaries, leaving Resource IR enforcement incomplete.

## 修正方針

Compute direct user function parameter-to-return borrow token summaries and propagate active token bindings from call arguments to call outputs before return checking. Keep this scoped to Resource IR borrow checking and add focused regressions.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 borrow token return summary 対応

Resource IR borrow checker に direct user function の borrow token parameter-to-return summary を追加した。summary は `fn id(t): t` のように戻り値がどの引数 token に由来し得るかを固定点で計算し、caller 側の `ResourceOp::Call` で active token binding を call output へ伝播する。

これにより、helper が active borrow token を返し、その結果を function return で外へ逃がす経路も `ResourceBorrowOperation::ReturnValue` の `BorrowConflict` になる。`nepl-core/tests/resource_ir.rs` に helper-mediated borrow token return escape の回帰を追加した。
