---
id: ISS-20260512T121408373Z-RESOURCE-VARIANT-NAME-NORMALIZATION--30049A9F
title: "Resource variant name normalization is duplicated across static checks"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/owner_variant_utils.rs, nepl-core/src/resource/owner_summary_variant_*.rs"
---

# ISS-20260512T121408373Z-RESOURCE-VARIANT-NAME-NORMALIZATION--30049A9F: Resource variant name normalization is duplicated across static checks

## 概要

Resource IR initialized, owner, and owner-summary variant logic each implements its own variant name normalization and match-pattern extraction. These checks must select the same enum payload path, but duplicated string normalization can diverge.

## 対象

- `nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/owner_variant_utils.rs, nepl-core/src/resource/owner_summary_variant_*.rs`

## 根拠

- `initialized_variant.rs`、`owner_variant_utils.rs`、`owner_summary_variant_construct.rs`、`owner_summary_variant_return.rs`、`owner_summary_resolved_variant.rs` がそれぞれ `rsplit("::")` / `split("::")` 相当の variant 正規化を持っていた。
- initialized check と owner check は同じ `ResourceMatchPattern::Variant` と enum payload projection を見て、到達可能 arm、payload initialized state、owner transfer を判断する必要がある。
- variant 名処理が module ごとに分かれると、qualified variant name の扱いが片方だけ変更され、cell state と owner state が別 arm を選択する設計回帰を source policy で検出できない。

## 問題

Resource IR initialized, owner, and owner-summary variant logic each implements its own variant name normalization and match-pattern extraction. These checks must select the same enum payload path, but duplicated string normalization can diverge.

## 影響

If owner and initialized checks normalize variant names differently, match-arm reachability, payload owner transfer, and raw cell initialization summaries can disagree, weakening static-check correctness around enum payload memory safety.

## 修正方針

Introduce one Resource IR variant-name utility module and migrate initialized, owner, and owner-summary variant paths to use it.

## 検証

Run Resource IR variant owner/payload focused tests, node nodesrc/test_resource_checker_responsibility.js, cargo fmt --check -p nepl-core, and cargo check -p nepl-core --tests.

## 対応結果

2026-05-12 に修正済み。

- `variant_name.rs` を追加し、`normalize_variant_name` と `match_pattern_variant_name` を Resource IR 共通 utility にした。
- `initialized_variant.rs` と initialized summary variant modules は、共通 `normalize_variant_name` / `match_pattern_variant_name` を使うようにした。
- owner variant effect、owner summary variant condition / payload condition / return / resolved parameter variant も同じ utility を使うようにした。
- `owner_summary_variant_construct.rs` と `owner_summary_variant_return.rs` の個別 normalization 実装を削除した。
- `nodesrc/test_resource_checker_responsibility.js` に `variant_name.rs` を登録し、共有 utility の存在と行数上限を監視する。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir variant_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_result_payload_raw_address_field -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
