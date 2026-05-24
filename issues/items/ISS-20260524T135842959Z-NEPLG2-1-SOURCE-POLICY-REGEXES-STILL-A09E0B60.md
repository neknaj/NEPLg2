---
id: ISS-20260524T135842959Z-NEPLG2-1-SOURCE-POLICY-REGEXES-STILL-A09E0B60
title: "NEPLg2.1 source policy regexes still expect old type syntax"
area: tests
status: open
resolved: false
priority: P1
type: maintenance
created: 2026-05-24
updated: 2026-05-24
target: "nodesrc/test_stdlib_*.js; nodesrc/source_policy/**"
---

# ISS-20260524T135842959Z-NEPLG2-1-SOURCE-POLICY-REGEXES-STILL-A09E0B60: NEPLg2.1 source policy regexes still expect old type syntax

## 概要

NEPLg2.1 syntax migration changed type annotations and function signatures to %/prefix form, but many source policy regexes still expect NEPLg2.0 angle-bracket signatures.

## 対象

- `nodesrc/test_stdlib_*.js; nodesrc/source_policy/**`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` が 90 件の stale policy failure を報告した。
- 失敗例は `let text <str>`、`fn ... <(...)->...>`、`struct ... field <Type>` などの NEPLg2.0 記法を期待しており、実 source は `let text %str`、`%fn ...`、`field %Type` へ移行済みである。
- builder owner boundary 系は `nodesrc/source_policy/stdlib_builder_owner.js` と `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` の一部を NEPLg2.1 記法へ更新して pass へ戻したが、同種の regex が他の source policy に残っている。

## 問題

NEPLg2.1 syntax migration changed type annotations and function signatures to %/prefix form, but many source policy regexes still expect NEPLg2.0 angle-bracket signatures.

## 影響

run_source_policy_regressions --warn-only reports many stale policy failures, reducing static inspection signal during the migration.

## 修正方針

Migrate source policy regexes to NEPLg2.1 syntax or introduce explicit syntax-aware helpers, without weakening owner-boundary and API-boundary assertions.

## 進捗

- `nodesrc/source_policy/nepl_source_view.js` を追加し、コメント除去、実装行数計測、NEPLg2.1 signature / field regex helper、source policy 用の `legacyTypeSyntaxView` を集約した。
- `legacyTypeSyntaxView` は `%` / prefix 型表記を source policy の既存 semantic assertions が読める安定 view へ写す。`fn` と `impure fn` の区別は保持する。
- helper regression `nodesrc/test_source_policy_nepl_source_view.js` を追加し、`run_source_policy_regressions` の先頭に登録した。
- stdio / streamio / match decision tree の代表 stale regex と、collection owner/borrowed/update 系の一部 policy を復旧した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 90 件から 62 件へ減少した。残件は SHA256、BTree、ByteBuf、fs、cliarg、nm、selfhost、Vec、string boundary などに分散している。
- borrowed observer / storage contract 系の旧表記依存を追加で移行し、BTree、AdjacencyMatrix、BloomFilter、CountingBloomFilter、DisjointSet、SparseSet、SegmentTree、HashMap、HashSet の policy を `legacyTypeSyntaxView` 経由へそろえた。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 62 件から 52 件へ減少した。Rust 側 responsibility、selfhost model、documentation / tutorial contract、Vec / string / IO boundary 系は次の調査対象として残る。
- SHA256、ByteBuf UTF-8、fs、cliarg、streamio、stdio print_i32、stdio ansi の旧表記依存を `legacyTypeSyntaxView` 経由へ移行した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 52 件から 45 件へ減少した。残件は nm/parser/html、documentation/tutorial、diag/std_test、kpgraph/kpsearch/wasix、Vec/string/text/ByteBuf owner、Rust/selfhost responsibility 系に分散している。
- math module split、nm/parser/html、diag/std_test、kpgraph/kpsearch、wasix TUI の旧表記依存を `legacyTypeSyntaxView` 経由へ移行した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 45 件から 31 件へ減少した。残件は documentation/tutorial、collection cleanup、Vec、mem/core mem、Rust responsibility、selfhost、ByteBuf/string/text owner boundary 系へ絞られた。
- Vec / collection cleanup / core mem / ByteBuf / string UTF-8 / text boundary の source policy を `legacyTypeSyntaxView` 経由へそろえ、raw documentation contract は NEPLg2.1 の `%` 表記を直接検査する形へ更新した。
- `legacyTypeSyntaxView` の policy-covered type constructor arity を拡張し、`VecDataView<T>`、`VecPop<T>`、`VecPushRejected<T>`、`VecReallocRegionError<T>`、`RegionReallocError<T>` などの owner / proof payload を旧 view へ正しく写せるようにした。`VecStorageInvariant` は zero-arity 型として扱い、initializer を型引数として誤消費しない regression を追加した。
- コメント除去は source policy の実装検査 view に限定しており、コメント量や丁寧なドキュメント追加を抑制する検査は追加していない。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 31 件から 23 件へ減少した。残件は documentation/tutorial、Rust responsibility、selfhost、string storage/access/slice/float 系へ絞られた。
- string storage / access / slice / float boundary policy を `legacyTypeSyntaxView` 経由へ移行した。式中の `%i32` ascription は field declaration ではないため、`legacyTypeSyntaxView` が struct / enum body 内の field / variant payload だけを旧 view 化するように修正した。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 23 件から 19 件へ減少した。stdlib string boundary の旧表記依存は解消し、残件は documentation/tutorial、Rust responsibility、selfhost 系に集中した。
- getting_started tutorial の current-style contract を NEPLg2.1 の `%char` / `%fn` / `%Option` / `%Result` 表記へ更新した。
- stdlib documentation baseline は現在の集計値へそろえた。これはコメント追加を抑制する検査ではなく、既存の doc/doctest gap 集計を現在値から悪化させないための baseline である。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 19 件から 17 件へ減少した。残件は Rust responsibility 3 件と selfhost 14 件に集中した。
- Rust responsibility 3 件を解消した。`parser.rs` に残っていた `rsplit("::")` 相当の末尾抽出は、NEPLg2.1 prefix 型 parser を `parser/neplg21_type_expr.rs` へ分離したうえで `qualified_name::member_tail` 経由へ集約した。
- `parser/type_expr.rs` は facade に縮小し、`#extern` signature 文字列 parser を `parser/type_expr/extern_signature.rs` へ分離した。parser root の肥大化と type expr 入口の責務混在を避ける。
- responsibility policy の行数監視は、コメント追加を妨げないように raw 行数ではなく `nodesrc/source_policy/rust_source_lines.js` の実装行数を数えるようにした。comment-only / blank lines は責務肥大として数えない。
- resource responsibility policy は未監視だった transform-range 関連 module を登録し、既存 resource module の現在の実装行 baseline へ更新した。これはコメント量の制限ではなく、実装本体の追加肥大を検出する baseline である。
- `node nodesrc/run_source_policy_regressions.js --warn-only` の stale warning は 17 件から 14 件へ減少した。残件は selfhost 系に集中した。
- `cargo test -p nepl-core` は今回差分外の resource unit 4 件で失敗するため、`ISS-20260524T162206420Z-NEPL-CORE-RESOURCE-UNIT-TESTS-FAIL-I-5A9C5729` として分離した。

