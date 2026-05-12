---
id: ISS-20260512T062230660Z-RESOURCE-OWNER-SUMMARY-VARIANT-CONDI-F79EFC3E
title: "Resource owner summary variant conditions exceeds split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/owner_summary_variant_conditions.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T062230660Z-RESOURCE-OWNER-SUMMARY-VARIANT-CONDI-F79EFC3E: Resource owner summary variant conditions exceeds split limit

## 概要

`owner_check` utility 分割後、`nodesrc/test_resource_checker_responsibility.js` は次の blocker として `owner_summary_variant_conditions.rs has 295 lines; responsibility split limit is 260` を報告した。branch condition conversion と payload/value condition handling が 1 module に再集中していた。

## 対象

- `nepl-core/src/resource/owner_summary_variant_conditions.rs`
- `nepl-core/src/resource/owner_summary_variant_payload_conditions.rs`
- `nepl-core/src/resource/owner_summary_variant_i32_conditions.rs`
- `nepl-core/src/resource/owner_summary_variant_paths.rs`
- `nepl-core/src/resource/mod.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_summary_variant_conditions.rs has 295 lines; responsibility split limit is 260` で失敗した。
- `owner_summary_variant_conditions.rs` は value condition と payload condition の両方を収集しており、payload leaf traversal と shared i32 condition set が value condition 変換と同居していた。

## 問題

value condition 変換、payload leaf traversal、known i32 condition enumeration が 1 file に混在し、owner variant summary の条件変換境界が肥大化していた。

## 影響

Owner variant summary condition handling is accumulating multiple condition-conversion responsibilities in one file, making the memory-safety summary logic harder to audit and weakening the responsibility split policy.

## 修正方針

上限は緩めない。payload condition collection と shared i32 condition set を別 module に分離し、`owner_summary_variant_conditions.rs` は value condition 変換と projection source extension に集中させる。

## 修正

- `owner_summary_variant_i32_conditions.rs` を追加し、`SUMMARY_I32_CONDITIONS` を所有させた。
- `owner_summary_variant_payload_conditions.rs` を追加し、payload condition collection と input/output payload leaf matching を所有させた。
- `owner_summary_variant_conditions.rs` は value condition 変換と `extend_owner_projection_source` を所有し、payload collector は re-export して既存 call site の責務を広げないようにした。
- `owner_summary_variant_paths.rs` は 380 行以内を維持した。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在、`mod` 宣言、line budget を追加した。
- line count は `owner_summary_variant_conditions.rs` 185、`owner_summary_variant_payload_conditions.rs` 113、`owner_summary_variant_i32_conditions.rs` 10、`owner_summary_variant_paths.rs` 379。

## 検証

- `cargo fmt -p nepl-core`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: variant condition / paths blocker は解消。次の別件として `initialized_alias.rs has 524 lines; responsibility split limit is 520` に到達したため、`ISS-20260512T063440093Z-RESOURCE-INITIALIZED-ALIAS-EXCEEDS-S-EB6E18E5` を追加した。
