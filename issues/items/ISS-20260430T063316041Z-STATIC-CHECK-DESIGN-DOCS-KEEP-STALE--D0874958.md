---
id: ISS-20260430T063316041Z-STATIC-CHECK-DESIGN-DOCS-KEEP-STALE--D0874958
title: "Static check design docs keep stale Resource diagnostic taxonomy"
area: docs
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "doc/neplg2/static_check_design_verification_20260430.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260430T063316041Z-STATIC-CHECK-DESIGN-DOCS-KEEP-STALE--D0874958: Static check design docs keep stale Resource diagnostic taxonomy

## 概要

The 2026-04-30 static-check design review still states that ResourceDiagnosticCode only has Move/Borrow/Raw/Lower and that cell/owner diagnostics still need to be added, even though the current Rust implementation already has ResourceDiagnosticCode::Cell and ResourceDiagnosticCode::Owner with explicit mappings. This stale text can make later self-host design and Rust cleanup plans target the wrong diagnostic boundary.

## 対象

- `doc/neplg2/static_check_design_verification_20260430.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `nepl-core/src/diagnostic_codes.rs` の `ResourceDiagnosticCode` は `Move` / `Borrow` / `Cell` / `Owner` / `Raw` / `Lower` に分離済みである。
- `nepl-core/src/compiler.rs` の `resource_cell_diagnostic_code` と `resource_owner_state_diagnostic_code` は `CellState` / `OwnerState` を `resource.cell.*` / `resource.owner.*` へ明示的に写像している。
- `doc/neplg2/static_check_design_verification_20260430.md` には、`ResourceDiagnosticCode` がまだ `Move` / `Borrow` / `Raw` / `Lower` だけで cell/owner を raw bucket に押し込んでいる、という obsolete な記述が残っていた。

## 問題

The 2026-04-30 static-check design review still states that ResourceDiagnosticCode only has Move/Borrow/Raw/Lower and that cell/owner diagnostics still need to be added, even though the current Rust implementation already has ResourceDiagnosticCode::Cell and ResourceDiagnosticCode::Owner with explicit mappings. This stale text can make later self-host design and Rust cleanup plans target the wrong diagnostic boundary.

## 影響

Static-check design discussion can regress toward treating cell and owner violations as raw-memory diagnostics, weakening the enum-first taxonomy required for type and memory safety. Self-host diagnostic planning could also copy an obsolete model.

## 修正方針

Update the design verification and complexity reduction docs to distinguish completed Cell/Owner diagnostic separation from remaining work: old HIR move_check/drop insertion authority, Resource IR final authority, unsafe memory effect gate, and owner-token/stdlib collection migration.

## 検証

Run node nodesrc/issues.js check and git diff --check after updating the docs and issue index.

実行済み:

- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
