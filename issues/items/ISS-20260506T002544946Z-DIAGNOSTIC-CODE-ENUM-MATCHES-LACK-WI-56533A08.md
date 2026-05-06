---
id: ISS-20260506T002544946Z-DIAGNOSTIC-CODE-ENUM-MATCHES-LACK-WI-56533A08
title: "Diagnostic code enum matches lack wildcard source policy"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-06
updated: 2026-05-06
target: "nodesrc/test_diagnostic_code_first_boundary.js, nepl-core/src/diagnostic_codes.rs"
---

# ISS-20260506T002544946Z-DIAGNOSTIC-CODE-ENUM-MATCHES-LACK-WI-56533A08: Diagnostic code enum matches lack wildcard source policy

## 概要

DiagnosticCode and subcode as_str/message mappings rely on exhaustive match arms for static safety, but the source policy did not reject wildcard arms in diagnostic code conversion impls. A future _ => arm could hide newly added enum variants from Rust's exhaustiveness checking.

## 対象

- `nodesrc/test_diagnostic_code_first_boundary.js, nepl-core/src/diagnostic_codes.rs`

## 根拠

- `doc/neplg2/compiler_diagnostics_redesign_plan.md` は diagnostic code を enum-first にし、`as_str()` / `message()` の exhaustive match で stable code / message を管理する前提にしている。
- `nepl-core/src/diagnostic_codes.rs` の現行実装は `_ =>` を使わず variant ごとに列挙していたが、`nodesrc/test_diagnostic_code_first_boundary.js` は code-less diagnostic と `.with_code(...)` だけを検査しており、conversion match の wildcard 回帰を検出していなかった。
- wildcard arm が入ると、新しい diagnostic code variant を追加しても Rust の match 網羅性検査が働かず、taxonomy の明示的な設計判断が抜け落ちる。

## 問題

DiagnosticCode and subcode as_str/message mappings rely on exhaustive match arms for static safety, but the source policy did not reject wildcard arms in diagnostic code conversion impls. A future _ => arm could hide newly added enum variants from Rust's exhaustiveness checking.

## 影響

Diagnostic code additions could silently fall back to an existing string/message path instead of forcing an explicit taxonomy decision, weakening the enum-first diagnostics redesign and static-check maintainability.

## 修正方針

Extend the diagnostic source policy to inspect nepl-core/src/diagnostic_codes.rs and reject wildcard match arms in DiagnosticCode and every diagnostic subcode conversion impl.

## 検証

Run node nodesrc/test_diagnostic_code_first_boundary.js, node nodesrc/issues.js check, and diff whitespace checks.

## 対応結果

2026-05-06 に `nodesrc/test_diagnostic_code_first_boundary.js` を拡張し、`nepl-core/src/diagnostic_codes.rs` の `*DiagnosticCode` impl を走査する source policy を追加した。

- 各 diagnostic code impl に `as_str(self)` と `message(self)` が存在することを検査する。
- 各 impl 内で `_ =>` / `_ if ... =>` の wildcard match arm を禁止する。
- これにより、新しい diagnostic code variant を追加したときに、stable code string と message を明示的に追加しなければ Rust 側の網羅性検査または source policy で検出される。

検証:

- `node nodesrc/test_diagnostic_code_first_boundary.js`
