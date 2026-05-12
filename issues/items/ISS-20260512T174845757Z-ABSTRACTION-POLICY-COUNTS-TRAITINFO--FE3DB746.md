---
id: ISS-20260512T174845757Z-ABSTRACTION-POLICY-COUNTS-TRAITINFO--FE3DB746
title: "Abstraction policy counts TraitInfo doc as ImplInfo optional string baseline"
area: core
status: verified
resolved: true
priority: P1
type: test
created: 2026-05-12
updated: 2026-05-13
target: "nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T174845757Z-ABSTRACTION-POLICY-COUNTS-TRAITINFO--FE3DB746: Abstraction policy counts TraitInfo doc as ImplInfo optional string baseline

## 概要

The abstraction static verification policy reports implInfoOptionString by counting Option<String> across all of typecheck/traits.rs. The remaining count is TraitInfo.doc, not ImplInfo, so the policy keeps a stale baseline of 1 and does not directly prove that ImplInfo has no optional string fields.

## 対象

- `nodesrc/test_abstraction_static_verification_policy.js`

## 根拠

- `nodesrc/test_abstraction_static_verification_policy.js` は `implInfoOptionString` を `typecheck/traits.rs` 全体の `Option<String>` 数として計算していた。
- 2026-05-12 時点の残り 1 件は `TraitInfo.doc` であり、`ImplInfo` の field ではない。
- 同じ policy の下部では `ImplInfo` struct body を既に抽出しているため、`ImplInfo` の optional field 再導入は struct body 単位で検査できる。
- 関連計画: [NEPLg2 abstraction static verification plan Stage 6](../../doc/neplg2/abstraction_static_verification_plan.md#stage-6-policy-baseline-%E3%82%92-0-%E3%81%AB%E4%B8%8B%E3%81%92%E3%82%8B)

## 問題

The abstraction static verification policy reports implInfoOptionString by counting Option<String> across all of typecheck/traits.rs. The remaining count is TraitInfo.doc, not ImplInfo, so the policy keeps a stale baseline of 1 and does not directly prove that ImplInfo has no optional string fields.

## 影響

Stage 6 policy baseline cannot be lowered to the final no-reintroduction rule. A future edit could add an Option<String> to ImplInfo while unrelated Option<String> counts hide the intent of the policy, weakening enum-first static verification of trait impl identity.

## 修正方針

Change the policy to inspect the ImplInfo struct body directly and require zero Option fields there. Keep TraitInfo.doc outside the ImplInfo optional-string metric, update the abstraction plan progress, and record this as the Stage 6 policy baseline cleanup.

## 対応記録

- `implInfoOptionString` の global `Option<String>` baseline を削除した。
- `ImplInfo` struct body を抽出したうえで `implInfoOptionalFields === 0` を要求し、optional field model の再導入を直接拒否するようにした。
- `TraitInfo.doc` の `Option<String>` は documentation data として残し、impl identity policy の metric から切り離した。
- `doc/neplg2/abstraction_static_verification_plan.md` Stage 6 と親 issue に、policy baseline を struct body 直接検査へ移したことを追記した。

## 検証

node nodesrc/test_abstraction_static_verification_policy.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues

## 確認済み

- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: completed。`nodesrc/test_parser_backend_responsibility_policy.js` の `monomorphize.rs` line limit warning は既存 open issue `ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587` の範囲として扱う。