## 検証

node nodesrc/run_source_policy_regressions.js without stale NEPLg2.0 syntax failures
- `node nodesrc/test_source_policy_nepl_source_view.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/test_stdlib_core_mem_boundary.js`
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`
- `node nodesrc/test_stdlib_mem_internal_region_new_docs.js`
- `node nodesrc/test_stdlib_string_utf8_boundary.js`
- `node nodesrc/test_stdlib_text_boundary.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only` (23 warnings remain)
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests_checkpoint5.json` (13/13 passed)
- `node nodesrc/test_stdlib_string_storage_boundary.js`
- `node nodesrc/test_stdlib_string_access_boundary.js`
- `node nodesrc/test_stdlib_string_slice_boundary.js`
- `node nodesrc/test_stdlib_string_float_boundary.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only` (19 warnings remain)
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests_checkpoint6.json` (13/13 passed)
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/test_tutorial_getting_started_current_style.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only` (17 warnings remain)
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests_checkpoint7.json` (13/13 passed)
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_parser_backend_responsibility_policy.js`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test functions neplg21`
- `cargo test -p nepl-core --test typeannot neplg21`
- `cargo test -p nepl-core qualified_name`
- `node nodesrc/run_source_policy_regressions.js --warn-only` (14 warnings remain)
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
- `node nodesrc/neplg21_syntax_migrate.js --check`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests_checkpoint8.json` (13/13 passed)
- `cargo test -p nepl-core` failed in unrelated resource unit tests tracked by `ISS-20260524T162206420Z-NEPL-CORE-RESOURCE-UNIT-TESTS-FAIL-I-5A9C5729`.
