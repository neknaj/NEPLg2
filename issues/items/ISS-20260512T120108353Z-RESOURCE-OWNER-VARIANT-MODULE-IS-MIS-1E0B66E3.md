---
id: ISS-20260512T120108353Z-RESOURCE-OWNER-VARIANT-MODULE-IS-MIS-1E0B66E3
title: "Resource owner variant module is missing responsibility policy coverage"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_variant_utils.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T120108353Z-RESOURCE-OWNER-VARIANT-MODULE-IS-MIS-1E0B66E3: Resource owner variant module is missing responsibility policy coverage

## 概要

Resource IR owner variant effects are part of the memory-safety authority, but owner_variant.rs has grown past one thousand lines and is not included in the resource responsibility policy. Variant condition/source helper logic can grow without focused policy coverage.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_variant_utils.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `nepl-core/src/resource/owner_variant.rs` は enum payload の owner consumption / owner return / unreachable variant / payload condition / value condition / summary source dedupe をまとめて扱い、1174 行まで増えていた。
- 同 file は Resource IR owner checker の memory-safety authority に含まれるが、`nodesrc/test_resource_checker_responsibility.js` の必須 module / line-limit 監視対象に含まれていなかった。
- variant condition / source helper は owner check 本体の state machine と別責務であり、同じ file に置くと Stage 4 の owner/provenance 修正時に helper 追加が無制限に積み上がる。

## 問題

Resource IR owner variant effects are part of the memory-safety authority, but owner_variant.rs has grown past one thousand lines and is not included in the resource responsibility policy. Variant condition/source helper logic can grow without focused policy coverage.

## 影響

The Resource IR owner checker can recreate the old monolithic checker problem around enum payload owner effects, making exhaustive static-check changes harder to review and increasing the risk of missing owner/provenance regressions.

## 修正方針

Split owner variant helper logic into a dedicated module and add owner_variant modules to the source responsibility policy with explicit line limits.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo fmt --check -p nepl-core, and cargo check -p nepl-core --tests.

## 対応結果

2026-05-12 に修正済み。

- `owner_variant_utils.rs` を追加し、variant owner summary source collection、unique push helper、payload condition suffix、variant name normalization、`OwnerValueCondition` truth evaluation を `owner_variant.rs` から分離した。
- `owner_variant.rs` は pending variant owner effect state と、その state を match / resolved variant / materialization に適用する処理へ戻した。
- `resource/mod.rs` に `owner_variant_utils` を登録した。
- `nodesrc/test_resource_checker_responsibility.js` に `owner_variant.rs`、`owner_variant_utils.rs`、`owner_variant_value_condition.rs` の存在確認と行数上限を追加した。
- 分割後の行数は `owner_variant.rs` 988 行、`owner_variant_utils.rs` 207 行、`owner_variant_value_condition.rs` 200 行。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir variant_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: passed
- `node nodesrc/issues.js check --dir issues`: passed
