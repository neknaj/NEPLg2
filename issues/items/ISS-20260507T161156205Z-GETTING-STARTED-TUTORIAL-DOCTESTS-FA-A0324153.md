---
id: ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153
title: "getting_started tutorial doctests fail on current main"
area: tutorials
status: open
resolved: false
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: "tutorials/getting_started/15_move_and_borrow.n.md, tutorials/getting_started/17_imports_and_modules.n.md"
---

# ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153: getting_started tutorial doctests fail on current main

## 概要

GitHub Actions run 25507326678 reports tutorials-test failure on latest main. The previous completed run log for the same failure set shows tutorials/getting_started/15_move_and_borrow.n.md::doctest#1 missing expected diag_code resource.move.use_moved, and tutorials/getting_started/17_imports_and_modules.n.md::doctest#1 failing with resolve.identifier.undefined.

## 対象

- `tutorials/getting_started/15_move_and_borrow.n.md, tutorials/getting_started/17_imports_and_modules.n.md`

## 根拠

- `gh run list --branch main --limit 6` で latest main run `25507326678` の `tutorials-test` が failure であることを確認した。
- `gh run view 25507054306 --job 74856320520 --log` で、直前 completed old run の `tutorials-test` が `44 total / 40 passed / 4 failed` と出力していることを確認した。
- 同 log では `tutorials/getting_started/15_move_and_borrow.n.md::doctest#1` が `diag_code: resource.move.use_moved` の missing code で失敗している。
- 同 log では `tutorials/getting_started/17_imports_and_modules.n.md::doctest#1` が `resolve.identifier.undefined` で失敗している。

## 問題

GitHub Actions run 25507326678 reports tutorials-test failure on latest main. The previous completed run log for the same failure set shows tutorials/getting_started/15_move_and_borrow.n.md::doctest#1 missing expected diag_code resource.move.use_moved, and tutorials/getting_started/17_imports_and_modules.n.md::doctest#1 failing with resolve.identifier.undefined.

## 影響

The rewritten getting_started tutorial is no longer executable documentation on main. Learners see stale or broken ownership/import examples, and CI tutorial failure hides unrelated compiler or stdlib regressions.

## 修正方針

Root-cause each doctest against current compiler diagnostics and module import semantics. Update stale diagnostic expectations to the current ResourceIR diagnostic ID when the static rejection is correct, and fix the import sample or resolver behavior depending on which side violates the current language contract.

## 検証

Use gh to confirm tutorials-test on the latest main run. After fixing, run the focused getting_started doctests and ensure CI tutorials-test reports these chapters passing.
