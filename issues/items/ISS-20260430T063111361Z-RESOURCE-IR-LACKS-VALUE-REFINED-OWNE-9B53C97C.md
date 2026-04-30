---
id: ISS-20260430T063111361Z-RESOURCE-IR-LACKS-VALUE-REFINED-OWNE-9B53C97C
title: "Resource IR lacks value-refined owner returns for realloc Result::Ok payloads"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl"
---

# ISS-20260430T063111361Z-RESOURCE-IR-LACKS-VALUE-REFINED-OWNE-9B53C97C: Resource IR lacks value-refined owner returns for realloc Result::Ok payloads

## 概要

realloc/realloc_ptr merges Result::Ok(0) for new_size <= 0 with Result::Ok(new_ptr) for successful growth. The current owner summary is keyed only by enum variant, so it cannot distinguish the Ok payload that carries a transferred owner from the Ok payload that carries no free obligation.

## 対象

- `nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl`

## 根拠

- 未記入

## 問題

realloc/realloc_ptr merges Result::Ok(0) for new_size <= 0 with Result::Ok(new_ptr) for successful growth. The current owner summary is keyed only by enum variant, so it cannot distinguish the Ok payload that carries a transferred owner from the Ok payload that carries no free obligation.

## 影響

Checked realloc wrappers either reject valid positive-size cleanup with resource.owner.maybe_leak/OwnerUnavailable, or would become unsound if Ok payload ownership were marked unconditionally. This blocks precise memory-safe realloc use without weakening owner checks.

## 修正方針

Add value-refined owner return summaries or split the realloc API contract so the owner-carrying Ok payload is represented separately from zero-size deallocation. The caller-side summary application must transfer the old owner to the returned MemPtr only when the success payload is proven owner-carrying.

## 検証

Add Resource IR regressions for realloc_ptr p old_size positive_new_size: Ok transfers the old owner to q and Err preserves p; also cover new_size <= 0 so Ok(0) does not create a fake owner.
