---
id: ISS-20260512T175900768Z-MONOMORPHIZE-TRAIT-LOOKUP-MODEL-EXCE-55B52B7E
title: "Monomorphize trait lookup model exceeds responsibility freeze"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/monomorphize.rs; nepl-core/src/monomorphize/trait_lookup.rs; nodesrc/test_parser_backend_responsibility_policy.js; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/parser_backend_responsibility_split_plan.md; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T175900768Z-MONOMORPHIZE-TRAIT-LOOKUP-MODEL-EXCE-55B52B7E: Monomorphize trait lookup model exceeds responsibility freeze

## 概要

The parser/backend responsibility policy reports nepl-core/src/monomorphize.rs at 1479 lines over the 1455-line freeze limit. Recent trait application work added monomorphize trait lookup key/model responsibility to the root monomorphize file, so the M1 trait impl index boundary is no longer enforced by module structure.

## 対象

- `nepl-core/src/monomorphize.rs; nepl-core/src/monomorphize/trait_lookup.rs; nodesrc/test_parser_backend_responsibility_policy.js; doc/neplg2/parser_backend_responsibility_split_plan.md`

## 根拠

- `node nodesrc/test_parser_backend_responsibility_policy.js` が `nepl-core/src/monomorphize.rs has 1479 lines; responsibility freeze limit is 1455` を報告した。
- `MonoTraitApplication` / `MonoTraitMethodKey` / `MonoTraitLookupKey` / `TraitImplEntry` / `TraitImplResolution` が root `monomorphize.rs` にあり、Stage M1 の trait impl index / lookup model 境界が file structure として分かれていなかった。
- 親計画: [NEPLg2 parser / backend responsibility split plan M1](../../doc/neplg2/parser_backend_responsibility_split_plan.md#M1-trait-impl-index)

## 問題

The parser/backend responsibility policy reports nepl-core/src/monomorphize.rs at 1479 lines over the 1455-line freeze limit. Recent trait application work added monomorphize trait lookup key/model responsibility to the root monomorphize file, so the M1 trait impl index boundary is no longer enforced by module structure.

## 影響

Large monomorphize.rs growth makes trait impl lookup, specialization, unresolved diagnostics, and backend-facing HIR rewriting harder to audit. It also hides future static-verification regressions because the source policy can only warn instead of identifying a dedicated module boundary.

## 修正方針

Split the monomorphize trait lookup application/key/entry/result model into nepl-core/src/monomorphize/trait_lookup.rs, keep monomorphize.rs as orchestration/specialization logic, and extend the parser/backend responsibility policy and plan to require the new module without raising the root line limit.

## 対応記録

- `nepl-core/src/monomorphize/trait_lookup.rs` を追加し、trait lookup の key/model 型を root `monomorphize.rs` から分離した。
- root `monomorphize.rs` は `mod trait_lookup;` で model を参照し、monomorphize orchestration / specialization logic を残す形にした。
- `nodesrc/test_parser_backend_responsibility_policy.js` に新 module の存在、`mod trait_lookup;`、root 1425 lines、新 module 90 lines の上限を追加した。line limit は引き上げていない。
- `nodesrc/test_abstraction_static_verification_policy.js` は `MonoTraitApplication` / key model の存在確認先を root file から `monomorphize/trait_lookup.rs` へ移し、typed lookup key の再導入禁止を維持した。
- `doc/neplg2/parser_backend_responsibility_split_plan.md` の M1 と Source policy に今回の split を記録した。

## 検証

cargo check -p nepl-core --tests; node nodesrc/test_parser_backend_responsibility_policy.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture`: pass
- `node nodesrc/test_parser_backend_responsibility_policy.js`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass（warning なし）
