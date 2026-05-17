---
id: ISS-20260517T002115786Z-RESOURCE-LOWERING-COVERAGE-CLASSIFIE-EAB61912
title: "Resource lowering coverage classifies raw load projections by literal name"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/resource/coverage_hir_place.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260517T002115786Z-RESOURCE-LOWERING-COVERAGE-CLASSIFIE-EAB61912: Resource lowering coverage classifies raw load projections by literal name

## 概要

Resource IR の HIR coverage helper は、raw load の projection source を判定するときに `"load"` と `starts_with("load_")` を直接見ていた。Resource IR lowering / effect 側は `RawMemoryOp::Load` enum を中心に扱っているため、coverage だけが helper spelling へ依存する別系統の証明になっていた。

## 対象

- `nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/resource/coverage_hir_place.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `coverage_hir_projection.rs` の `raw_load_address_expr` は、intrinsic を `"load"`、direct call を `callee_is_raw_load` の `base == "load" || base.starts_with("load_")` で分類していた。
- 同じ raw memory operation の分類は `lower_raw_memory.rs` の `raw_memory_op_from_intrinsic` / `raw_memory_op_from_callee` に既に集約されている。
- coverage は lowering completeness gate の根拠なので、lowering と別の文字列分類を持つと静的検査プログラム自体の誤りを検出しにくい。

## 問題

Resource IR lowering は raw memory operation を `RawMemoryOp` で扱う一方、HIR coverage の field projection helper は raw load を literal helper 名と prefix で再分類していた。これは「静的検査プログラム自体も enum / match による検査が効く形にする」方針に反し、後続の raw helper 追加や renamed helper で coverage と lowering の認識がずれる余地を残す。

## 影響

Static-check coverage becomes dependent on textual helper spelling instead of the compiler raw-memory operation enum. That weakens the enum/match policy and can hide or misreport Resource IR lowering completeness for memory-safety-relevant field projections.

## 修正方針

Reuse the central `raw_memory_op_from_intrinsic` / `raw_memory_op_from_callee` classification in HIR coverage helpers. Add regression coverage that the aggregate field-load lowering test also runs typed coverage comparison, and add source policy that rejects literal `"load"` / `starts_with("load_")` raw-load classification in coverage.

## 検証

Run the focused Resource IR regression plus resource responsibility/source policy checks.

## 対応内容

- `coverage_hir_projection.rs` の raw load projection 判定を `RawMemoryOp::Load` enum 判定へ移した。
- `callee_is_raw_load` を削除し、direct call / intrinsic のどちらも `lower_raw_memory` の中央 classifier を使うようにした。
- `resource_ir_lowering_treats_compiler_field_load_as_field_read` に typed lowering coverage assertion を追加した。
- `nodesrc/test_resource_checker_responsibility.js` に literal helper spelling / prefix classifier の再導入禁止を追加した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_treats_compiler_field_load_as_field_read -- --exact --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
