---
id: ISS-20260517T054640694Z-IMPURE-HOST-EFFECT-MARKERS-DUPLICATE-3783BAA7
title: "impure host effect markers duplicate typed effect enums"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/effects.rs
---

# ISS-20260517T054640694Z-IMPURE-HOST-EFFECT-MARKERS-DUPLICATE-3783BAA7: impure host effect markers duplicate typed effect enums

## 概要

effects.rs still exposes IMPURE_IO_EFFECT_MARKERS as a string list for external IO and nondeterministic host effects even though ExternalIoOp and NondetOp already own the typed effect domains and as_str mappings.

## 対象

- `nepl-core/src/effects.rs`

## 根拠

- `effects.rs` は external I/O と nondeterministic host effect を `ExternalIoOp` / `NondetOp` enum と `as_str` match で既に型付き分類していた。
- その一方で、同じ spelling 群を `IMPURE_IO_EFFECT_MARKERS` という string list として別に公開していた。
- テストは marker list が typed classifier に写ることを確認していたため、enum 側を唯一の列挙元にする代わりに、重複 list と classifier の一致を後追いで検査する構造になっていた。

## 問題

effects.rs still exposes IMPURE_IO_EFFECT_MARKERS as a string list for external IO and nondeterministic host effects even though ExternalIoOp and NondetOp already own the typed effect domains and as_str mappings.

## 影響

Host effect marker strings can drift from ExternalIoOp/NondetOp. Static-check effect tests and future consumers can depend on list agreement instead of enum/match exhaustiveness, weakening maintenance of effect correctness.

## 修正方針

Remove IMPURE_IO_EFFECT_MARKERS and give ExternalIoOp and NondetOp enum-owned ALL lists. Update tests and source policy to enumerate host effect spellings through the typed enums.

## 対応内容

- `IMPURE_IO_EFFECT_MARKERS` を削除した。
- `ExternalIoOp::ALL` と `NondetOp::ALL` を追加し、host effect operation の列挙元を enum 実装側へ移した。
- `host_effect_operation_domains_round_trip_through_typed_classifiers` を追加し、`ExternalIoOp::ALL` / `NondetOp::ALL` の各 operation が対応 classifier と往復し、相互に混ざらないことを固定した。
- `nodesrc/test_static_check_boundary_responsibility.js` に source policy を追加し、旧 marker list の再導入を拒否する。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test effects host_effect -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
