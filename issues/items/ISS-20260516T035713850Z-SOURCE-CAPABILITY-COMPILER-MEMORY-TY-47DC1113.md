---
id: ISS-20260516T035713850Z-SOURCE-CAPABILITY-COMPILER-MEMORY-TY-47DC1113
title: "Source capability compiler memory type evidence re-walks module outside unified proof"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/source_capability/**"
---

# ISS-20260516T035713850Z-SOURCE-CAPABILITY-COMPILER-MEMORY-TY-47DC1113: Source capability compiler memory type evidence re-walks module outside unified proof

## 概要

SourceCapabilityProof unifies raw memory and owner aggregate evidence, but compiler memory type definitions are still collected by a separate module-level walker before being inserted into the proof. This keeps one capability domain outside the shared proof lifecycle.

## 対象

- `nepl-core/src/source_capability/**`

## 根拠

- `SourceCapabilityProofCollector` は raw memory / owner aggregate evidence を `walk_module_capability_evidence` から収集していたが、`compiler_memory_types` だけは `module_compiler_memory_type_definitions(module)` で別走査していた。
- `module_compiler_memory_type_definitions` は `Module.root.items` を独自に走査しており、shared proof traversal の lifecycle / policy から外れていた。
- 直前の [ISS-20260516T034018225Z-SOURCE-CAPABILITY-EVIDENCE-USES-PER--87E68F2E](./ISS-20260516T034018225Z-SOURCE-CAPABILITY-EVIDENCE-USES-PER--87E68F2E.md) で unified proof を導入したが、全 capability domain を完全に単一 proof に乗せるには struct definition evidence も observer callback で収集する必要があった。

## 問題

SourceCapabilityProof unifies raw memory and owner aggregate evidence, but compiler memory type definitions are still collected by a separate module-level walker before being inserted into the proof. This keeps one capability domain outside the shared proof lifecycle.

## 影響

Source capability authority remains harder to audit than necessary: future changes to source traversal, struct definition handling, or capability collection can diverge between the generic proof collector and the compiler memory type classifier.

## 修正方針

Move compiler memory type definition observation into the shared SourceCapabilityObserver traversal. Keep memory_type_definition.rs as a typed classifier over StructDef, and have SourceCapabilityProofCollector record CompilerMemoryType evidence from the same walk as all other capability domains.

## 解決内容

- `SourceCapabilityObserver` に `observe_struct_definition` を追加し、`source_capability/walk.rs` の shared traversal が `Stmt::StructDef` を capability evidence event として通知するようにした。
- `SourceCapabilityProofCollector` は `observe_struct_definition` で `compiler_memory_type_from_struct_def` を呼び、`CompilerMemoryType` evidence を proof 内の `BTreeSet` に直接記録するようにした。
- `memory_type_definition.rs` は module walker を持たず、`StructDef -> Option<CompilerMemoryType>` の typed classifier に縮小した。
- source policy を更新し、struct definition observation が shared traversal / unified proof collector にあることと、旧 `module_compiler_memory_type_definitions` を proof が呼ばないことを監視するようにした。

## 検証

- `cargo check -p nepl-core`
- `cargo test -p nepl-core compiler_memory_type_definition -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo fmt -p nepl-core -- --check`
