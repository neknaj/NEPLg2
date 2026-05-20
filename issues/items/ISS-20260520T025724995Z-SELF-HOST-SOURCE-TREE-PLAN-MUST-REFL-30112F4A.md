---
id: ISS-20260520T025724995Z-SELF-HOST-SOURCE-TREE-PLAN-MUST-REFL-30112F4A
title: "self-host source tree plan must reflect expanded proof architecture"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "doc/neplg2/self_host_source_tree_layout_review_20260518.md, doc/neplg2/self_host_plan.md, issues/items/ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD.md"
---

# ISS-20260520T025724995Z-SELF-HOST-SOURCE-TREE-PLAN-MUST-REFL-30112F4A: self-host source tree plan must reflect expanded proof architecture

## 概要

The current self-host source tree layout review predates the Stage 6 proof architecture expansion. core/proof now contains large query/solver/fact/API files, so continuing implementation without a refreshed split policy risks recreating the flat Rust compiler structure in NEPL.

## 対象

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md, doc/neplg2/self_host_plan.md, issues/items/ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD.md`

## 根拠

- 2026-05-18 の source tree review は、`core/proof/` が source span / raw backend 程度の最小 proof entry だった時点の構造を前提にしていた。
- 2026-05-20 時点では `core/proof/` が source span、raw backend、module directive、module declaration、type kind、trait coherence、lifetime、Resource cell、owner、borrow、effect の domain を持つ。
- 現行 file size は `solver.nepl` 959 行、`query.nepl` 490 行、`fact.nepl` 469 行、`api.nepl` 418 行であり、既存の source tree review が定めた pass implementation の 900 行目安を `solver.nepl` が超えている。
- Rust 側にも `parser.rs` 4,044 行、`codegen_llvm.rs` 3,847 行、`codegen_wasm.rs` 2,392 行、`compiler.rs` 2,183 行、`types.rs` 2,176 行、`loader.rs` 2,165 行などの flat 巨大 file が残っており、self-host 側が同じ構造を移植しないための最新 gate が必要である。

## 問題

The current self-host source tree layout review predates the Stage 6 proof architecture expansion. core/proof now contains large query/solver/fact/API files, so continuing implementation without a refreshed split policy risks recreating the flat Rust compiler structure in NEPL.

## 影響

Self-host implementation could keep adding proof obligations, evidence, refutations, and API projection code to a few large files, weakening auditability and match-based static verification.

## 修正方針

Update the self-host source tree layout review with current Rust/self-host file-size data and a concrete proof/refutation/API split policy before adding more self-host compiler implementation.

## 対応内容

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` に 2026-05-20 proof architecture refresh を追加した。
- `core/proof/` の現行 file size、Rust 側の主要巨大 file、proof split policy、次に proof domain を増やす前の分割 gate を明記した。
- `doc/neplg2/self_host_plan.md` からも、2026-05-20 追補込みの source tree review を正とするように更新した。
- self-host parent issue に proof architecture refresh の現状と follow-up issue を追記した。
- follow-up として `ISS-20260520T025806063Z-SELF-HOST-PROOF-FILES-EXCEED-SPLIT-T-023C09E6` を作成した。

## 検証

Run issues index/check and diff check; the change is documentation/issue planning only.
