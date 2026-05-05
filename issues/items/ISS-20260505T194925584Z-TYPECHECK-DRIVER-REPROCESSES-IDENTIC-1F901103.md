---
id: ISS-20260505T194925584Z-TYPECHECK-DRIVER-REPROCESSES-IDENTIC-1F901103
title: "typecheck driver reprocesses identical imported definitions as item conflicts"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/typecheck/driver.rs, nepl-core/tests/neplg2.rs, nepl-core/tests/kp.rs, issues/items/ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700.md"
---

# ISS-20260505T194925584Z-TYPECHECK-DRIVER-REPROCESSES-IDENTIC-1F901103: typecheck driver reprocesses identical imported definitions as item conflicts

## 概要

Import expansion can place the same stdlib top-level definition in module.root.items multiple times with the same Span. The typecheck driver processes those identical definitions repeatedly, producing resolve.item.name_conflict and type.impl.duplicate_for_trait_target before Resource IR scanner regressions can run.

## 対象

- `nepl-core/src/typecheck/driver.rs, nepl-core/tests/neplg2.rs, nepl-core/tests/kp.rs, issues/items/ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700.md`

## 根拠

- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture` が、Resource IR に入る前に `resolve.item.name_conflict` と `type.impl.duplicate_for_trait_target` を大量に出して失敗した。
- diagnostics の primary span は stdlib 側の同一 file / 同一 range で繰り返されており、異なる source definition の衝突ではなく、import 展開後の同一定義再処理だった。
- `find_same_signature_func_in_file` の修正だけでは enum/struct/impl/function-body generation 側の再処理は止まらないため、driver pass ごとの top-level definition identity を明示的に dedup する必要があった。

## 問題

Import expansion can place the same stdlib top-level definition in module.root.items multiple times with the same Span. The typecheck driver processes those identical definitions repeatedly, producing resolve.item.name_conflict and type.impl.duplicate_for_trait_target before Resource IR scanner regressions can run.

## 影響

Valid source that imports overlapping stdlib modules is rejected by duplicate diagnostics. This hides Resource IR initialized-range failures behind resolver noise and weakens diagnostic correctness.

## 修正方針

Deduplicate top-level definition processing by definition Span inside typecheck driver passes. Keep different-span duplicates as real errors, but skip exact same definition identity for declaration, impl collection, function registration, function body generation, and final impl generation.

## 検証

Add a regression that imports overlapping stdlib modules and ensure duplicate item/impl diagnostics are not emitted. Re-run local_scanner_new_logic_debug and focused Resource IR/source policy tests.

## 2026-05-06 対応結果

`typecheck/driver.rs` に top-level definition の `Span` identity dedup を追加し、同一 imported definition を declaration collection、impl collection、callable registration、function body generation、final impl generation で一度だけ処理するようにした。

異なる `Span` の同名 item / duplicate impl は従来どおり診断する。今回 skip するのは exact same source definition が import graph 上で再観測された場合だけであり、重複診断を握りつぶすものではない。

回帰として `stdlib_overlapping_imports_do_not_reprocess_same_top_level_definitions` を追加し、`core/field`、`core/mem`、`std/stdio` の重なった import が `resolve.item.name_conflict` / `type.impl.duplicate_for_trait_target` を出さないことを固定した。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test neplg2 stdlib_overlapping_imports_do_not_reprocess_same_top_level_definitions -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: duplicate import diagnostics は解消。次の別件として `stdio_write_fd_mem_result` の `resource.owner.maybe_leak` を検出したため、`ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700` を追加した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
