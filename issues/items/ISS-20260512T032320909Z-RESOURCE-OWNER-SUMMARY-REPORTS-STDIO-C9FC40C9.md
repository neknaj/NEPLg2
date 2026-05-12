---
id: ISS-20260512T032320909Z-RESOURCE-OWNER-SUMMARY-REPORTS-STDIO-C9FC40C9
title: "Resource owner summary reports stdio/ANSI string temporaries as maybe leaks"
area: static-check
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/std/stdio.nepl, stdlib/platforms/wasix/tui.nepl, nepl-core/src/resource, examples"
---

# ISS-20260512T032320909Z-RESOURCE-OWNER-SUMMARY-REPORTS-STDIO-C9FC40C9: Resource owner summary reports stdio/ANSI string temporaries as maybe leaks

## 概要

After origin/main 99433272, examples failed with `resource.owner.maybe_leak` in stdio/ANSI helpers such as `print_i32__i32__unit__imp` and `ansi_text_style_code__AnsiTextStyle__str__pure`. The root cause was not stdio consuming its arguments, but Resource IR lowering not giving allocation-returning Copy `str` temporaries a statement lifetime boundary.

## 対象

- `stdlib/std/stdio.nepl, stdlib/platforms/wasix/tui.nepl, nepl-core/src/resource, examples`

## 根拠

- `node nodesrc/tests.js -i examples -o tmp/agent1-stdio-summary-examples-before.json -j 4 --dist web/dist --no-tree`: `total=12, passed=1, failed=11` with `resource.owner.maybe_leak` in stdio/ANSI string helper paths.
- `print`, `print_i32`, `ansi_text_style_code`, and `concat` take `str` as a Copy value and must not be reclassified as consuming owners. Otherwise valid code that prints or concatenates a source string would be rejected after the call.
- Resource IR owner checking already has `EndScope` as the typed boundary for owner/state cleanup. The missing piece was a line-level `EndScope` for Copy state-only `str` temporaries produced during expression lowering.
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

Resource IR treated temporary `str` values returned by helpers such as `from_i32` and ANSI style code construction as owner-bearing leaves, but those temporaries were never scoped when they were used as dropped statement results or as arguments to non-consuming stdio calls. The owner checker therefore reported `resource.owner.maybe_leak` at the callee summary boundary even though the leak was actually an unmodeled statement temporary lifetime.

## 影響

Examples and downstream stdlib users could not compile under the stricter resource checker when using ordinary stdio printing or ANSI styling. A superficial fix that made `print` consume `str` would have broken Copy string semantics and hidden real leaks behind stdlib signatures.

## 修正方針

Fixed in Resource IR lowering. For each HIR block line, lowering now records the operations emitted by that line, finds top-level temporary outputs whose type is Copy but still needs state-only resource scoping (`str`), and emits a line-end `ResourceOp::EndScope`. Non-dropped line results are preserved as the `EndScope` result, so returned values are not consumed. The temporary-scope classifier is split into `lower_temporary_scope.rs` so `lower.rs` remains responsible for HIR traversal/lowering orchestration. This keeps real `resource.owner.maybe_leak` diagnostics active and does not change stdio/ANSI APIs.

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_accepts_stdio_string_temporaries -- --nocapture`: passed. This regression covers direct `print_i32`, ANSI style `str` temporaries, and a `str` returned by `from_i32` that is bound to a local and printed twice so the line temporary scope cannot consume the surviving local value.
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: `244 passed`
- `node nodesrc/test_resource_checker_responsibility.js`: passed after splitting temporary scope classification into `lower_temporary_scope.rs`.
- `cargo fmt --check -p nepl-core`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdout.n.md -i tests/stdlib/features_tui.n.md -o tmp/agent1-stdio-summary-stdout-final.json -j 4 --dist web/dist --no-tree`: `total=12, passed=12`
- `node nodesrc/tests.js -i examples -o tmp/agent1-stdio-summary-examples-final.json -j 4 --dist web/dist --no-tree`: `total=12, passed=11, failed=1`; stdio/ANSI failures are gone. The remaining `examples/nm.nepl` cliarg raw scratch owner leak is tracked separately as [ISS-20260512T041752474Z-RESOURCE-OWNER-SUMMARY-REPORTS-CLIAR-97FEDA3D](./ISS-20260512T041752474Z-RESOURCE-OWNER-SUMMARY-REPORTS-CLIAR-97FEDA3D.md).
- `node nodesrc/issues.js check`: passed
