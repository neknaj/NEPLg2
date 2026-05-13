---
id: ISS-20260513T052006137Z-RESOURCE-CELL-AND-OWNER-GATES-TRUST--3287B5AF
title: "Resource cell and owner gates trust raw memory boundary instead of Resource IR proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/src/compiler.rs
---

# ISS-20260513T052006137Z-RESOURCE-CELL-AND-OWNER-GATES-TRUST--3287B5AF: Resource cell and owner gates trust raw memory boundary instead of Resource IR proof

## 概要

run_resource_cell_gate and run_resource_owner_obligation_gate skip diagnostics solely because the diagnostic span belongs to a raw-memory-boundary source. That turns a stdlib/source capability into an exception table instead of requiring Resource IR to prove initialized-cell and free-obligation correctness.

## 対象

- `nepl-core/src/compiler.rs`

## 根拠

- `run_resource_cell_gate` は `source_map.raw_memory_boundary_allowed(span.file_id)` が true の診断を `continue` していた。
- `run_resource_owner_obligation_gate` も同じ raw-memory-boundary 判定で owner 診断を捨てていた。
- これは raw-memory-backed implementation の正当性を Resource IR の initialized cell / owner obligation state で証明するのではなく、source file capability を例外表として扱う挙動だった。

## 問題

run_resource_cell_gate and run_resource_owner_obligation_gate skip diagnostics solely because the diagnostic span belongs to a raw-memory-boundary source. That turns a stdlib/source capability into an exception table instead of requiring Resource IR to prove initialized-cell and free-obligation correctness.

## 影響

A raw-memory-boundary implementation can bypass initialized/moved/drop and owner obligation diagnostics, hiding use-after-move, initialized non-Copy overwrite, missing free obligation, or leak patterns exactly where memory safety needs the strongest compiler proof.

## 修正方針

Remove raw-memory-boundary suppression from Resource IR cell and owner gates. Raw-memory-boundary may not waive initialized-cell or owner-obligation invariants; any safe internal wrapper must pass the same Resource IR proof as other source. Add focused regressions so future boundary changes cannot reintroduce this bypass.

## 検証

Run focused compiler unit tests for the Resource IR gates, source policy checks, issue check, and git diff checks.

## 2026-05-13 修正

`run_resource_cell_gate` と `run_resource_owner_obligation_gate` から `SourceMap` 引数と `raw_memory_boundary_allowed` による診断抑制を削除した。これにより raw-memory-boundary source であっても、initialized / moved / dropped / maybe moved の cell state と free obligation の owner state は必ず Resource IR gate で検査される。

`nodesrc/test_static_check_boundary_responsibility.js` には、cell gate / owner gate が `SourceMap` を受け取らず、gate body 内で `raw_memory_boundary_allowed` を参照しないことを監視する source policy を追加した。

検証:

- `cargo test -p nepl-core compiler::tests::resource_ -- --nocapture`: 11/11 pass。
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass。
- `node nodesrc/issues.js check`: pass。
