---
id: ISS-20260512T134505293Z-RAW-BODY-MEMORY-OPERATIONS-REMAIN-ST-0EE9AFB4
title: "Raw body memory operations remain stringly typed"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/effects.rs; nepl-core/src/typecheck/effect_check.rs; nepl-core/tests/effects.rs"
---

# ISS-20260512T134505293Z-RAW-BODY-MEMORY-OPERATIONS-REMAIN-ST-0EE9AFB4: Raw body memory operations remain stringly typed

## 概要

raw_body_memory_operations returns Vec<String>, so raw Wasm/LLVM memory instruction classification bypasses the RawMemoryOp enum discipline and cannot be exhaustively matched by backend kind.

## 対象

- `nepl-core/src/effects.rs; nepl-core/src/typecheck/effect_check.rs; nepl-core/tests/effects.rs`

## 根拠

- `nepl-core/src/effects.rs` の `raw_body_memory_operations` は `Vec<String>` を返していた。
- Wasm raw body の `i32.load` / `i64.store` / `memory.grow` と LLVM raw body の `load` / `store` / `llvm.memcpy.*` が、backend 種別や operation 種別を型で保持せず診断表示用文字列へ直接落ちていた。
- Stage 5 の raw memory effect は `RawMemoryOp` / `ExternalIoOp` / `NondetOp` へ enum-first に移行済みだが、raw body instruction scan だけが文字列分類として残っていた。

## 問題

raw_body_memory_operations returns Vec<String>, so raw Wasm/LLVM memory instruction classification bypasses the RawMemoryOp enum discipline and cannot be exhaustively matched by backend kind.

## 影響

Effect diagnostics and raw-memory boundary review can regress to ad hoc string handling even though Stage 5 migrated raw memory effects to typed enums.

## 修正方針

Introduce typed RawBodyMemoryOp backend enums for Wasm and LLVM raw body instructions, return those from raw_body_memory_operations, and update diagnostics/tests to consume the enum through exhaustive match.

## 対応記録

- `RawBodyMemoryOp`、`WasmRawBodyMemoryOp`、`LlvmRawBodyMemoryOp` を追加した。
- `raw_body_memory_operations` は `Vec<RawBodyMemoryOp>` を返すようにし、Wasm / LLVM の raw body memory instruction を backend 別 enum で分類するようにした。
- `typecheck/effect_check.rs` の診断は enum の `as_str` を表示に使うだけにし、検査判断は typed operation collection に依存するようにした。
- `nepl-core/tests/effects.rs` に Wasm / LLVM raw body memory operation が typed enum として返る regression を追加した。
- `nodesrc/test_static_check_boundary_responsibility.js` に、`Vec<String>` / `Option<String>` 版の raw body memory operation scan を再導入しない source policy を追加した。

## 検証

- `cargo test -p nepl-core --test effects raw_body_memory_operations_are_typed_by_backend -- --nocapture`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
