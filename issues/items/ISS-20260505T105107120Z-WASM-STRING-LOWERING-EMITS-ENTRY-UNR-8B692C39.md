---
id: ISS-20260505T105107120Z-WASM-STRING-LOWERING-EMITS-ENTRY-UNR-8B692C39
title: "WASM string lowering emits entry-unreachable literals"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/codegen_wasm.rs, nepl-core/tests/codegen_diagnostics.rs"
---

# ISS-20260505T105107120Z-WASM-STRING-LOWERING-EMITS-ENTRY-UNR-8B692C39: WASM string lowering emits entry-unreachable literals

## 概要

WASM codegen lowers every module string literal into the data section before applying function reachability. Large imported selfhost/stdlib modules can therefore emit diagnostic/parser strings from entry-unreachable functions even after user function lowering is filtered.

## 対象

- `nepl-core/src/codegen_wasm.rs, nepl-core/tests/codegen_diagnostics.rs`

## 根拠

- `nepl-core/src/codegen_wasm.rs` は `generate_wasm` の冒頭で `lower_strings(&module.string_literals)` を実行しており、function reachability を計算する前に全 literal を data segment 化していた。
- user function lowering は `collect_reachable_wasm_functions` で entry-reachable function に絞っているため、文字列 lowering だけが到達解析から外れていた。
- selfhost parser / diagnostics / stdlib import graph は文字列 literal が多く、未到達関数の文字列まで wasm data section に入れると emit 時間と wasm サイズが imported module size に引きずられる。

## 問題

WASM codegen lowers every module string literal into the data section before applying function reachability. Large imported selfhost/stdlib modules can therefore emit diagnostic/parser strings from entry-unreachable functions even after user function lowering is filtered.

## 影響

WASM emit time, data section size, and validation/runtime load cost grow with imported module size instead of the entry-reachable program. This keeps selfhost driver fixtures expensive and weakens the effect of reachability fixes.

## 修正方針

Collect string literal ids referenced by reachable WASM functions and only emit those data segments, while preserving original literal ids for field selector lookup and diagnostics.

## 検証

Add a regression where main references a live string and an entry-unreachable function references a marker string; generated wasm bytes must contain the live string and omit the unreachable marker.

## 対応内容

- `collect_reachable_string_literal_ids` を追加し、WASM 到達関数の HIR body に現れる `LiteralStr` id だけを収集するようにした。
- `StringLower.offsets` は元の literal id を維持する `Vec<Option<u32>>` に変更し、未使用 literal は data segment を持たないが field selector lookup 用の `values` は従来通り保持するようにした。
- `lower_strings` は reachable literal id だけを data section に配置する。到達関数が参照する literal id が存在しない場合は従来通り `backend.wasm.string_literal_not_found` を返す。

## 検証結果

- `cargo test -p nepl-core --test codegen_diagnostics wasm_codegen_omits_entry_unreachable_string_literal_data -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics wasm_codegen_reports_missing_string_literal_without_panicking -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: 13 passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed

## 残件

この修正後も `tests/stdlib/selfhost_cli_driver.n.md::doctest#2` 相当 source の native wasm emit は 180 秒 timeout のまま残る。未到達文字列の data section 肥大は解消したが、selfhost driver timeout の主因は `ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C` で継続する。
