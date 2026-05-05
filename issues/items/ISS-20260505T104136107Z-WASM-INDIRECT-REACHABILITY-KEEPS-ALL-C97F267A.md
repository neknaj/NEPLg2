---
id: ISS-20260505T104136107Z-WASM-INDIRECT-REACHABILITY-KEEPS-ALL-C97F267A
title: "WASM indirect reachability keeps all monomorphized functions"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/wasm_shared.rs, nepl-core/tests/codegen_diagnostics.rs"
---

# ISS-20260505T104136107Z-WASM-INDIRECT-REACHABILITY-KEEPS-ALL-C97F267A: WASM indirect reachability keeps all monomorphized functions

## 概要

WASM reachability analysis returns the full monomorphized function set whenever any reachable function contains call_indirect. Selfhost compiler fixtures use function values, so one indirect call makes codegen/precheck revisit unrelated specialized functions and can turn otherwise bounded entry programs into large backend workloads.

## 対象

- `nepl-core/src/wasm_shared.rs, nepl-core/tests/codegen_diagnostics.rs`

## 根拠

- `nepl-core/src/wasm_shared.rs` の `collect_reachable_wasm_functions` は、reachable expression 内に `CallIndirect` が 1 つでもあると `all_names` を返していた。
- 同じ collector は `FnValue` / function-valued `Var` / direct user call をすでに収集していたため、`call_indirect` の存在だけを理由に entry-unreachable 関数まで backend 対象へ戻す必要はなかった。
- `wasm_codegen_keeps_indirect_address_taken_callee_without_all_functions_fallback` で、`main` が `FnValue("callee")` 経由の indirect call を持ち、同じ module に entry-unreachable な unsupported function があっても precheck/codegen が成功することを固定した。

## 問題

WASM reachability analysis returns the full monomorphized function set whenever any reachable function contains call_indirect. Selfhost compiler fixtures use function values, so one indirect call makes codegen/precheck revisit unrelated specialized functions and can turn otherwise bounded entry programs into large backend workloads.

## 影響

Entry-reachable codegen filtering is defeated for higher-order code. Unsupported or expensive functions that are not actually address-taken can still affect wasm precheck/codegen, increasing compile time and hiding true entry reachability.

## 修正方針

Track direct calls and explicit function-value references as reachable roots, and keep indirect calls precise by retaining only functions whose address is actually observed in reachable expressions instead of falling back to all functions.

## 検証

Add wasm codegen regression where main performs an indirect call through an address-taken callee while an unreachable unsupported function is present; precheck and wasm generation must ignore the unreachable function.

## 対応内容

- `collect_reachable_wasm_functions` から `has_indirect -> all_names` fallback を削除した。
- `CallIndirect` 自体は callee expression と arguments を走査し、`FnValue` / direct call / function-valued expression に現れる具体的な function symbol を通常の reachable root として扱う。
- indirect call の signature diagnostics は `collect_wasm_signature_set` の reachable set に基づくまま維持し、unknown function-valued parameter の `backend.wasm.indirect_signature_missing` は既存 regression で確認した。

## 検証結果

- `cargo test -p nepl-core --test codegen_diagnostics wasm_codegen_keeps_indirect_address_taken_callee_without_all_functions_fallback -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics wasm_precheck_reports_indirect_signature_missing_code -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics wasm_codegen_reports_indirect_signature_missing_without_panicking -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test functions function_first_class -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test effects pure_indirect_impure_function_value_is_rejected -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed

## 残件

この修正後も `tests/stdlib/selfhost_cli_driver.n.md::doctest#2` 相当 source の native wasm emit は 180 秒 timeout のまま残る。`call_indirect` の全関数 fallback は解消したが、selfhost parser / pipeline を含む monomorphize と Resource IR/backend work の規模問題は `ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C` で継続する。
