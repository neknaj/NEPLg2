---
id: ISS-20260513T153534522Z-RESOURCE-BRANCH-RESULT-REGRESSIONS-S-63F812E4
title: "Resource branch result regressions still use stale raw cleanup assumptions"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260513T153534522Z-RESOURCE-BRANCH-RESULT-REGRESSIONS-S-63F812E4: Resource branch result regressions still use stale raw cleanup assumptions

## 概要

The resource_ir_owner_check_preserves_branch_result_variant_owner_return and resource_ir_owner_check_preserves_branch_result_from_owner_returning_helper tests fail on clean main. They exercise branch-local Result owner return propagation but still use dealloc_raw mem_ptr_addr for MemPtr cleanup in error paths, which conflicts with the current MemPtr/non-owning pointer and typed owner cleanup policy.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_branch_result -- --nocapture` failed on this branch after `ISS-20260513T151511232Z-RESOURCE-OWNER-ALIAS-RESOLUTION-CAN--CB5B7B73` was fixed.
- The same command was checked against clean main before the fix and failed with the same two tests, so this is not a regression from the returned-owner materialization fix.
- The failing snippets call `dealloc_raw mem_ptr_addr out 3` on a `MemPtr` owner in error paths. Current Resource IR policy treats `MemPtr` as non-owning pointer projection and requires typed owner cleanup (`dealloc_ptr`) or a valid raw-memory-boundary proof for storage-only raw cleanup.

## 問題

The resource_ir_owner_check_preserves_branch_result_variant_owner_return and resource_ir_owner_check_preserves_branch_result_from_owner_returning_helper tests fail on clean main. They exercise branch-local Result owner return propagation but still use dealloc_raw mem_ptr_addr for MemPtr cleanup in error paths, which conflicts with the current MemPtr/non-owning pointer and typed owner cleanup policy.

## 影響

These stale regressions make the Resource IR owner test filter fail even after the stdio/fs returned-owner false positive is fixed. They can hide real owner-summary failures and keep full resource_ir review noisy.

## 修正方針

Audit the two tests against the current memory model. If their intent is still branch-local Result owner propagation, rewrite cleanup paths to use typed owner-consuming dealloc_ptr or otherwise model a valid raw-memory-boundary proof. Keep the branch Result owner return assertions as focused regressions.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_branch_result -- --nocapture should pass after the tests or checker are corrected.
