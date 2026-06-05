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
- 2026-06-05: `resolved` tree root を `SelfhostTypeArena` へ投影する `project.nepl` を追加した。primitive / function type は arena-local `SelfhostTypeId` を得られるようになり、named type は type constructor table 未接続のため `UnsupportedNamedType` として fail-closed にした。
- 2026-06-05: `core/ty/ty` に `SelfhostNamedTypeId` と `SelfhostTypeRecord::Named` を追加し、`resolve/type_resolver/constructor.nepl` の constructor table から arity 0 named type を `SelfhostTypeArena` へ投影できるようにした。constructor table なし API は引き続き named type を拒否し、unknown named type / bare generic constructor は typed error で fail-closed にした。
- 2026-06-05: constructor-aware reducer が constructor table header に従って generic type argument list を再帰消費し、`SelfhostResolvedTypeNode::Applied` へ縮約するようにした。constructor-aware projection は `SelfhostTypeRecord::Applied` と arena type argument table へ投影し、constructor table なし projection は Applied node を `UnsupportedNamedType` として fail-closed にする。
- 2026-06-05: `core/ty/ty/key.nepl` を追加し、arena-local `SelfhostTypeId` を payload に入れない `SelfhostCanonicalTypeKeyArena` へ type record tree を投影できるようにした。primitive / named / applied / function key node と key argument table を持ち、projection は型 record / argument edge 数に対して O(n) である。同じ key arena 内の structural equality も追加した。
- 2026-06-05: `resolve/type_resolver/typeparam` を追加し、generic binder から作る `SelfhostTypeParameterEnv` と `SelfhostTypeParameterId` を named constructor identity から分離した。`selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters` は `T` / `E` を `SelfhostResolvedTypeNode::Parameter` へ縮約し、constructor と parameter が同名の場合は `TypeParameterConstructorNameConflict` として fail-closed にする。
- 2026-06-05: `core/ty/ty` に `SelfhostTypeParameterBinding`、`SelfhostTypeRecord::Parameter`、`SelfhostCanonicalTypeKeyNode::Parameter` を追加した。type resolver projection は `SelfhostTypeParameterId` を現在 binder の `binder_depth = 0` と `parameter_index` へ正規化して TypeArena に保存し、canonical key equality は source spelling / span / arena-local `TypeId` ではなく binder identity で比較する。
- 2026-06-05: user-defined type constructor header を `SelfhostTypeConstructorKind` へ正規化し、負 arity、予約名、同一 table 内の重複名を `SelfhostTypeConstructorTableErrorKind` として登録時に拒否するようにした。constructor / type parameter lookup は `SelfhostTypeBoundPlan` に束縛し、validate と build が同じ lookup 結果を共有するため、reducer は raw `arity` や source span lookup を通常経路で繰り返さない。旧 constructor-aware validate/build helper は削除し、公開 API から bound plan を迂回できないようにした。
- 2026-06-05: constructor-aware projection でも `Applied` node の `SelfhostNamedTypeId` を constructor table で再検査し、constructor kind の型引数数と applied argument range が一致しない resolved tree を `GenericConstructorArgumentArityMismatch` として拒否するようにした。reducer 由来でない public resolved-tree constructor から不正な `SelfhostTypeRecord::Applied` が TypeArena に入る経路を閉じた。
- 2026-06-05: `syntax/ast/prefix_expr.nepl` を追加し、parser / focused test 由来の `SelfhostSyntaxRange` を pre-HIR の flat `SelfhostExprPrefixList` へ変換する入力境界を作った。`%` type annotation marker、lambda marker、`@function` marker、literal、identifier、control form marker を token index / span 付き enum payload として保持し、call tree / HIR / TypeId / DefId allocation は行わない。`void` は expression start として拒否し、legacy grouping token 混入は typed build error にする。
- 2026-06-05: `module_parser/body_range.nepl` を追加し、declaration body block の envelope と first expression segment を `SelfhostSyntaxRange` として保持するようにした。parser は body を HIR / call tree へ落とさず、単純な function body では `declaration_body.first_expression` から `SelfhostExprPrefixList` を構築できることを focused doctest で確認した。複数式 body / nested block は envelope を後段 segmenter へ渡す設計にした。
- 2026-06-05: `syntax/parser/body_segmenter.nepl` を追加し、declaration body envelope を `ExpressionLine` / `BlockIntro` の typed segment list へ分解するようにした。`ExpressionLine.head` は `SelfhostExprPrefixList` の入力候補、`BlockIntro.body` は recursive segmenter の入力として分離し、nested body を flat prefix list に直接渡さない契約を source policy と doctest で固定した。
- 2026-06-05: `check/expr` を追加し、`SelfhostExprPrefixList` と callable candidate list を受け取る call reduction の初期境界を実装した。`SelfhostTypeExpectation` は expected type の由来と span を保持し、generic inference state / overload rejection / call reduction error は enum payload で分ける。現段階では named direct call の arity / expected result / no partial application / ambiguity / generic fail-closed を検査し、HIR lowering は行わない。
- 2026-06-05: `check/expr/body_line.nepl` を追加し、`SelfhostBodySegmentKind::ExpressionLine.head` から `SelfhostExprPrefixList` を作って `selfhost_call_reduce_prefix` へ渡す接続を実装した。`BlockIntro` は `NotExpressionLine`、prefix build failure は `PrefixBuildFailed`、call reduction failure は `CallReduceFailed` として分け、nested body を flat prefix list に直接渡さない契約を source policy と focused doctest で固定した。
- 2026-06-05: `resolve/type_resolver` に先頭 1 型式だけを縮約して `next_index` を返す `selfhost_type_prefix_list_reduce_prefix*` API を追加した。`TrailingItems` を返す full annotation reducer と分け、`%i32 add 1 2` のような ascription 入力で後続 expression token を誤って型式 trailing item として拒否しない境界にした。
- 2026-06-05: `check/expr/ascription.nepl` を追加し、`%T expr` を `SelfhostTypeExpectationSource::ExplicitAscription` と内側 `SelfhostSyntaxRange` へ投影する owner 付き入口を実装した。`body_line.nepl` には arena owner を受け取る `selfhost_check_expr_reduce_body_segment_with_arena` を追加し、`%` で始まる expression line は call reduction へ直接渡さず、ascription projection 後の内側 expression だけを縮約する。
- 2026-06-05: `stage1` smoke helper に `%i32 add 1 2` の固定 token fixture を追加した。lexer / parser の詳細ではなく、type resolver が返す型式消費境界と body line connector の owner 戻しを確認する fixture とした。
- 2026-06-05: focused doctest を止めていた既存 effect 境界も修正した。`selfhost_diagnostics_push` / `selfhost_diagnostics_free` / `lex_stack_drop_top` は `Vec` owner の更新または解放を行うため `impure fn` に正規化し、`lex_stack_drop_top` は引き続き public `drop_last` API へ委譲して `Vec` 内部 storage layout へ依存しない。
- 残件: line head に対する candidate collection、argument type checking、generic instantiation inference、trait solving、ascription と外側 expected type の diagnostic 統合、`@function` / indirect call、cross-arena serialized canonical key / fingerprint、nested generic binder depth と stable binder identity は未実装のため、この issue は open のまま維持する。

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

