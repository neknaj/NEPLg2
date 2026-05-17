---
id: ISS-20260517T045756082Z-RAW-BODY-BACKEND-INTRINSIC-PURITY-IS-C82FFD7B
title: "raw body backend intrinsic purity is hard-coded in typecheck"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/typecheck/effect_check.rs
---

# ISS-20260517T045756082Z-RAW-BODY-BACKEND-INTRINSIC-PURITY-IS-C82FFD7B: raw body backend intrinsic purity is hard-coded in typecheck

## 概要

After raw body direct callees were typed, typecheck/effect_check.rs still treats any direct callee whose name starts with llvm. as pure through a consumer-side string prefix check. The raw body parser, not the typecheck consumer, should classify backend intrinsic callees.

## 対象

- `nepl-core/src/typecheck/effect_check.rs`

## 根拠

- `nepl-core/src/typecheck/effect_check.rs` の `raw_callee_is_impure` は、direct callee 文字列が `llvm.` で始まる場合だけ pure として扱っていた。
- raw body direct callee は `RawBodyDirectCallee` で typed artifact 化済みだったが、backend intrinsic の分類だけは typecheck consumer 側の string prefix 特例として残っていた。
- `nepl-core/src/effects.rs` は raw body backend を知っているため、backend intrinsic かどうかの分類は parser 側で行える。

## 問題

After raw body direct callees were typed, typecheck/effect_check.rs still treats any direct callee whose name starts with llvm. as pure through a consumer-side string prefix check. The raw body parser, not the typecheck consumer, should classify backend intrinsic callees.

## 影響

Raw body effect validation still contains a backend-specific string exception outside the typed proof artifact. This makes static-check behavior harder to audit and can drift from raw body memory-operation parsing when LLVM intrinsic call forms change.

## 修正方針

Extend RawBodyDirectCallee with a typed backend intrinsic variant produced by effects.rs for LLVM raw bodies. Typecheck must match that variant instead of checking `callee.starts_with("llvm.")`. Add regression/policy coverage so the prefix exception cannot be reintroduced in the typecheck gate.

## 検証

cargo test -p nepl-core --test effects raw_body -- --nocapture; cargo check -p nepl-core; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues

## 対応内容

- `RawBodyBackend` enum と `RawBodyDirectCallee::BackendIntrinsic { callee, backend }` を追加した。
- `effects.rs` が LLVM raw body 内の `llvm.*` direct callee を backend intrinsic として typed 分類するようにした。
- `typecheck/effect_check.rs` から `callee.starts_with("llvm.")` の特例を削除し、`RawBodyDirectCallee::BackendIntrinsic` match arm で扱うようにした。
- source policy に typecheck gate での `starts_with("llvm.")` 再導入禁止を追加した。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test effects raw_body -- --nocapture`: 3 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
