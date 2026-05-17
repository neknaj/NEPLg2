---
id: ISS-20260517T044649993Z-RAW-BODY-DIRECT-CALLEE-PROOF-IS-RECL-FF4BD50E
title: "raw body direct callee proof is reclassified from strings by consumers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/effects.rs
---

# ISS-20260517T044649993Z-RAW-BODY-DIRECT-CALLEE-PROOF-IS-RECL-FF4BD50E: raw body direct callee proof is reclassified from strings by consumers

## 概要

raw_body_direct_callees returns raw callee strings, and source capability proof plus typecheck effect validation both call raw_memory_op_from_name on those strings. This leaves raw body memory-helper proof as duplicated string classification at each consumer instead of a typed effect/proof artifact produced by the raw body parser.

## 対象

- `nepl-core/src/effects.rs`

## 根拠

- `nepl-core/src/effects.rs` の旧 `raw_body_direct_callees` は raw body 内の `call` から callee 文字列だけを返していた。
- `nepl-core/src/source_capability/proof.rs` はその文字列に対して `raw_memory_op_from_name(&callee)` を再実行し、raw operation source proof を作っていた。
- `nepl-core/src/typecheck/effect_check.rs` も同じ callee 文字列を再分類して pure raw body の許可可否を決めていた。
- これにより raw body parser が作る証明 artifact と、それを消費する source capability / typecheck gate の間に typed contract がなく、consumer ごとの分類 drift を防げなかった。

## 問題

raw_body_direct_callees returns raw callee strings, and source capability proof plus typecheck effect validation both call raw_memory_op_from_name on those strings. This leaves raw body memory-helper proof as duplicated string classification at each consumer instead of a typed effect/proof artifact produced by the raw body parser.

## 影響

Static-check proof for raw bodies is harder to audit and easier to extend inconsistently. A future raw helper or backend call form can be accepted in one consumer and missed in another, undermining enum/match exhaustiveness and the generic proof policy.

## 修正方針

Introduce a typed RawBodyDirectCallee enum produced in effects.rs. Consumers must match RawMemory { operation, callee } versus Other(callee) instead of re-running raw_memory_op_from_name. Add source policy coverage to forbid raw_body_direct_callees plus consumer-side raw helper reclassification in source capability/typecheck.

## 検証

cargo test -p nepl-core --test effects; cargo test -p nepl-core raw_body --lib; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues

## 対応内容

- `RawBodyDirectCallee` enum を追加し、raw body parser が `RawMemory { callee, operation }` と `Other(callee)` を typed data として返すようにした。
- source capability proof は raw body direct call を文字列から再分類せず、`RawBodyDirectCallee::RawMemory` を match して raw memory operation proof を挿入するようにした。
- typecheck effect gate も同じ enum を消費し、raw memory helper call とその他 direct callee の effect 判定を exhaustive match に寄せた。
- `nodesrc/test_static_check_boundary_responsibility.js` に consumer-side `raw_body_direct_callees` / `raw_memory_op_from_name(&callee)` の再導入禁止を追加した。

## 検証結果

- `cargo test -p nepl-core --test effects raw_body -- --nocapture`: 3 passed
- `cargo test -p nepl-core raw_body --lib -- --nocapture`: 0 matched / passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
