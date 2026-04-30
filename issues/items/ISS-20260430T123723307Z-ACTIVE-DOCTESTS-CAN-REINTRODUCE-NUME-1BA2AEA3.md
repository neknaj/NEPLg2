---
id: ISS-20260430T123723307Z-ACTIVE-DOCTESTS-CAN-REINTRODUCE-NUME-1BA2AEA3
title: "active doctests can reintroduce numeric diagnostic IDs"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: "nodesrc/test_doctest_diag_code_metadata.js, tests/compiler/compile_fail_diag_location.n.md"
source: doc/neplg2/compiler_diagnostics_redesign_plan.md#stage-d0-数値-id-の削除と-enum-registry-導入
---

# ISS-20260430T123723307Z-ACTIVE-DOCTESTS-CAN-REINTRODUCE-NUME-1BA2AEA3: active doctests can reintroduce numeric diagnostic IDs

## 概要

Active .n.md doctest files can still contain numeric diagnostic ID wording or metadata such as diag_code: 3092 without a source-policy failure. This weakens the diagnostics redesign rule that stable diagnostic codes are enum-backed strings only.

## 対象

- `nodesrc/test_doctest_diag_code_metadata.js, tests/compiler/compile_fail_diag_location.n.md`

## 根拠

- `tests/compiler/compile_fail_diag_location.n.md` の active prose に `diag_code: 3092` という旧数値 ID 表記が残っていた。
- `nodesrc/test_doctest_diag_code_metadata.js` は parser が `diag_ids` を出さないことは確認していたが、active repository source に旧 `diag_id:` / numeric `diag_code:` が戻らないことまでは検査していなかった。
- diagnostic redesign plan は active code path / doctest metadata から数値 ID を削除し、stable string code を enum registry の境界表現にする方針である。

## 問題

Active .n.md doctest files can still contain numeric diagnostic ID wording or metadata such as diag_code: 3092 without a source-policy failure. This weakens the diagnostics redesign rule that stable diagnostic codes are enum-backed strings only.

## 影響

Old numeric IDs can return through tests or prose and make future diagnostics regressions compare legacy buckets rather than hierarchical DiagnosticCode variants.

## 修正方針

Extend the doctest diagnostic metadata policy to scan active test/tutorial/stdlib/example sources for diag_id/diag_ids metadata and numeric diag_code/diag_codes forms, then update active prose to name the stable string code.

## 検証

Run node nodesrc/test_doctest_diag_code_metadata.js, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`nodesrc/test_doctest_diag_code_metadata.js` に active doctest source scan を追加し、`tests` / `tutorials` / `stdlib` / `examples` 配下の `.n.md` と `.nepl` doc-comment doctest から次を禁止した。

- `diag_id:` / `diag_ids:` metadata。
- `diag_code:` / `diag_codes:` に数値または `Dxxxx` 形式を入れる表記。

あわせて `tests/compiler/compile_fail_diag_location.n.md` の説明を `resolve.entry_function.missing_or_ambiguous` の stable code 表記へ直した。これにより active doctest が旧数値 diagnostic ID へ戻ると source policy で失敗する。

検証:

- `node nodesrc/test_doctest_diag_code_metadata.js`: passed
- `rg -n 'diag_id:|diag_ids:|diag_code:\\s*(?:\\[\\s*)?(?:"?D?\\d{3,4}"?|\\d{3,4})|diag_codes:\\s*(?:\\[\\s*)?(?:"?D?\\d{3,4}"?|\\d{3,4})' tests tutorials stdlib examples -g '*.n.md' -g '*.nepl'`: no matches
