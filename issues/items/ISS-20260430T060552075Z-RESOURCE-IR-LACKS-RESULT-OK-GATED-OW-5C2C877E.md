---
id: ISS-20260430T060552075Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-5C2C877E
title: "Resource IR lacks Result::Ok-gated owner consumption summaries for checked MemPtr frees"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260430T060552075Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-5C2C877E: Resource IR lacks Result::Ok-gated owner consumption summaries for checked MemPtr frees

## 概要

Checked MemPtr free/realloc wrappers return Result and consume the storage owner only on Result::Ok. After call-site RawMemory lowering is reserved for direct raw operations, dealloc_ptr p in a Result::Ok arm leaves the p.raw owner obligation live, while restoring unconditional RawMemory::Dealloc at the wrapper call would consume the owner even on Err.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- 未記入

## 問題

Checked MemPtr free/realloc wrappers return Result and consume the storage owner only on Result::Ok. After call-site RawMemory lowering is reserved for direct raw operations, dealloc_ptr p in a Result::Ok arm leaves the p.raw owner obligation live, while restoring unconditional RawMemory::Dealloc at the wrapper call would consume the owner even on Err.

## 影響

Valid checked cleanup code is rejected with resource.owner.maybe_leak, and the unsafe alternative would hide double-free/leak paths on failed checked frees. This blocks strict memory-safety validation for core/mem and self-host storage cleanup.

## 修正方針

Add enum-variant-gated owner summaries parallel to initialized-cell variant summaries. Summarize owner consumption/return per Result::Ok/Err branch for checked MemPtr dealloc/realloc APIs, record the pending owner effect at the call result, and apply it only inside matching match arms.

## 検証

Add Resource IR regressions for dealloc_ptr/realloc_ptr where Ok consumes the owner and Err preserves it. Re-run tests/stdlib/memory_safety.n.md cleanup cases and owner_check focused tests.
