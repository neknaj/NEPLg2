---
id: ISS-20260512T041752474Z-RESOURCE-OWNER-SUMMARY-REPORTS-CLIAR-97FEDA3D
title: "Resource owner summary reports cliarg raw argv scratch owners as maybe leaks"
area: static-check
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/std/env/cliarg.nepl, nepl-core/src/resource, examples/nm.nepl"
---

# ISS-20260512T041752474Z-RESOURCE-OWNER-SUMMARY-REPORTS-CLIAR-97FEDA3D: Resource owner summary reports cliarg raw argv scratch owners as maybe leaks

## 概要

After the stdio string temporary Resource IR fix, `examples/nm.nepl` still fails under Resource IR owner checking with `resource.owner.maybe_leak` in `cliarg_count__unit__i32__imp` and `cliarg_get__i32__Option_T_str__imp`. The reported owner places are `Local("meta").field0`, `Local("argv").field0`, and `Local("argv_buf").field0`, so this is a distinct cliarg raw scratch owner-flow issue rather than a stdio/ANSI temporary issue.

## 対象

- `stdlib/std/env/cliarg.nepl, nepl-core/src/resource, examples/nm.nepl`

## 根拠

- `node nodesrc/tests.js -i examples -o tmp/agent1-stdio-summary-examples-final.json -j 4 --dist web/dist --no-tree` improved the suite from stdio/ANSI-wide failure to `total=12, passed=11, failed=1`.
- The remaining failure is `examples\nm.nepl::doctest#1` during compile, with `resource.owner.maybe_leak` in `cliarg_count__unit__i32__imp` for `Local("meta").field0` and in `cliarg_get__i32__Option_T_str__imp` for `Local("meta").field0`, `Local("argv").field0`, and `Local("argv_buf").field0`.
- Existing resolved cliarg issues covered raw argv boundary capability and raw cell initialization, but this failure is specifically owner obligation propagation/consumption for internal scratch buffers after the Resource IR owner summary changes.
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

`cliarg_count` and `cliarg_get` allocate WASI argv metadata / argv buffers, route them through checked boundary helpers, and later consume them through cleanup paths. Resource IR owner checking is still seeing the field owners on `meta`, `argv`, and `argv_buf` as live at function exit. The fix must prove the exact consuming cleanup path, or expose a true leak, without reclassifying raw address integers or non-owning `MemPtr` values as owners.

## 影響

examples cannot reach a fully passing state, and WASI argv helpers do not prove that internal raw scratch allocations are consumed exactly once. Leaving this as a false positive would hide whether cliarg owner transfer/dealloc summaries are correct.

## 修正方針

Trace Resource IR owner flow for cliarg metadata and argv buffers through cli_args_sizes_result, cli_args_get_result, load/cstr conversion, and dealloc. Fix owner summary/storage-origin propagation so the actual free obligation owner is consumed by the checked cleanup path without weakening resource.owner.maybe_leak or treating non-owning MemPtr/i32 values as owners.

## 検証

trunk build; node nodesrc/tests.js -i examples -o tmp/examples-cliarg-owner-fixed.json -j 4 --dist web/dist --no-tree; focused cliarg doctests and Resource IR owner regression.
