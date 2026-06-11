---
id: ISS-20260611T164153627Z-WEB-PLAYGROUND-NEEDS-CI-BROWSER-SMOK-2397F899
title: "Web Playground needs CI browser smoke for workspace editor and GUI flows"
area: tools
status: open
resolved: false
priority: P1
type: test
created: 2026-06-11
updated: 2026-06-11
target: ".github/workflows/ci.yml; nodesrc/playground_editor_test_runner.js; web/src/**"
---

# ISS-20260611T164153627Z-WEB-PLAYGROUND-NEEDS-CI-BROWSER-SMOK-2397F899: Web Playground needs CI browser smoke for workspace editor and GUI flows

## 概要

Current CI covers trunk build and many source policies, but rendered browser smoke for file open/edit/split/close/run/build/status and GUI lifecycle is not wired as a stable gate.

## 対象

- `.github/workflows/ci.yml; nodesrc/playground_editor_test_runner.js; web/src/**`

## 根拠

- 未記入

## 問題

Current CI covers trunk build and many source policies, but rendered browser smoke for file open/edit/split/close/run/build/status and GUI lifecycle is not wired as a stable gate.

## 影響

Layout, focus, disposal, file tree, tab transfer, and browser-only runtime regressions can reach main despite Node-level tests passing.

## 修正方針

Add a Playwright or equivalent browser smoke target that runs after trunk build and exercises editor workspace and GUI lifecycle flows. Keep timeout detection non-fatal only where explicitly intended.

## 検証

CI produces a browser smoke result for open/edit/split/close/run/build/status/GUI close and fails on real functional regressions.
