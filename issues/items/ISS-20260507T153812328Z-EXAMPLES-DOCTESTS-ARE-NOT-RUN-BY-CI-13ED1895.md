---
id: ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895
title: "Examples doctests are not run by CI"
area: examples
status: open
resolved: false
priority: P2
type: test
created: 2026-05-07
updated: 2026-05-07
target: ".github/workflows/ci.yml, examples/*.nepl, nodesrc/tests.js"
---

# ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895: Examples doctests are not run by CI

## 概要

The CI workflow runs tests/, tutorials/, stdlib/, nm compile, and a counter emit smoke, but it does not run `node nodesrc/tests.js -i examples`. Executable examples such as rpn.nepl and rpn_legacy.nepl contain doctests for ANSI/color output and REPL behavior that can drift without a main-branch CI failure.

## 対象

- `.github/workflows/ci.yml, examples/*.nepl, nodesrc/tests.js`

## 根拠

- 未記入

## 問題

The CI workflow runs tests/, tutorials/, stdlib/, nm compile, and a counter emit smoke, but it does not run `node nodesrc/tests.js -i examples`. Executable examples such as rpn.nepl and rpn_legacy.nepl contain doctests for ANSI/color output and REPL behavior that can drift without a main-branch CI failure.

## 影響

Example regressions can reach main even when user-facing sample programs are broken. This is especially risky after stdlib I/O, ANSI style, string, collection, and ownership changes because examples are the public integration surface for those APIs.

## 修正方針

Add an examples doctest CI step or matrix entry that runs `node nodesrc/tests.js -i examples -o examples-tests.json -j 4`, uploads the JSON artifact, and includes its result in the final Pages status summary.

## 検証

GitHub Actions on main executes the examples doctest job. A deliberate mismatch in an examples/*.nepl doctest fails CI and appears in the uploaded examples test JSON.
