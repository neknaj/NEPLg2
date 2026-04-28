---
id: ISS-20260428T163754109Z-SELF-HOST-TYPE-STAGE-LACKS-TYPEID-AN-5345670C
title: "self-host type stage lacks TypeId and TypeArena"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/ty/ty.nepl, tests/stdlib/neplg2_type_arena.n.md"
---

# ISS-20260428T163754109Z-SELF-HOST-TYPE-STAGE-LACKS-TYPEID-AN-5345670C: self-host type stage lacks TypeId and TypeArena

## 概要

S3 の type/check stage に入るための TypeId / TypeKind / TypeArena がまだ marker API だけで、parser 以降の型情報を stable id として保持できない。

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, tests/stdlib/neplg2_type_arena.n.md`

## 根拠

- `doc/neplg2/self_host_execution_plan.md` の S3 checkpoint は `selfhost/s3-type-arena` を TypeId、TypeKind、arena、subst の入口としている。
- `stdlib/neplg2/core/ty/ty.nepl` は marker API だけで、型を arena-local stable id として保持する table を持っていなかった。
- S3 の resolver / checker / unify は、parser AST から得た型情報を直接値として持つのではなく、arena-local `TypeId` で参照できる必要がある。

## 問題

S3 の type/check stage に入るための TypeId / TypeKind / TypeArena がまだ marker API だけで、parser 以降の型情報を stable id として保持できない。

## 影響

unify、subst、resolver、checker が型を値ごとに直接持つ設計へ流れ、self-host compiler の型推論と diagnostics が Rust 実装との比較対象を持てない。

## 修正方針

filesystem や static/resource checker に依存しない最小 TypeId / TypeKind / TypeArena を stdlib/neplg2/core/ty に追加し、primitive type と function type の登録、lookup、arity access の回帰を固定する。

## 対応

- `SelfhostTypeId`、`SelfhostTypeKind`、`SelfhostTypeRecord`、`SelfhostTypeArena` を追加した。
- function type の引数列は arena の `function_args` table に集約し、record は `first_arg` / `arg_count` / `result` の小さい Copy 値だけを持つ設計にした。
- primitive type 登録、function type 登録、kind lookup、function arity / argument / result access を追加した。
- function 引数列の複製中に範囲外が起きた場合も、所有中の argument buffer を解放してから `IndexOutOfBounds` を返すようにした。
- `tests/stdlib/neplg2_type_arena.n.md` に primitive stable id、function argument/result lookup、invalid/non-function access の回帰を追加した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/neplg2-type-arena-doctest.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg2-type-arena-focused.json -j 1`: total=3 passed=3
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_type_arena.n.md -i tests/stdlib/neplg2_stdlib_map.n.md -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-type-arena-syntax.json -j 1`: total=56 passed=56
- `origin/main` の `1099d02 fix(core): gate incomplete resource lowering` まで rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/neplg2-type-arena-doctest-after-rebase.json -j 1`: total=1 passed=1
- rebase 後、`node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg2-type-arena-focused-after-rebase.json -j 1`: total=3 passed=3
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_type_arena.n.md -i tests/stdlib/neplg2_stdlib_map.n.md -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-type-arena-syntax-after-rebase.json -j 1`: total=56 passed=56
- `origin/main` の `f372221 fix(core): lower while returns for resource coverage` まで rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/neplg2-type-arena-doctest-after-rebase-f372221.json -j 1`: total=1 passed=1
- rebase 後、`node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/neplg2-type-arena-focused-after-rebase-f372221.json -j 1`: total=3 passed=3
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_type_arena.n.md -i tests/stdlib/neplg2_stdlib_map.n.md -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-type-arena-syntax-after-rebase-f372221.json -j 1`: total=56 passed=56
