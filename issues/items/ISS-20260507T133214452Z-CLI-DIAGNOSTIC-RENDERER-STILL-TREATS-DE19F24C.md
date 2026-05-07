---
id: ISS-20260507T133214452Z-CLI-DIAGNOSTIC-RENDERER-STILL-TREATS-DE19F24C
title: "CLI diagnostic renderer still treats DiagnosticCode as optional"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-cli/src/main.rs, nodesrc/test_diagnostic_code_first_boundary.js"
---

# ISS-20260507T133214452Z-CLI-DIAGNOSTIC-RENDERER-STILL-TREATS-DE19F24C: CLI diagnostic renderer still treats DiagnosticCode as optional

## 概要

GitHub Actions build for run 25498880571 failed because nepl-cli/src/main.rs renders diagnostics with d.code.map(...). Diagnostic.code is now mandatory DiagnosticCode, so the CLI still contains an obsolete Option-era access pattern and fails to compile on the full workspace build.

## 対象

- `nepl-cli/src/main.rs, nodesrc/test_diagnostic_code_first_boundary.js`

## 根拠

- GitHub Actions run `25498880571` の build job が `nepl-cli/src/main.rs:1950` で `DiagnosticCode is not an iterator` として失敗した。
- 原因は `Diagnostic.code` が mandatory `DiagnosticCode` に移行済みなのに、CLI renderer が旧 `Option<DiagnosticCode>` 時代の `d.code.map(...)` を残していたこと。
- `nodesrc/test_diagnostic_code_first_boundary.js` は `nepl-core` / language / LSP / web を監視していたが、`nepl-cli/src` を監視対象に含めていなかった。
- 関連親 issue: [Rust compiler diagnostics are not aligned with Resource IR and self-host model](./ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md)

## 問題

GitHub Actions build for run 25498880571 failed because nepl-cli/src/main.rs renders diagnostics with d.code.map(...). Diagnostic.code is now mandatory DiagnosticCode, so the CLI still contains an obsolete Option-era access pattern and fails to compile on the full workspace build.

## 影響

The code-first diagnostic redesign cannot be trusted if downstream renderers keep optional-code assumptions. CI build stops before compile/tests/deploy, and future CLI diagnostic rendering changes can reintroduce stale Option handling unless source policy covers nepl-cli.

## 修正方針

Render d.code directly through DiagnosticCode::as_str(), add nepl-cli/src to the diagnostic code-first source policy roots, and reject any .code.map(...) optional-code access pattern.

## 検証

Run cargo check, cargo check -p nepl-cli, node nodesrc/test_diagnostic_code_first_boundary.js, node nodesrc/run_source_policy_regressions.js --warn-only, and node nodesrc/issues.js check.

## 2026-05-07 対応結果

`nepl-cli/src/main.rs` の diagnostic renderer を mandatory code 前提に直し、`d.code.as_str()` を直接表示するようにした。`nodesrc/test_diagnostic_code_first_boundary.js` は `nepl-cli/src` を Rust source root に含め、`.code.map(...)` を拒否する。

検証:

- `cargo fmt --check`: passed
- `cargo check -p nepl-cli`: passed
- `cargo check`: passed
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
