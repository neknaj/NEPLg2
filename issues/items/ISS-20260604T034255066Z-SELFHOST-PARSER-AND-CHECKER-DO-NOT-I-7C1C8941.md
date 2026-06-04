---
id: ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941
title: "selfhost parser and checker do not implement full prefix expression and type range contracts"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/check/checker.nepl"
---

# ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941: selfhost parser and checker do not implement full prefix expression and type range contracts

## 概要

Subagent audit found module_parser.nepl explicitly stating it is not a full expression parser, and checker.nepl still treating later stages as unimplemented. This conflicts with plan.md, where prefix expression ranges and type-stack reduction are central to NEPLg2, and with the Zenn policy of making static checks part of the core contract rather than surface simulation.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/check/checker.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found module_parser.nepl explicitly stating it is not a full expression parser, and checker.nepl still treating later stages as unimplemented. This conflicts with plan.md, where prefix expression ranges and type-stack reduction are central to NEPLg2, and with the Zenn policy of making static checks part of the core contract rather than surface simulation.

## 影響

Selfhost compiler modules cannot validate the same prefix argument and % type ranges that the Rust compiler and Web highlighting now rely on, so stdlib/neplg2 tests can pass smoke cases while missing core language invariants.

## 修正方針

Implement or stage a real PrefixList/TypePrefixList parser boundary, connect checker range validation, and keep partial parser smoke paths marked as transitional rather than public compiler contract.

## 進捗

- 2026-06-05: `SelfhostSyntaxRange` を追加し、module declaration header が `%` type annotation range と lambda header range を typed evidence として保持するようにした。`module_parser/prefix_range.nepl` は token stream 上の flat range だけを切り、型木・式木・call boundary は parser では確定しない。
- 2026-06-05: module checker / proof solver は function 宣言で type annotation range と lambda header range が nonempty かつ header span 内にあることを検査するようにした。非 function 宣言では lambda/type range を受け付けない。
- 2026-06-05: `resolve/type_resolver` を追加し、parser の `%` annotation range から `%` marker を除いた flat type prefix item list を作る resolver input 境界を実装した。`void` は専用 marker item、`unit` は通常の named type item として分類し、まだ `TypeId` は生成しない。
- 2026-06-05: `resolve/type_resolver` の flat type prefix item list を TypeId 割当前の `resolved` tree へ縮約する reducer を追加した。`fn i32 fn i32 i32` は nonempty function type として flatten し、`fn void fn unit unit` は 0 引数 function が function を返す nested type として保持する。plan / validation / build は別 module に分割し、build 層が source string を再読しない境界にした。
- 残件: prefix expression AST、user-defined generic type constructor の kind / arity reduction、TypeId allocation、expected type / overload / generic / no partial application を含む call reduction は未実装のため、この issue は open のまま維持する。

## 検証

Add normal tests for prefix argument extent, %TypeExpr extent, nested block arguments, malformed prefix calls, and checker diagnostics once cfg-test-style tests are available.

2026-06-05 checkpoint:

- `node nodesrc/test_selfhost_parser_current_syntax_boundary.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/test_selfhost_module_checker_split_contract.js`
- `node nodesrc/test_selfhost_proof_entry_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_parser.n.md -o tmp\selfhost-prefix-parser-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_proof.n.md -o tmp\selfhost-prefix-proof-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\check -o tmp\selfhost-prefix-check-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-prefix-boundary-stdlib-neplg2.json --no-tree -j 2 --assert-io --dist web\dist`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（既存 5 warning のみ）
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-05 type resolver input checkpoint:

- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-tests2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-tests3.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 type resolver reduction checkpoint:

- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-reduce-tests-split.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-reduce-split.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-type-resolver-reduction-stdlib-neplg2-split.json --no-tree -j 2 --assert-io --dist web\dist`
