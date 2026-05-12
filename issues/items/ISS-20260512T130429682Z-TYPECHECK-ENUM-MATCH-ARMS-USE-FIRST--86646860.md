---
id: ISS-20260512T130429682Z-TYPECHECK-ENUM-MATCH-ARMS-USE-FIRST--86646860
title: "Typecheck enum match arms use first namespace separator for variant names"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/match_check.rs; nepl-core/src/typecheck/syntax_helpers.rs; nepl-core/tests/import_clause.rs; nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260512T130429682Z-TYPECHECK-ENUM-MATCH-ARMS-USE-FIRST--86646860: Typecheck enum match arms use first namespace separator for variant names

## 概要

`typecheck/match_check.rs` extracts enum match arm variant names with `find("::")`, while match expected-type inference and Resource IR use the last separator / tail form.

## 対象

- `nepl-core/src/typecheck/match_check.rs`
- `nepl-core/src/typecheck/syntax_helpers.rs`
- `nepl-core/tests/import_clause.rs`
- `nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/typecheck/match_check.rs` extracted the checked arm name with `variant.name.find("::")`, so `dep::E::A` became `E::A` instead of `A`.
- `nepl-core/src/typecheck/call_reduction.rs` already uses `rfind("::")` when it infers the enum name from match arms.
- `nepl-core/src/resource/variant_name.rs` uses a tail-based helper for Resource IR enum payload place normalization and comparison.

## 問題

`typecheck/match_check.rs` had a first-separator rule while other enum match paths use the last separator / tail rule. This left type checking and Resource IR with different names for the same enum payload arm.

## 影響

Qualified enum variant patterns with an additional namespace component can be rejected as unknown variants or tracked under a different payload key, weakening type-check and Resource IR consistency for enum payload checks.

## 修正方針

Introduce a shared typecheck helper for the variant member tail, use it in enum match arm checking, add an alias-qualified enum match regression, and add a source policy test so `match_check.rs` does not reintroduce first-separator parsing.

## 対応記録

- `variant_member_tail` を `typecheck/syntax_helpers.rs` に追加し、qualified path の最後の `::` 以降を enum member 名として扱うようにした。
- `check_enum_match_expr` の duplicate / unknown / exhaustive 判定を `variant_member_tail` 経由へ統一した。
- `import_clause.rs` に `dep::E::A` / `dep::E::B` の alias-qualified enum match arm regression を追加した。
- `nodesrc/test_static_check_boundary_responsibility.js` に `match_check.rs` が `variant_member_tail` を使い、`find("::")` を再導入しない policy を追加した。

## 検証

- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo test -p nepl-core --test import_clause alias_qualified_enum_match_arm_uses_variant_member_tail -- --nocapture`: passed
- `cargo test -p nepl-core typecheck::syntax_helpers::tests::variant_member_tail_uses_last_separator -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/issues.js index --dir issues`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
