---
id: ISS-20260513T123242078Z-RESOURCE-OWNER-VARIANT-CALL-RECORDIN-7F8E6B04
title: "Resource owner variant call recording remains in monolithic owner_variant module"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_variant_record.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260513T123242078Z-RESOURCE-OWNER-VARIANT-CALL-RECORDIN-7F8E6B04: Resource owner variant call recording remains in monolithic owner_variant module

## 概要

Resource owner variant effect handling keeps call-summary recording, match application, summary collection, result lifecycle, and reserved-source checks in owner_variant.rs. The module remains over 1200 lines, so Resource IR variant correctness work can keep accumulating unrelated logic in one file.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_variant_record.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `nepl-core/src/resource/owner_variant.rs` が 1227 行あり、call summary から pending variant owner effect を構築する処理、match arm 適用、result materialization、summary 逆変換、dedup/merge、reserved source 検査を同じ module に保持していた。
- `nodesrc/test_resource_checker_responsibility.js` は直前まで `owner_variant.rs` の line limit を 1250 行へ引き上げる必要があり、Resource IR variant owner correctness の将来変更が単一 module へ再集中する状態だった。

## 問題

Resource owner variant effect handling keeps call-summary recording, match application, summary collection, result lifecycle, and reserved-source checks in owner_variant.rs. The module remains over 1200 lines, so Resource IR variant correctness work can keep accumulating unrelated logic in one file.

## 影響

Static-check correctness becomes harder to audit because variant owner effects mix source-summary interpretation with state application. Future changes to Result/enum owner semantics can accidentally weaken consumption or return tracking while touching unrelated match-application code.

## 修正方針

Split the call-summary recording path into a dedicated owner_variant_record.rs module, keep enum/match-based pending effect construction exhaustive, and lower owner_variant.rs responsibility line limits so the source policy catches future growth.

## 検証

Run cargo check -p nepl-core --tests, focused Resource IR variant/owner tests if present, node nodesrc/test_resource_checker_responsibility.js, node nodesrc/issues.js check --dir issues, and git diff --check.

## 2026-05-13 修正

Resource owner variant effect の call-summary recording を `owner_variant_record.rs` へ分離した。

- `OwnerReturnSummary` から pending variant owner consumption / return / payload condition / unreachable variant を作る処理を dedicated module に移した。
- `owner_variant.rs` は pending effect model と match/result application、summary collection、merge/reservation の責務に縮小した。
- `nodesrc/test_resource_checker_responsibility.js` に `owner_variant_record.rs` を mandatory module / `mod` declaration / line limit 監視として追加し、`owner_variant.rs` の limit を 1250 から 1120 へ下げた。

検証:

- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