2026-06-05 type resolver TypeId projection checkpoint:

- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-project-tests2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-project-facade2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-type-resolver-project-stdlib-neplg2.json --no-tree -j 2 --assert-io --dist web\dist`

2026-06-05 type constructor lookup checkpoint:

- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md -o tmp\selfhost-type-arena-named-tests2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-constructor-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`

2026-06-05 generic type application checkpoint:

- `node nodesrc/test_selfhost_type_record_payload.js`
- `node nodesrc/test_selfhost_type_arena_report_contract.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md -o tmp\selfhost-type-arena-applied-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-applied-shard-1-2.json --no-tree --shard 1/2 -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-applied-shard-2-2.json --no-tree --shard 2/2 -j 1 --assert-io --dist web\dist`

2026-06-05 type constructor kind validation checkpoint:

- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/test_selfhost_type_resolver_type_parameters.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_constructor_validation.n.md -o tmp\selfhost-type-constructor-validation-projection-final2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_constructor_type_parameters.n.md -o tmp\selfhost-type-constructor-type-parameters-final6.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_constructor_projection.n.md -o tmp\selfhost-type-constructor-projection-final.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-kind-bound.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 prefix expression input checkpoint:

- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/test_selfhost_parser_current_syntax_boundary.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_expr_prefix.n.md -o tmp\selfhost-expr-prefix-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax\ast\prefix_expr.nepl -o tmp\selfhost-expr-prefix-module-doctest.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax -o tmp\selfhost-syntax-focused.json --no-tree -j 2 --assert-io --dist web\dist`

2026-06-05 function body range checkpoint:

- `node nodesrc/test_selfhost_function_body_prefix_range_contract.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/test_selfhost_parser_current_syntax_boundary.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_expr_prefix.n.md -o tmp\selfhost-expr-prefix-body-range.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_parser.n.md -o tmp\selfhost-parser-body-range.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_checker.n.md -o tmp\selfhost-checker-body-range.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax\parser\module_parser.nepl -o tmp\selfhost-module-parser-facade-body-range.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 body segmenter checkpoint:

