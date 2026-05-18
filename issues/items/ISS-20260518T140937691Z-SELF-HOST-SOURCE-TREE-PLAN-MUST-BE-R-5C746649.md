---
id: ISS-20260518T140937691Z-SELF-HOST-SOURCE-TREE-PLAN-MUST-BE-R-5C746649
title: "self-host source tree plan must be revalidated against the current Rust compiler structure"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "doc/neplg2/self_host_source_tree_layout_review_20260518.md, doc/neplg2/self_host_plan.md, issues/items/ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD.md"
---

# ISS-20260518T140937691Z-SELF-HOST-SOURCE-TREE-PLAN-MUST-BE-R-5C746649: self-host source tree plan must be revalidated against the current Rust compiler structure

## 概要

The existing self-host plan lists a directory skeleton, but implementation has continued while the Rust compiler grew into a large partially flat tree. Before adding more self-host compiler code, the NEPL version needs an explicit source-tree and file-splitting policy based on the current Rust implementation size, the static-check redesign, and the requirement for generic proof machinery.

## 対象

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md, doc/neplg2/self_host_plan.md, issues/items/ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD.md`

## 根拠

- 2026-05-18 時点の `nepl-core/src` は 382 files / 約 79,607 行であり、root 直下だけで 36 files / 約 27,732 行がある。
- 大きい root file として `parser.rs` 4,036 行、`codegen_llvm.rs` 3,847 行、`codegen_wasm.rs` 2,392 行、`compiler.rs` 2,183 行、`types.rs` 2,044 行、`loader.rs` 2,011 行がある。
- self-host 側の `stdlib/neplg2/` も部分実装段階で 39 files / 約 10,239 行になっており、`core/syntax/lexer.nepl` 1,329 行、`core/hir/hir.nepl` 1,036 行、`core/syntax/token.nepl` 864 行、`core/ty/ty.nepl` 769 行が既に大きい。
- 静的検査大規模修正の方針上、Resource IR / owner / initialized / borrow / effect / abstraction check は個別 ad hoc 証明器ではなく、typed fact と obligation を汎用 proof engine へ渡す構造が必要である。

## 問題

The existing self-host plan lists a directory skeleton, but implementation has continued while the Rust compiler grew into a large partially flat tree. Before adding more self-host compiler code, the NEPL version needs an explicit source-tree and file-splitting policy based on the current Rust implementation size, the static-check redesign, and the requirement for generic proof machinery.

## 影響

Without a renewed layout policy, self-host work can copy the Rust compiler's flat parser/typecheck/codegen shape into stdlib/neplg2, making static checking, abstraction features, Resource IR, diagnostics, documentation, and future self-host maintenance harder to verify.

## 修正方針

Audit nepl-core/src and the current stdlib/neplg2 tree, document the target hierarchy, file-splitting rules, proof-engine placement, and migration order, then link the self-host parent issue to the renewed plan.

## 検証

Run issues index/check and git diff --check for the docs-only checkpoint.

## 対応結果

2026-05-18 に [doc/neplg2/self_host_source_tree_layout_review_20260518.md](../../doc/neplg2/self_host_source_tree_layout_review_20260518.md) を追加し、現行 Rust 実装の分量、self-host 側の既存巨大 file、目標ディレクトリ構造、汎用 proof engine の配置、分割原則、直近の適用順を整理した。

[doc/neplg2/self_host_plan.md](../../doc/neplg2/self_host_plan.md) からこの文書を参照し、2026-05-18 以降の実装では同文書の階層化方針を優先することを明記した。
