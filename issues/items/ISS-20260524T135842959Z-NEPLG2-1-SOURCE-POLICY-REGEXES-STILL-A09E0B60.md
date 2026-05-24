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
