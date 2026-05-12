---
id: ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573
title: "Import visibility does not enforce pub private item boundaries"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resolve.rs, nepl-core/src/typecheck/env.rs, nepl-core/src/typecheck/name_lookup.rs, stdlib/core/mem.nepl"
---

# ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573: Import visibility does not enforce pub private item boundaries

## 概要

ImportResolution records which files are imported but Binding does not carry item visibility, so imported private definitions are still selectable through open imports. This blocks the Stage 6 core/mem split because moving raw memory helpers behind non-public imports would not actually hide them from user source.

## 対象

- `nepl-core/src/resolve.rs, nepl-core/src/typecheck/env.rs, nepl-core/src/typecheck/name_lookup.rs, stdlib/core/mem.nepl`

## 根拠

- `nepl-core/src/parser.rs` の `parse_visibility` は `pub` がない top-level item を `Visibility::Private` として parse している。
- `nepl-core/src/module_graph.rs` の host-side export table は `Visibility::Pub` だけを export し、`non_pub_import_does_not_reexport` regression も持っている。
- しかし実際の compile pipeline は flat loader representation を typecheck し、`nepl-core/src/typecheck/env.rs` の `Binding` は item visibility を保持していない。
- `nepl-core/src/typecheck/name_lookup.rs` は `ImportResolution::binding_is_visible_unqualified` へ source file / binding file / name だけを渡すため、対象 binding が private かどうかを判定できない。
- `nepl-core/src/resolve.rs` の `UnqualifiedImportVisibility::All` は `binding_name == name` だけで imported binding を許可する。ここにも `Visibility::Pub` 条件がない。
- `stdlib/core/mem.nepl` の `mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` / generic `load` / `store` は現在 `fn` で定義されているが、module boundary だけでは user import から隠せない。

## 問題

ImportResolution records which files are imported but Binding does not carry item visibility, so imported private definitions are still selectable through open imports. This blocks the Stage 6 core/mem split because moving raw memory helpers behind non-public imports would not actually hide them from user source.

## 影響

Raw allocator, raw load/store, mem_ptr_addr, mem_ptr_wrap, and token construction helpers cannot be made internal by module refactoring alone. Safe user source can continue to reach raw memory discipline even if stdlib files are reorganized, so static memory safety remains dependent on later Resource IR diagnostics instead of the module boundary.

## 修正方針

Make visibility part of the typecheck binding authority. Imported cross-file bindings must require Visibility::Pub unless the source has an explicit compiler/internal capability or the access is same-file. Add compile_fail regressions for private import access, then split core/mem into public safe facade and internal raw-memory-boundary modules.

実装は次の順で行う。

1. `Binding` に top-level item visibility を表す typed field を追加する。local binding と top-level item を混同しないため、必要であれば `BindingVisibility` enum を導入する。
2. `FnDef` / `FnAlias` / `StructDef` constructor / `EnumDef` variant constructor / extern などの binding 登録時に、source item の visibility を保持する。
3. unqualified / qualified lookup の cross-file access で `Visibility::Pub` を要求する。同一 file 内の private access は維持する。
4. `#import "x" as *`、qualified import、selective import、`pub #import` re-export について compile-level regression を追加し、host-only `module_graph` test と typecheck behavior がずれないようにする。
5. その後 `core/mem` の Stage 6 分割に入り、raw helper module は internal raw-memory-boundary capability 内からだけ使う。public facade は `MemPtr` view、owner token、initialized cell 操作を分ける。

## 検証

Add compiler tests proving a private function in an imported module is not callable through as * or qualified import, while pub re-exports remain callable. Add core/mem migration tests proving raw helper access is limited to internal raw-memory-boundary sources.

関連:

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / mem / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
- [ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D](./ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D.md)
- [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)
