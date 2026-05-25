---
id: ISS-20260525T154712899Z-RESOURCE-IR-RETURN-PATH-REPLAY-UNIT--010D6DB7
title: "Resource IR return-path replay unit test reports callee dealloc proof instead of caller live slot"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-25
updated: 2026-05-25
target: "nepl-core/tests/resource_ir.rs; nepl-core/src/resource/collection_slot_summary_*; nepl-core/src/resource/initialized*.rs"
---

# ISS-20260525T154712899Z-RESOURCE-IR-RETURN-PATH-REPLAY-UNIT--010D6DB7: Resource IR return-path replay unit test reports callee dealloc proof instead of caller live slot

## 概要

The clean HEAD baseline fails resource_ir_collection_slot_return_path_state_only_replay_does_not_duplicate_diagnostics. The report contains StorageDeallocRequiresRawReleaseProof on the callee parameter instead of the expected caller LiveSlotDuringStorageDealloc diagnostic produced by state-only return-path replay.

## 対象

- `nepl-core/tests/resource_ir.rs; nepl-core/src/resource/collection_slot_summary_*; nepl-core/src/resource/initialized*.rs`

## 根拠

- 未記入

## 問題

The clean HEAD baseline fails resource_ir_collection_slot_return_path_state_only_replay_does_not_duplicate_diagnostics. The report contains StorageDeallocRequiresRawReleaseProof on the callee parameter instead of the expected caller LiveSlotDuringStorageDealloc diagnostic produced by state-only return-path replay.

## 影響

Resource IR return-path summary verification is not fully green, so NEPLg2.1 cannot honestly claim a clean resource_ir integration baseline before main merge.

## 修正方針

Audit collection-slot summary construction for invalid callee storage dealloc, distinguish callee-side summary diagnostics from call-site replay diagnostics, and keep state-only return-path replay from duplicating or masking the caller-specific violation.

## 検証

Run the clean-head failing focused resource_ir return_path test, then the broader resource_ir return-path and collection-slot summary tests after the implementation is corrected.
