---
id: ISS-20260517T004939361Z-SOURCE-CAPABILITY-PROOF-IS-STILL-CON-C609583C
title: "Source capability proof is still consumed as file-scoped privilege instead of use-site proof"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/proof.rs, nepl-core/src/source_map.rs, nepl-core/src/compiler.rs, nepl-core/src/typecheck/effect_check.rs, nodesrc/test_static_check_boundary_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T004939361Z-SOURCE-CAPABILITY-PROOF-IS-STILL-CON-C609583C: Source capability proof is still consumed as file-scoped privilege instead of use-site proof

## 概要

SourceCapabilityProof is now collected through one generic source traversal, but it is collapsed into SourceCapabilities as a file-level enum set. Compiler and typecheck gates suppress raw memory, raw address, and checked MemPtr diagnostics by file_id plus operation instead of consuming a proof tied to the exact privileged use site.

## 対象

- `nepl-core/src/source_capability/proof.rs, nepl-core/src/source_map.rs, nepl-core/src/compiler.rs, nepl-core/src/typecheck/effect_check.rs, nodesrc/test_static_check_boundary_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `source_capability/proof.rs` は `SourceCapabilityProofCollector` で source evidence を単一 traversal から集めている。
- `source_map.rs` の `SourceCapabilities` は `BTreeSet<SourceCapability>` を file に保持し、`raw_memory_operation_boundary_allowed(id, operation)` は file id と operation だけで判定する。
- `compiler.rs` の `resource_effect_boundary_diagnostic_is_raw_boundary_allowed` は `RawMemoryOutsideBoundary` / `CheckedMemPtrOutsideBoundary` / `RawAddressViewOutsideBoundary` を diagnostic span の file capability で抑制する。
- `typecheck/effect_check.rs` も raw body / raw helper call の許可を `span.file_id` から source map に問い合わせており、exact use-site proof artifact を消費していない。

## 問題

SourceCapabilityProof is now collected through one generic source traversal, but it is collapsed into SourceCapabilities as a file-level enum set. Compiler and typecheck gates suppress raw memory, raw address, and checked MemPtr diagnostics by file_id plus operation instead of consuming a proof tied to the exact privileged use site.

## 影響

A configured stdlib source file that proves one raw operation can allow the same operation elsewhere in the file without an exact local proof. This is safer than a path-only allowlist, but still weaker than the intended generic proof model and makes static-check implementation mistakes harder to catch through typed proof exhaustiveness.

## 修正方針

Separate compiler-owned source eligibility from privileged-use proof. Introduce a typed source proof artifact keyed by file, span or lexical use id, and operation/domain. Make Resource IR and typecheck gates query the exact proof event for raw operation, raw body, raw address view, owner aggregate constructor/field, and compiler memory definition uses. Add source-policy guards against returning to file-level raw boundary checks as the sole authority.

## 検証

Add compiler regressions where one stdlib file contains a proven raw helper plus an unrelated unproven raw use and require the unrelated use to fail. Keep user source denied even with helper-looking names. Run focused source capability tests, Resource IR effect gate tests, static-check boundary policy, issues check, and cargo check.
