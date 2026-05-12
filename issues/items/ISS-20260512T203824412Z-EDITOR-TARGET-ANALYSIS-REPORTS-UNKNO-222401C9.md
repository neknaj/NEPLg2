---
id: ISS-20260512T203824412Z-EDITOR-TARGET-ANALYSIS-REPORTS-UNKNO-222401C9
title: "Editor target analysis reports unknown target directives twice"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-13
target: "nepl-language/src/lib.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_diagnostics_redesign_plan.md"
---

# ISS-20260512T203824412Z-EDITOR-TARGET-ANALYSIS-REPORTS-UNKNO-222401C9: Editor target analysis reports unknown target directives twice

## 概要

The editor analysis target resolver scans module directives, but when every #target directive is unknown it leaves found target as None and then scans root directives again. Unknown target directives therefore produce duplicate loader.target.unknown diagnostics in nepl-language, and nepl-web has the same found-target based fallback shape.

## 対象

- `nepl-language/src/lib.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 根拠

- `nepl-language/src/lib.rs` の `resolve_target_for_analysis` は `module.directives` を先に走査するが、unknown `#target` では `found` が `None` のまま残る。
- その後 `found.is_none()` を条件に `module.root.items` を走査するため、同じ unknown `#target` が root directive として再診断される。
- `nepl-web/src/lib.rs` も同じ `found.is_none()` fallback 構造を持っていた。

## 問題

The editor analysis target resolver scans module directives, but when every #target directive is unknown it leaves found target as None and then scans root directives again. Unknown target directives therefore produce duplicate loader.target.unknown diagnostics in nepl-language, and nepl-web has the same found-target based fallback shape.

## 影響

Editor and web diagnostics over-report a single source error, making diagnostic code-count regressions unreliable and weakening the Stage D3 contract that one source violation maps to one stable diagnostic event.

## 修正方針

Track whether any #target directive was seen separately from whether a valid target was found, and only run the fallback root scan when no target directive was seen at all. Add focused regression counts for unknown target diagnostics.

## 検証

cargo test -p nepl-language target_directive_diagnostics_keep_loader_codes; cargo check -p nepl-language -p nepl-lsp --tests; node nodesrc/test_diagnostic_code_first_boundary.js; node nodesrc/issues.js check --dir issues

## 2026-05-13 修正

`found valid target` と `saw target directive` を分離した。

- `nepl-language` / `nepl-web` の `resolve_target_for_analysis` に `saw_target_directive` を追加した。
- `module.directives` で unknown を含む `#target` を見た場合も `saw_target_directive = true` にする。
- fallback の `module.root.items` 走査は `!saw_target_directive` の場合だけ実行する。
- `nepl-language` の regression は `loader.target.unknown` と `code_message` の組が 1 件だけ出ることを固定する。
- `nodesrc/test_diagnostic_code_first_boundary.js` は language / web の target analysis が `saw_target_directive` based fallback を使うことを監視する。

これにより、単一の unknown `#target` が同じ stable code で二重報告される状態を解消した。
