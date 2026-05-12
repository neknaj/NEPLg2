---
id: ISS-20260512T212421953Z-SELFHOST-DIAGNOSTIC-CODE-ENUM-POLICY-656F8C6E
title: "Selfhost diagnostic code enum policy does not verify leaf mappings"
area: selfhost
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-12
updated: 2026-05-13
target: "nodesrc/test_selfhost_diag_code_enum.js, stdlib/neplg2/core/infra/diag.nepl"
---

# ISS-20260512T212421953Z-SELFHOST-DIAGNOSTIC-CODE-ENUM-POLICY-656F8C6E: Selfhost diagnostic code enum policy does not verify leaf mappings

## 概要

D5 self-host diagnostic parity currently checks the top-level typed diagnostic enum and raw string constructor ban, but it does not verify that every leaf diagnostic enum variant is present in its stable string conversion function exactly once. A missing leaf arm or wildcard fallback could weaken enum-first diagnostic maintenance without being caught by source policy.

## 対象

- `nodesrc/test_selfhost_diag_code_enum.js, stdlib/neplg2/core/infra/diag.nepl`

## 根拠

- `nodesrc/test_selfhost_diag_code_enum.js` は `SelfhostDiagnosticCode` の top-level 階層、`SelfhostDiagnostic.code` の typed field、raw string constructor の禁止を確認していた。
- 一方で、`SelfhostLoaderDiagnosticCode` / `SelfhostLexerDiagnosticCode` / `SelfhostParserDiagnosticCode` / `SelfhostResolveDiagnosticCode` / `SelfhostCliDiagnosticCode` の leaf variant が、対応する `selfhost_*_diag_code_name` の `match` arm に全件出ているかは確認していなかった。
- そのため leaf enum 追加時に stable string 変換を更新し忘れても、source policy 側では D5 self-host diagnostic parity の不備として検出できなかった。

## 問題

D5 self-host diagnostic parity currently checks the top-level typed diagnostic enum and raw string constructor ban, but it does not verify that every leaf diagnostic enum variant is present in its stable string conversion function exactly once. A missing leaf arm or wildcard fallback could weaken enum-first diagnostic maintenance without being caught by source policy.

## 影響

Self-host diagnostic categories added for parser, resolver, checker, Resource IR, or backend could drift from the Rust diagnostic contract while tests still pass. That undermines the requirement that diagnostic IDs are enum-managed and match-exhaustive rather than free-form strings.

## 修正方針

Extend the self-host diagnostic source policy to enumerate Selfhost*DiagnosticCode leaf enums, inspect the corresponding selfhost_*_diag_code_name functions, reject wildcard arms, and require every variant to appear exactly once in the conversion match.

## 検証

node nodesrc/test_selfhost_diag_code_enum.js; node nodesrc/issues.js check --dir issues

## 対応結果

2026-05-13 に `nodesrc/test_selfhost_diag_code_enum.js` を拡張し、self-host diagnostic の leaf enum mapping を source policy で検査するようにした。

- `SelfhostDiagnosticCode` の category 追加時は policy 更新を要求する。
- 各 `Selfhost*DiagnosticCode` leaf enum の variant を列挙し、対応する `selfhost_*_diag_code_name` に exactly once で現れることを検査する。
- leaf conversion function の wildcard arm を拒否する。
- stable string が該当 stage prefix（`loader.`、`lexer.`、`parser.`、`resolve.`、`cli.`）で始まることを検査する。

これにより、self-host diagnostic の stable string 境界も enum-first / exhaustive-match 方針から外れた場合に source policy で検出できる。