- `node nodesrc/test_selfhost_body_segmenter_contract.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/test_selfhost_function_body_prefix_range_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_body_segmenter.n.md -o tmp\selfhost-body-segmenter.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_expr_prefix.n.md -o tmp\selfhost-expr-prefix-after-segmenter.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 call reduction input checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_module_checker_split_contract.js`
- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_call_reduce.n.md -o tmp\selfhost-call-reduce.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 expression line call reduction connector checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_body_segmenter_contract.js`
- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/test_selfhost_module_checker_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_call_reduce.n.md -o tmp\selfhost-call-reduce-body-line-final.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\check\expr.nepl -o tmp\selfhost-check-expr-facade-body-line-final.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 expression ascription expectation checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（今回追加した selfhost / type resolver policy は pass。既存の `test_resource_gate_order.js` と `test_diagnostic_code_first_boundary.js` は warning）
- `node nodesrc/test_selfhost_string_helpers_boundary.js`
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_ascription_tests.json`（2/2 pass）
- `git diff --check`

2026-06-05 canonical type key checkpoint:

- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/test_selfhost_type_record_payload.js`
- `node nodesrc/test_selfhost_type_arena_report_contract.js`
- `node nodesrc/test_selfhost_type_key_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_key.n.md -o tmp\selfhost-type-key-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\ty\ty.nepl -o tmp\selfhost-ty-key-smoke.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-canonical-type-key-stdlib-neplg2.json --no-tree -j 2 --assert-io --dist web\dist`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（既存 5 warning のみ）
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-05 generic type parameter environment checkpoint:

- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/test_selfhost_type_resolver_type_parameters.js`
- `node nodesrc/test_neplg21_vec_type_arity_imports.js`
- `node nodesrc/test_nepl_doc_report_metadata_policy.js`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver\typeparam.nepl -o tmp\selfhost-typeparam-doctest.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-smoke.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver_type_parameters.n.md -o tmp\selfhost-type-resolver-type-parameters.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 binder-indexed type parameter arena/key checkpoint:

- `node nodesrc/test_selfhost_type_record_payload.js`
- `node nodesrc/test_selfhost_type_key_contract.js`
- `node nodesrc/test_selfhost_type_resolver_type_parameters.js`
- `node nodesrc/test_selfhost_type_arena_report_contract.js`
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`
- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md -o tmp\selfhost-type-arena-parameter.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_key.n.md -o tmp\selfhost-type-key-parameter.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver_type_parameters.n.md -o tmp\selfhost-type-resolver-type-parameters-projection.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_proof.n.md -o tmp\selfhost-type-proof-parameter-kind.json --no-tree -j 1 --assert-io --dist web\dist`
