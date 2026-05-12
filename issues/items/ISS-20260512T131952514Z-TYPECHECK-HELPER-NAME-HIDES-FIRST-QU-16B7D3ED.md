---
id: ISS-20260512T131952514Z-TYPECHECK-HELPER-NAME-HIDES-FIRST-QU-16B7D3ED
title: "Typecheck helper name hides first-qualifier split semantics"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/syntax_helpers.rs; nepl-core/src/typecheck/*.rs; nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260512T131952514Z-TYPECHECK-HELPER-NAME-HIDES-FIRST-QU-16B7D3ED: Typecheck helper name hides first-qualifier split semantics

## 概要

`typecheck/syntax_helpers.rs` exposed `parse_variant_name` even though the helper splits at the first namespace separator and is also used for import alias and trait member lookup. The name made it easy to reuse the first-separator rule for enum variant member matching, which must use the last separator.

## 対象

- `nepl-core/src/typecheck/syntax_helpers.rs`
- `nepl-core/src/typecheck/constructor_apply.rs`
- `nepl-core/src/typecheck/name_lookup.rs`
- `nepl-core/src/typecheck/prefix_check.rs`
- `nepl-core/src/typecheck/trait_call_apply.rs`
- `nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `parse_variant_name` used `splitn(2, "::")`, so `dep::Result::Ok` is intentionally split as `dep` / `Result::Ok`.
- The same helper was imported by constructor application, qualified import lookup, trait method lookup, and prefix lookup; it was not an enum-variant-tail parser.
- `match_check.rs` required the opposite rule for arm member names and now uses `variant_member_tail`, which is based on the last separator.

## 問題

The helper name mixed two different concepts: leading qualifier splitting for name lookup and enum member tail extraction for match arms. This ambiguity already produced a typecheck / Resource IR consistency bug, so leaving the old name would keep the root cause in place.

## 影響

Future typecheck changes can reintroduce first-vs-last separator mismatches between name resolution, enum constructors, match arms, and Resource IR payload places, weakening static-check consistency around qualified enum payloads.

## 修正方針

Rename the helper to a qualifier-oriented name, keep variant member tail extraction as the separate last-separator helper, update all typecheck call sites, and add source policy coverage that rejects the old ambiguous helper name.

## 対応記録

- `parse_variant_name` を `split_qualified_name` へ改名した。
- constructor application、qualified import lookup、trait method lookup、prefix lookup の call site を `split_qualified_name` へ移行した。
- `syntax_helpers.rs` の unit test で `split_qualified_name` は first separator、`variant_member_tail` は last separator を使うことを明示した。
- `nodesrc/test_static_check_boundary_responsibility.js` に `split_qualified_name` / `variant_member_tail` の共存と旧 `parse_variant_name` 名の再導入禁止を追加した。

## 検証

- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo test -p nepl-core typecheck::syntax_helpers::tests -- --nocapture`: passed
- `cargo test -p nepl-core --test import_clause alias_qualified_enum_match_arm_uses_variant_member_tail -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/issues.js index --dir issues`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
