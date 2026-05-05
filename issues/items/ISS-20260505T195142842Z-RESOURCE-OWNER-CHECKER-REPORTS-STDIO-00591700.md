---
id: ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700
title: "Resource owner checker reports stdio fd_write scratch MaybeLeak after duplicate import cleanup"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_control.rs, stdlib/std/stdio/write.nepl, nepl-core/tests/kp.rs"
---

# ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700: Resource owner checker reports stdio fd_write scratch MaybeLeak after duplicate import cleanup

## 概要

After identical imported definitions are no longer reprocessed, local_scanner_new_logic_debug reaches Resource IR owner checking and fails in stdio_write_fd_mem_result with resource.owner.maybe_leak for the iov and nwritten scratch owners on the fd_write loop boundary.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_control.rs, stdlib/std/stdio/write.nepl, nepl-core/tests/kp.rs`

## 根拠

- 未記入

## 問題

After identical imported definitions are no longer reprocessed, local_scanner_new_logic_debug reaches Resource IR owner checking and fails in stdio_write_fd_mem_result with resource.owner.maybe_leak for the iov and nwritten scratch owners on the fd_write loop boundary.

## 影響

KP/scanner-style programs remain blocked before runtime. The failure suggests variant-gated owner cleanup or loop/match owner state around stdio fd_write scratch storage is still too conservative or the stdio write boundary needs an explicit Resource IR-safe ownership shape.

## 修正方針

Audit stdio_write_fd_mem_result together with Resource IR owner loop/match merge. Preserve the rule that scratch owners must be freed on all paths, then either make the checker prove the existing cleanup or redesign the stdio write boundary without weakening owner obligations.

## 検証

Re-run cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture and focused owner Resource IR regressions. Add a regression that stdio_write_fd_mem_result does not report MaybeLeak for iov/nwritten when cleanup paths are exhaustive.
