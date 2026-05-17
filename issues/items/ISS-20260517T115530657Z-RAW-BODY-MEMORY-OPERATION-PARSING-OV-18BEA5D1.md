---
id: ISS-20260517T115530657Z-RAW-BODY-MEMORY-OPERATION-PARSING-OV-18BEA5D1
title: "Raw body memory operation parsing overmatches backend opcode strings"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/effects.rs, nepl-core/tests/effects.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T115530657Z-RAW-BODY-MEMORY-OPERATION-PARSING-OV-18BEA5D1: Raw body memory operation parsing overmatches backend opcode strings

## 概要

raw body memory effect collection classifies WASM loads/stores with substring checks and LLVM memory intrinsics with starts_with branches outside the raw body memory operation enum. This can overclassify backend text such as non-memory llvm.memcpy_like callees, and the classifier semantics are not owned by the enum that downstream gates consume.

## 対象

- `nepl-core/src/effects.rs, nepl-core/tests/effects.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `effects.rs` の `wasm_memory_op_from_opcode` は `op.contains(".load")` / `op.contains(".store")` で memory access を分類していた。
- LLVM memory intrinsic は `callee.starts_with("llvm.memcpy")` / `llvm.memmove` / `llvm.memset` で分類しており、`llvm.memcpy_like` のような別 callee を memory intrinsic と誤分類し得た。
- downstream は `RawBodyMemoryOp::{Wasm,Llvm}` enum を消費しているが、実際の opcode/callee 分類 semantics は enum 外の free function に分散していた。

## 問題

raw body memory effect collection classifies WASM loads/stores with substring checks and LLVM memory intrinsics with starts_with branches outside the raw body memory operation enum. This can overclassify backend text such as non-memory llvm.memcpy_like callees, and the classifier semantics are not owned by the enum that downstream gates consume.

## 影響

Effect safety for raw bodies depends on parser string details instead of an exhaustive typed classifier. Overclassification can grant or require the wrong raw-body memory capability, while future opcode changes can update tests without updating every direct branch.

## 修正方針

Move WASM opcode and LLVM opcode/callee classification onto WasmRawBodyMemoryOp and LlvmRawBodyMemoryOp. Use exact intrinsic-name boundary checks for LLVM memory intrinsics, keep raw_body_memory_operations as a consumer of typed classifiers, and add regressions/policy against direct substring/prefix classification in the consumer.

## 関連計画

- [静的検査の不必要な複雑化の解消についての大規模な修正の仕様と実装計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対応内容

- `WasmRawBodyMemoryOp::from_opcode` を追加し、WASM raw-body memory opcode の分類を enum 実装へ移した。
- `LlvmRawBodyMemoryOp::from_instruction_opcode` / `from_intrinsic_callee` を追加し、LLVM instruction opcode と memory intrinsic callee の分類を enum 実装へ移した。
- LLVM memory intrinsic は base name そのもの、または `.` で続く intrinsic variant だけを認める boundary check にした。
- `raw_body_memory_operations` は typed classifier method を呼ぶだけにし、consumer 側の substring / unbounded prefix branch を削除した。
- `effects` regression に `custom.loadx` / `llvm.memcpy_like` の過剰分類拒否を追加した。
- static-check responsibility policy に typed classifier method と旧 direct branch 禁止を追加した。

## 検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test effects raw_body_memory_operations_are_typed_by_backend -- --exact --nocapture`: 1/1 passed
- `cargo test -p nepl-core --test effects raw_body -- --nocapture`: 3/3 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
