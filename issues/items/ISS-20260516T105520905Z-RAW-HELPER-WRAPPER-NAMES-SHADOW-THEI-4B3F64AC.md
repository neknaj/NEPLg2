---
id: ISS-20260516T105520905Z-RAW-HELPER-WRAPPER-NAMES-SHADOW-THEI-4B3F64AC
title: "Raw helper wrapper names shadow their own raw operation evidence"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-17
target: nepl-core/src/source_capability/proof.rs
---

# ISS-20260516T105520905Z-RAW-HELPER-WRAPPER-NAMES-SHADOW-THEI-4B3F64AC: Raw helper wrapper names shadow their own raw operation evidence

## 概要

After owner-field boundary fixes, adjacency_matrix/create doctest reaches core/mem/pointer/scalar.nepl where wrappers named load_u8/store_u8 and core/mem/pointer/bulk.nepl wrapper mem_copy call the raw primitive with the same name. SourceCapabilityScope binds top-level function names before walking bodies, so the call-head raw primitive evidence is treated as shadowed and RawMemoryOperationBoundary is not granted.

## 対象

- `nepl-core/src/source_capability/proof.rs`

## 根拠

- `stdlib/alloc/collections/adjacency_matrix/api/create.nepl` の doctest を現行 TestReport 形式へ更新したところ、`core/mem/pointer/scalar.nepl` の `load_u8` / `store_u8` と `core/mem/pointer/bulk.nepl` の `mem_copy` が `effect.pure.calls_impure` / `resource.raw.memory_outside_boundary` で拒否された。
- `SourceCapabilityScope::from_module` は top-level 関数名も shadow として登録するため、同名 raw helper wrapper 内の raw primitive call-head が local shadow と同じ扱いで落とされていた。

## 問題

After owner-field boundary fixes, adjacency_matrix/create doctest reaches core/mem/pointer/scalar.nepl where wrappers named load_u8/store_u8 and core/mem/pointer/bulk.nepl wrapper mem_copy call the raw primitive with the same name. SourceCapabilityScope binds top-level function names before walking bodies, so the call-head raw primitive evidence is treated as shadowed and RawMemoryOperationBoundary is not granted.

## 影響

Compiler-owned raw helper wrappers are rejected by effect.pure.calls_impure and resource.raw.memory_outside_boundary even though their source contains direct raw operation evidence; downstream stdlib doctests fail before Resource IR can validate the caller flow.

## 修正方針

Record raw operation evidence for a named function's own raw helper symbol when the shadowing symbol is the current function name, while still rejecting unrelated local/parameter/qualifier shadowing and keeping operation-specific capabilities.

## 検証

Add loader regressions for same-name raw helper wrappers and unrelated shadow rejection, run raw_memory_boundary tests, static-check source policy, and focused adjacency_matrix create doctest.

## 対応

- `SourceCapabilityScope` の binding を `TopLevelCallable` と `LocalValue` に分け、top-level callable shadow と local/parameter/match binding shadow を区別するようにした。
- `raw_evidence_gate.rs` を追加し、同名 raw helper wrapper の raw primitive 証拠を「現在走査中の関数名と一致する top-level callable で、かつ中央 raw helper registry に存在する symbol」の場合だけ許可するようにした。
- local shadow / parameter shadow / unrelated qualified shadow は引き続き raw operation capability evidence にならない。
- loader regression と source policy で、同名 wrapper positive case と local shadow negative case を固定した。

## 検証結果

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core raw_memory_boundary_accepts_same_name_raw_helper_wrapper_evidence -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary_rejects_local_shadow_inside_same_name_raw_helper -- --nocapture`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -n 1 --dist web/dist`: raw capability diagnostic は消え、次の doctest exit-code contract 問題に進んだ。
