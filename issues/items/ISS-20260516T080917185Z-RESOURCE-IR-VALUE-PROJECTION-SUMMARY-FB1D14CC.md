---
id: ISS-20260516T080917185Z-RESOURCE-IR-VALUE-PROJECTION-SUMMARY-FB1D14CC
title: "Resource IR value projection summary is hard-coded to Result"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/resource/initialized_alias_flow_value_projection.rs
---

# ISS-20260516T080917185Z-RESOURCE-IR-VALUE-PROJECTION-SUMMARY-FB1D14CC: Resource IR value projection summary is hard-coded to Result

## 概要

A helper that carries a raw-address field through any other enum wrapper has the same source/IR proof shape as Result::Ok, but the checker refuses to summarize it because the enum is not named Result. This is a special-case proof and violates the generic static-check design.

## 対象

- `nepl-core/src/resource/initialized_alias_flow_value_projection.rs`

## 根拠

The value projection alias summary gate checks the returned type name with type_is_result_enum and only accepts Result. The rest of the summary engine already traces actual Resource IR alias flow through construct/read/match/call operations.

## 問題

A helper that carries a raw-address field through any other enum wrapper has the same source/IR proof shape as Result::Ok, but the checker refuses to summarize it because the enum is not named Result. This is a special-case proof and violates the generic static-check design.

## 影響

Initialized raw-cell alias proof becomes incomplete for custom enum payloads and can force future stdlib/compiler work back toward ad-hoc allowlists. It also makes static-check mistakes harder to catch because the authority is a string name rather than an exhaustive structural rule.

## 修正方針

Remove the Result-name gate and let the existing simple Resource IR value-projection proof decide whether a function can be summarized. Add regression coverage with a custom enum payload and a source-policy guard against reintroducing the Result-only gate.

## 検証

Run the new custom enum Resource IR regression, the existing Result payload regression, resource checker responsibility policy, issues check, cargo fmt/check, and git diff --check.

## 対応

2026-05-16 Agent 1:

- `function_allows_value_projection_summary` から `Result` 型名 gate を削除し、single-block/simple-op Resource IR body の value projection proof だけで summary 可否を決めるようにした。
- `MaybeBox::Ready(Boxed)` を返す custom enum helper 経由で raw address field alias が維持される regression を追加した。
- `nodesrc/test_resource_checker_responsibility.js` に `type_is_result_enum` と `name == "Result"` の再導入禁止を追加し、静的検査の authority が enum 名文字列へ戻らないようにした。

この修正は stdlib module / function / enum 名の allowlist ではなく、Resource IR の construct/read/match/call から導出される value projection alias flow を使う。
