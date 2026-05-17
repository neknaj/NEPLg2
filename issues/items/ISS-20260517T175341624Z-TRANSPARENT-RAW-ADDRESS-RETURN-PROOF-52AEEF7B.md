---
id: ISS-20260517T175341624Z-TRANSPARENT-RAW-ADDRESS-RETURN-PROOF-52AEEF7B
title: "transparent raw address return proof accepts ordinary get calls"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-18
target: "nepl-core/src/resource/lower_raw_address_return.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T175341624Z-TRANSPARENT-RAW-ADDRESS-RETURN-PROOF-52AEEF7B: transparent raw address return proof accepts ordinary get calls

## 概要

Transparent raw-address return analysis classifies named return calls through FieldAccessorKind::from_call_base_name, so an ordinary user function named get can be treated as a compiler field projection proof.

## 対象

- `nepl-core/src/resource/lower_raw_address_return.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `lower_raw_address_return.rs` の transparent return analysis は、関数 body の return expression に現れる named call を `raw_address_source_from_return_named_call` へ流し、そこで `FieldAccessorKind::from_call_base_name` を使っていた。
- `from_call_base_name` は intrinsic spelling (`get_field`) と source member spelling (`get`) を統合していたため、`FuncRef::User("get", ...)` のような通常関数呼び出しを compiler field projection proof として扱い得た。
- これは直前に修正した Resource IR lowering / coverage の ordinary direct call proof boundary と同じ根であり、transparent summary path にだけ残っていた。

## 問題

Transparent raw-address return analysis classifies named return calls through FieldAccessorKind::from_call_base_name, so an ordinary user function named get can be treated as a compiler field projection proof.

## 影響

Resource IR can derive raw-address alias evidence from callee spelling instead of typed intrinsic/typecheck evidence, weakening memory-safety proof boundaries.

## 修正方針

Split transparent return named-call analysis so ordinary calls never consume core/field source member spelling; field projection proof must come from intrinsic/typed HIR evidence only. Add regression and source policy coverage.

## 検証

Run focused nepl-core resource tests, static check boundary policy, issue index/check, and diff check.

## 関連計画

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: raw-address return propagation は ordinary function spelling ではなく、typed intrinsic evidence と TypeCtx/compiler-memory identity proof を消費する。

## 修正内容

- `RawAddressReturnCalleeEvidence` enum を追加し、transparent return analysis 内で ordinary call と intrinsic を明示的に分離した。
- field accessor projection proof は `RawAddressReturnCalleeEvidence::Intrinsic` の branch だけで `FieldAccessorKind::from_intrinsic_name` から導出するようにした。
- `FieldAccessorKind::from_call_base_name` を削除し、source member spelling と intrinsic spelling を Resource IR consumer 側で再結合できないようにした。
- ordinary user function `get(MemPtr, str) -> i32` を返り値内で呼ぶ helper が raw-address alias proof を作らない regression test を追加した。
- source policy に `lower_raw_address_return.rs` が `from_call_base_name` を使わない検査を追加した。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core resource::lower::tests -- --nocapture`: 2 passed
- `cargo test -p nepl-core resource::lower::tests::transparent_raw_address_return_ignores_ordinary_get_call -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
