---
id: ISS-20260428T173941917Z-SELF-HOST-TYPE-STAGE-LACKS-STRUCTURA-F41EF07F
title: "self-host type stage lacks structural type equality"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/neplg2/core/ty/ty.nepl, tests/stdlib/neplg2_type_arena.n.md"
---

# ISS-20260428T173941917Z-SELF-HOST-TYPE-STAGE-LACKS-STRUCTURA-F41EF07F: self-host type stage lacks structural type equality

## 概要

TypeArena can allocate primitive and function TypeId values, but only index equality is available. S3 unify/checker code cannot compare reconstructed type shapes without falling back to ad hoc record inspection.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, tests/stdlib/neplg2_type_arena.n.md`

## 根拠

- `doc/neplg2/self_host_execution_plan.md` の S3 は type/check stage を `TypeId` / `TypeKind` / arena / subst の入口としている。
- `ISS-20260428T163754109Z-SELF-HOST-TYPE-STAGE-LACKS-TYPEID-AN-5345670C` で TypeArena は追加されたが、`selfhost_type_id_eq` は arena index の比較だけだった。
- unify / overload / checker は「同じ index か」ではなく「同じ型構造か」を判定する必要があり、function type の引数列と戻り値を arena record から辿る共通 API が必要だった。

## 問題

TypeArena can allocate primitive and function TypeId values, but only index equality is available. S3 unify/checker code cannot compare reconstructed type shapes without falling back to ad hoc record inspection.

## 影響

unify, overload, and checker work would duplicate structural comparison logic and could confuse same-shape types allocated at different arena indices with genuinely different types.

## 修正方針

Add a TypeArena structural equality API that compares primitive kinds and function argument/result shapes through arena records, then add regression tests for same-shape function types and mismatched arity/result.

## 対応

- `selfhost_type_arena_types_equal` を追加し、同じ arena 内の valid `TypeId` を構造比較できるようにした。
- primitive type は `SelfhostTypeKind` の一致で比較し、function type は arity、各引数 TypeId、戻り値 TypeId を再帰的に比較するようにした。
- invalid / out-of-range TypeId は、同じ invalid index 同士でも false にした。
- `tests/stdlib/neplg2_type_arena.n.md` に、別 index の同形 function type が true になる回帰と、引数型 / arity / 戻り値 / invalid TypeId mismatch が false になる回帰を追加した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/neplg2-type-equality-doctest.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg2-type-equality-focused.json -j 1`: total=5 passed=5
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_type_arena.n.md -i tests/stdlib/neplg2_stdlib_map.n.md -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-type-equality-syntax.json -j 1`: total=58 passed=58
- `origin/main` `2cdcf45` へ rebase 後、`node nodesrc\tests.js -i stdlib\neplg2\core\ty\ty.nepl --no-tree -o tmp\neplg2-type-equality-doctest-main-rebase.json -j 1`: total=1 passed=1
- rebase 後、`node nodesrc\tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\neplg2-type-equality-focused-main-rebase.json -j 1`: total=5 passed=5
- rebase 後、`node nodesrc\tests.js -i stdlib\neplg2\core\ty\ty.nepl -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\neplg2-type-equality-ty-focused-main-rebase.json -j 1`: total=6 passed=6
- 現在の self-host broad command は `ISS-20260428T184502533Z-SELF-HOST-IMPORT-SPEC-TEST-OVERFLOWS-BDC6F326` の wasm codegen stack overflow が別 issue として残るため、type equality の完了判定からは分離した。
