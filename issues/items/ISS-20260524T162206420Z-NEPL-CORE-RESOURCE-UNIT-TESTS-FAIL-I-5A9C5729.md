---
id: ISS-20260524T162206420Z-NEPL-CORE-RESOURCE-UNIT-TESTS-FAIL-I-5A9C5729
title: "nepl-core resource unit tests fail in current branch"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-24
updated: 2026-05-24
target: "nepl-core/src/resource/effect_return_escape_tests.rs; nepl-core/src/resource/i32_call_facts_tests.rs"
---

# ISS-20260524T162206420Z-NEPL-CORE-RESOURCE-UNIT-TESTS-FAIL-I-5A9C5729: nepl-core resource unit tests fail in current branch

## 概要

cargo test -p nepl-core currently fails in resource::effect_return_escape_tests::return_escape_protects_region_token_identity_inside_result_owner_payload, resource::effect_return_escape_tests::return_escape_treats_final_owner_carrier_payload_as_protected, resource::i32_call_facts_tests::records_i32_constant_result_for_mangled_add_call, and resource::i32_call_facts_tests::records_i32_difference_result_for_mangled_sub_call.

## 対象

- `nepl-core/src/resource/effect_return_escape_tests.rs; nepl-core/src/resource/i32_call_facts_tests.rs`

## 根拠

- `cargo test -p nepl-core` は 352 passed / 4 failed で終了した。
- 失敗は `effect_return_escape_tests` と `i32_call_facts_tests` に限られ、今回変更した parser / source policy helper とは別領域である。
- `cargo check -p nepl-core`、`cargo test -p nepl-core --test functions neplg21`、`cargo test -p nepl-core --test typeannot neplg21`、`cargo test -p nepl-core qualified_name` は通過した。

## 問題

cargo test -p nepl-core currently fails in resource::effect_return_escape_tests::return_escape_protects_region_token_identity_inside_result_owner_payload, resource::effect_return_escape_tests::return_escape_treats_final_owner_carrier_payload_as_protected, resource::i32_call_facts_tests::records_i32_constant_result_for_mangled_add_call, and resource::i32_call_facts_tests::records_i32_difference_result_for_mangled_sub_call.

## 影響

Full nepl-core unit verification is red even though focused NEPLg2.1 parser and source policy checks pass, so unrelated work cannot honestly claim a fully green cargo test baseline.

## 修正方針

Investigate the current Resource IR summary/protection changes, determine whether owner-protection projection and i32 direct-call fact propagation regressed or the tests are stale, then fix the implementation or update the tests without weakening static checks.

## 検証

Run cargo test -p nepl-core and focused cargo test -p nepl-core effect_return_escape i32_call_facts --lib -- --nocapture.
