---
id: ISS-20260426T174140008Z-NEPLG2-SELFHOST-LACKS-SHARED-DIAGNOS-CD740CFD
title: "neplg2 selfhost lacks shared diagnostic outcome infrastructure"
area: selfhost
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/neplg2/core/infra/diag.nepl, stdlib/neplg2/core/infra/outcome.nepl, tests/stdlib/neplg2_diag_outcome.n.md"
---

# ISS-20260426T174140008Z-NEPLG2-SELFHOST-LACKS-SHARED-DIAGNOS-CD740CFD: neplg2 selfhost lacks shared diagnostic outcome infrastructure

## 概要

NEPLg2 self-host S1 has spans, source text, and lexer diagnostics, but no shared diagnostic value or diagnostic-carrying Result abstraction. Parser, module loading, typecheck, and backend stages would otherwise invent incompatible error shapes.

## 対象

- `stdlib/neplg2/core/infra/diag.nepl, stdlib/neplg2/core/infra/outcome.nepl, tests/stdlib/neplg2_diag_outcome.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S1 / S3 は parser / typecheck / codegen が同じ diagnostic を返す設計を要求している。
- `stdlib/neplg2/core/infra` には span / text はあるが、shared diagnostic value と diagnostic-carrying Result がなかった。

## 問題

NEPLg2 self-host S1 has spans, source text, and lexer diagnostics, but no shared diagnostic value or diagnostic-carrying Result abstraction. Parser, module loading, typecheck, and backend stages would otherwise invent incompatible error shapes.

## 影響

Self-host stages cannot compose diagnostics consistently, error recovery would lose labels/notes, and later parser/typecheck work would grow ad hoc Result wrappers that are difficult to replace.

## 修正方針

Add pure core infra/diag and infra/outcome modules with severity, label, diagnostic, diagnostic collection, and Outcome helpers that carry Result values plus diagnostics without stdio or filesystem dependencies.

## 解決内容

- `stdlib/neplg2/core/infra/diag.nepl` を追加し、`SelfhostDiagSeverity`、`SelfhostDiagnosticLabel`、`SelfhostDiagnostic`、`SelfhostDiagnostics` と constructor / push / len / get / has_errors / free helper を実装した。
- `SelfhostDiagnostic` は Copy 値だけを保持し、collection から安全に読み出せるようにした。初期段階では primary label と note を 1 件ずつ保持し、複数 label / note は parser recovery の要求に合わせて拡張する。
- `stdlib/neplg2/core/infra/outcome.nepl` を追加し、`Result<T,E>` と `SelfhostDiagnostics` を同時に運ぶ `SelfhostOutcome<T,E>` を実装した。
- `SelfhostOutcome` は result を typed one-cell pointer として所有する。これにより、非 Copy result と diagnostics を raw memory detour なしで分離して扱える。
- `tests/stdlib/neplg2_diag_outcome.n.md` を追加し、diagnostic construction、label/note storage、diagnostic append、Outcome ok/err、diagnostic append、result extraction を固定した。
- 実装中に確認した owned aggregate decomposition の設計不足は、`ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE` として別 issue に分離した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/diag.nepl -i stdlib/neplg2/core/infra/outcome.nepl -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/neplg2-diag-outcome-focused-pass2.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/neplg2-diag-outcome-after-rebase.json -j 1`: 27/27 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-neplg2-diag-outcome-after-rebase.json -j 4`: 414/414 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-neplg2-diag-outcome-after-rebase.json -j 4`: 282/282 passed
- `trunk build`: passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-neplg2-diag-outcome-after-rebase.json`: 13/13 passed
- `node nodesrc/issues.js check`: ok
- `git diff --check`: ok
