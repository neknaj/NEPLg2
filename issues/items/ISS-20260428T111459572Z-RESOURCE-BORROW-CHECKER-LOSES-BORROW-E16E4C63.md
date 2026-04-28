---
id: ISS-20260428T111459572Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-E16E4C63
title: "Resource borrow checker loses borrow token returns through function values"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T111459572Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-E16E4C63: Resource borrow checker loses borrow token returns through function values

## 概要

Borrow token parameter-to-return summaries are applied to direct ResourceOp::Call only. A known function value such as let f @borrow_id; f token returns the same active borrow token through ResourceOp::IndirectCall, but the borrow checker does not track FunctionValue aliases and loses the binding.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T111052658Z-RESOURCE-BORROW-CHECKER-LOSES-BORROW-F9979F31` で direct call の borrow token return summary は追加された。
- しかし `ResourceOp::FunctionValue` と `ResourceOp::IndirectCall` は borrow checker で function alias として扱われておらず、known function value 経由の helper call で summary が適用されなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は callback / function value 境界でも borrow state を落とさない Resource IR 化を求めている。
- Resource IR には `FunctionValue` output と `IndirectCall` callee place が残っているため、known alias に対して direct call と同じ summary を適用できる。

## 問題

Borrow token parameter-to-return summaries are applied to direct ResourceOp::Call only. A known function value such as let f @borrow_id; f token returns the same active borrow token through ResourceOp::IndirectCall, but the borrow checker does not track FunctionValue aliases and loses the binding.

## 影響

Lifetime escape diagnostics can still be hidden behind first-class functions, which are expected in self-host and stdlib helper code. This keeps borrow checking dependent on syntactic direct calls.

## 修正方針

Track known function value aliases in ResourceBorrowCheckEngine, merge them through local copies and branches, and apply borrow token return summaries to known ResourceOp::IndirectCall targets.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 borrow token function value summary 対応

`ResourceBorrowCheckEngine` に known function value alias table を追加した。`ResourceOp::FunctionValue` で function name を place に紐づけ、`DeclareLocal` / `Read` / `Move` / `Assign` と branch / loop / match merge で alias を保持する。

`ResourceOp::IndirectCall` の callee が known function value alias を持つ場合は、direct call と同じ borrow token parameter-to-return summary を適用する。これにより `let f @borrow_id; f token` のような first-class function 経由でも active borrow token binding が output へ伝播し、return escape が `ResourceBorrowOperation::ReturnValue` の `BorrowConflict` になる。

`nepl-core/tests/resource_ir.rs` に function value 経由の borrow token return escape 回帰を追加した。
