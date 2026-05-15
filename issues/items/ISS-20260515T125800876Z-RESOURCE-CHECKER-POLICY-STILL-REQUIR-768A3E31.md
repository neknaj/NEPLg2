---
id: ISS-20260515T125800876Z-RESOURCE-CHECKER-POLICY-STILL-REQUIR-768A3E31
title: "Resource checker policy still requires public escape owner filter in summary filter"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/effect_return_summary_filter.rs"
---

# ISS-20260515T125800876Z-RESOURCE-CHECKER-POLICY-STILL-REQUIR-768A3E31: Resource checker policy still requires public escape owner filter in summary filter

## 概要

The source policy still requires effect_return_summary_filter.rs to import raw_identity_projection_has_owner_protection from effect_return_escape, but ISS-20260515T110646911Z separated public raw escape filtering from internal raw identity summary filtering so RegionToken provenance remains available for checked MemPtr proof.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/effect_return_summary_filter.rs`

## 根拠

- `nodesrc/test_resource_checker_responsibility.js` が `effect_return_summary_filter.rs must import raw_identity_projection_has_owner_protection from super::effect_return_escape` として失敗していた。
- `ISS-20260515T110646911Z-CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK-B846CF4C` では、public raw escape diagnostic と internal raw identity summary filtering を意図的に分離している。
- 現行の `effect_return_summary_filter.rs` は `str` の opaque high-level owner identity を summary から抑止しつつ、`RegionToken` provenance は checked `MemPtr` proof のために保持する。これを public escape filter に戻すと静的検査の証明力が落ちる。

## 問題

The source policy still requires effect_return_summary_filter.rs to import raw_identity_projection_has_owner_protection from effect_return_escape, but ISS-20260515T110646911Z separated public raw escape filtering from internal raw identity summary filtering so RegionToken provenance remains available for checked MemPtr proof.

## 影響

The policy warning hides real Resource IR regressions and could pressure future changes to re-couple public escape diagnostics with internal provenance summaries, breaking checked RegionToken-derived MemPtr proof.

## 修正方針

Update the Resource checker policy to enforce the intentional separation: summary filtering must keep its internal opaque-owner filter, must not import the public escape owner-protection helper, must suppress str summaries, and must keep RegionToken provenance summaries.

## 検証

Run nodesrc/test_resource_checker_responsibility.js, focused effect_return_summary_filter Rust tests, source policy warn-only, issues check, and diff whitespace check.

## 対応

`nodesrc/test_resource_checker_responsibility.js` の監視を、現在の Resource IR 設計に合わせて更新した。

- `effect_return_summary_filter.rs` が `raw_identity_projection_has_owner_protection` を import することを要求する古い検査を削除した。
- 代わりに、summary filter が内部 provenance 用の `raw_identity_projection_has_summary_opaque_owner_protection` を持つことを確認する。
- `str` raw identity summary は抑止すること、`RegionToken` owner provenance summary は保持することを policy で固定した。
- `node nodesrc/test_resource_checker_responsibility.js` はこの stale filter policy を通過し、次の別 blocker として `owner_return_apply.rs has 434 lines; responsibility split limit is 410` を露出した。この別件は `ISS-20260515T125912798Z-RESOURCE-OWNER-RETURN-APPLY-EXCEEDS--5CCCCE97` に分離した。

## 検証結果

- `cargo test -p nepl-core --lib effect_return_summary_filter -- --nocapture`: 4 passed
- `node --check nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: stale filter policy は解消。次 blocker `ISS-20260515T125912798Z-RESOURCE-OWNER-RETURN-APPLY-EXCEEDS--5CCCCE97` に到達。
