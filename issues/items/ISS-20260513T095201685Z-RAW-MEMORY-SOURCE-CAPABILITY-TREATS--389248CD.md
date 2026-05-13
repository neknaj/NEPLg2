---
id: ISS-20260513T095201685Z-RAW-MEMORY-SOURCE-CAPABILITY-TREATS--389248CD
title: "Raw memory source capability treats shadowed helper names as evidence"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/source_capability/**, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260513T095201685Z-RAW-MEMORY-SOURCE-CAPABILITY-TREATS--389248CD: Raw memory source capability treats shadowed helper names as evidence

## 概要

The raw-memory-boundary source capability scanner classified any AST identifier named like a raw helper as evidence before name resolution. A stdlib-owned source could therefore receive raw memory capability from a parameter, local binding, or same-module safe helper named mem_ptr_addr/alloc_ptr/load_i32 even when the source performed no raw memory operation.

## 対象

- `nepl-core/src/source_capability/**, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/source_capability/raw_memory.rs` は raw-memory-boundary capability の evidence を AST から抽出するが、修正前は `PrefixItem::Symbol(Symbol::Ident(...))` の名前が `mem_ptr_addr` / `alloc_ptr` / `load_i32` などに一致するだけで evidence としていた。
- この段階は typecheck/name resolution 前なので、同名の関数引数、`let` binding、同一 module 内の safe helper と imported raw helper を区別できない。
- capability は file 単位で Resource IR effect gate の抑制条件に使われるため、単なる名前一致で付与すると「source property の証明」ではなく spelling allow に退行する。

## 問題

The raw-memory-boundary source capability scanner classified any AST identifier named like a raw helper as evidence before name resolution. A stdlib-owned source could therefore receive raw memory capability from a parameter, local binding, or same-module safe helper named mem_ptr_addr/alloc_ptr/load_i32 even when the source performed no raw memory operation.

## 影響

Raw-memory-boundary authority would be based on spelling rather than a proven source property, weakening Stage 6 static-check soundness and allowing pure unsafe-memory suppression to attach to files that only shadow raw helper names.

## 修正方針

Make the scanner track lexical and same-module shadowing for function parameters, block locals, match payload bindings, and top-level definitions. Keep raw body, intrinsic, imported raw helper call, owner helper call, and restricted constructor evidence as enum-classified cases.

## 検証

Add loader regressions that shadow mem_ptr_addr, alloc_ptr, and load_i32 without raw operations; add source policy coverage for the new scope module.

## 修正結果

- `RawMemoryBoundaryScope` を `nepl-core/src/source_capability/raw_memory_scope.rs` に分離し、raw evidence 判定と lexical shadowing 管理を分けた。
- scanner は同一 module の top-level 定義、関数/method parameter、block 内 `let` binding、match payload binding を shadowing として扱い、shadow された raw helper 名を evidence として数えない。
- raw body instruction、raw intrinsic、imported raw helper / owner helper call、`MemPtr` / `RegionToken` restricted constructor は引き続き `RawMemoryBoundaryEvidence` enum 経由で分類する。
- `nodesrc/test_static_check_boundary_responsibility.js` に新 scope module の存在、行数上限、`bind_stmt_locals` / `bind_match_pattern` の監視を追加した。

## 残件

この修正は source capability proof の誤検出を閉じるもので、Stage 6 の raw-memory-backed stdlib public API 移行全体を完了するものではない。`OwnedBuffer<T>` / owner token / safe public wrapper の整理は親 issue `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` で継続する。

## 検証結果

- `cargo test -p nepl-core loader::tests::raw_memory_boundary -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test effects raw_memory_boundary -- --nocapture`: 4 passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_resource_gate_order.js`: passed
