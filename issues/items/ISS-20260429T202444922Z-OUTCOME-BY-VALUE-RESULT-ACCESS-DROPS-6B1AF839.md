---
id: ISS-20260429T202444922Z-OUTCOME-BY-VALUE-RESULT-ACCESS-DROPS-6B1AF839
title: "Outcome by-value result access drops Diags owner contract"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/diag/error.nepl, stdlib/tests/error.n.md"
---

# ISS-20260429T202444922Z-OUTCOME-BY-VALUE-RESULT-ACCESS-DROPS-6B1AF839: Outcome by-value result access drops Diags owner contract

## 概要

Outcome can carry an owned Diags value. outcome_with_diags replaces an existing Diags without freeing it, and outcome_result consumes Outcome while returning only result, leaving any Diags owner unclosed.

## 対象

- `stdlib/alloc/diag/error.nepl, stdlib/tests/error.n.md`

## 根拠

- `Diag` payload は owner-neutral に整理済みだが、`Diags` は `Vec<Diag>` backing storage owner を持つ。
- `outcome_with_diags` は既存 `Outcome.diags` が `Some old_ds` の場合でも、old_ds を `diags_free` せずに新しい `Diags` へ置き換えていた。
- `outcome_result` は by-value で `Outcome` を消費して `result` だけを返すため、`diags` が `Some ds` の場合に `Diags` owner を閉じる契約がなかった。
- `stdlib/tests/error.n.md` に、既存 diagnostics を持つ `Outcome` へ別 diagnostics を付与してから `outcome_result` で result 軸だけを取り出す regression を追加した。

## 問題

Outcome can carry an owned Diags value. outcome_with_diags replaces an existing Diags without freeing it, and outcome_result consumes Outcome while returning only result, leaving any Diags owner unclosed.

## 影響

Code that observes only the Result axis of an Outcome can leak the diagnostics backing Vec owner or force callers into undocumented cleanup order. This weakens the owner-neutral Diag redesign because Diags ownership is still part of Outcome.

## 修正方針

Make by-value Outcome result access consume or free the Diags axis before returning result, and make outcome_with_diags free any existing Diags before replacement. Add regression coverage for replacing diagnostics and extracting result from an Outcome that contains Diags.

## 修正内容

- `outcome_with_diags` は `result` を返却先へ移しつつ、既存 `diags` が `Some` なら `diags_free` で閉じてから新しい `Diags` を保持するようにした。
- `outcome_result` は返さない `diags` axis を `diags_free` で閉じてから `result` を返す契約にした。
- `stdlib/tests/error.n.md` に diagnostics 置換後の by-value `outcome_result` regression を追加した。

## 検証

- `git diff --check`: passed
- `node nodesrc/issues.js check`: passed
- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 2 --dist web/dist`: passed
- `node nodesrc/tests.js -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md --no-tree -o tmp/outcome-diags-free-contract.json -j 1 --dist web/dist`: `total=5`, `passed=5`
