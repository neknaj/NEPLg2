---
id: ISS-20260506T171738048Z-SELFHOST-MODULE-IMPORT-SPECS-STORES--9975F52D
title: "selfhost module_import_specs stores owned str payloads in Vec under strict ResourceIR"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-07
target: "stdlib/neplg2/core/module/import_spec.nepl, stdlib/neplg2/core/module/graph.nepl, stdlib/alloc/collections/vec.nepl"
---

# ISS-20260506T171738048Z-SELFHOST-MODULE-IMPORT-SPECS-STORES--9975F52D: selfhost module_import_specs stores owned str payloads in Vec under strict ResourceIR

## 概要

After lexer stale Vec construction and direct import-spec fixture leaks are fixed, tests/stdlib/neplg2_import_spec.n.md reaches selfhost_import_specs_loop and Resource IR reports use_after_move/maybe_leak for Vec<SelfhostImportSpec>. SelfhostImportSpec contains owned str path/alias values produced by str_slice, and pushing those owner payloads into raw Vec storage relies on generic Vec element drop/transfer support that is still not represented.

## 対象

- `stdlib/neplg2/core/module/import_spec.nepl, stdlib/neplg2/core/module/graph.nepl, stdlib/alloc/collections/vec.nepl`

## 根拠

- `ISS-20260506T155806325Z-SELFHOST-LEXER-AND-IMPORT-SPEC-FIXTU-68BADCC8` の直接修正後、`tests/stdlib/neplg2_import_spec.n.md` が `selfhost_import_specs_loop` まで進み、`resource.owner.use_after_move` と `resource.owner.maybe_leak` を報告した。
- 問題箇所は単なる fixture leak ではなく、`SelfhostImportSpec` が `str` owner field を保持したまま `Vec<SelfhostImportSpec>` へ push される構造にある。
- 現在の generic `Vec` は raw storage 上の要素 Drop/transfer contract を ResourceIR に十分表現できていないため、このまま owner aggregate を入れると memory safety の検査を弱める方向になる。

## 問題

After lexer stale Vec construction and direct import-spec fixture leaks are fixed, tests/stdlib/neplg2_import_spec.n.md reaches selfhost_import_specs_loop and Resource IR reports use_after_move/maybe_leak for Vec<SelfhostImportSpec>. SelfhostImportSpec contains owned str path/alias values produced by str_slice, and pushing those owner payloads into raw Vec storage relies on generic Vec element drop/transfer support that is still not represented.

## 影響

The selfhost module graph cannot safely use a Vec of import specs with owned string payloads under mandatory memory-safety checking. Keeping this shape would either hide leaks in Vec element storage or force the checker to weaken owner diagnostics.

## 修正方針

Redesign module import collection so the Vec element is Copy-only or has explicit element ownership semantics: store item indexes/ranges and resolve path/alias against the AST while it is alive, or implement typed Vec element Drop/transfer support before storing owned str payloads. Do not keep SelfhostImportSpec as a Copy aggregate containing owned str fields.

## 検証

Add focused selfhost module_import_specs/module_graph doctests that compile under strict ResourceIR, run tests/stdlib/neplg2_import_spec.n.md, and add source policy preventing Vec<SelfhostImportSpec> from carrying owned str payloads without an explicit drop contract.
