---
id: ISS-20260427T052030799Z-NM-BLOCK-PARSER-VEC-NODE-ALLOCATION--10EE8F1D
title: "nm block parser が Vec<Node> allocation failure を unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/nm/parser.nepl, tests/stdlib/nm.n.md, nodesrc/test_stdlib_nm_parser_no_block_unwraps.js"
source: ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB
---

# ISS-20260427T052030799Z-NM-BLOCK-PARSER-VEC-NODE-ALLOCATION--10EE8F1D: nm block parser が Vec<Node> allocation failure を unwrap_ok で trap する

## 概要

stdlib/nm/parser.nepl の parse_markdown と section close helper は root/children 用 Vec<Node> の生成と push を unwrap_ok で処理し、allocation failure を parser の失敗値へ戻せない。

## 対象

- `stdlib/nm/parser.nepl, tests/stdlib/nm.n.md, nodesrc/test_stdlib_nm_parser_no_block_unwraps.js`

## 根拠

parse_markdown は v::new<Node> と v::push<Node> を複数箇所で unwrap_ok し、close_one_section は section children/root への push を unwrap_ok している。

## 問題

nm block parser は docs/self-host 周辺の Markdown 処理基盤だが、block node 蓄積の allocation/grow failure で診断可能な値を返す前に trap する。

## 影響

大きい NM document や memory pressure で parser が落ち、RV-STDLIB-010 の unsafe helper debt が block parser 側に残る。

## 修正方針

Vec<Node> new/push を helper 経由の match に切り替える。失敗時は consumed owner を再利用せず空 Document/empty Vec sentinel に切り替えて解析を止め、source policy regression で block parser への unwrap_ok 再導入を防ぐ。

## 解決内容

- `NodePushRes` を追加し、block parser の `Vec<Node>` owner と push 成否を同時に返せるようにした。
- `nm_node_empty_vec` / `nm_push_node` を追加し、`v::push<Node>` の `Err` を `ok=false` と空 Vec sentinel へ変換するようにした。
- `nest_stack_push_from_hdr_result` を追加し、section stack grow の `realloc_ptr<NestSection>` 失敗を `Result` として扱うようにした。
- `close_one_section` の children/root push と `parse_markdown` の root/kids `v::new` / `v::push` から `unwrap_ok` を除去した。
- `parse_markdown` は `failed=true` で block scan を止め、失敗時に consumed owner を再利用しない空 Document sentinel へ戻す形にした。
- `nodesrc/test_stdlib_nm_parser_no_block_unwraps.js` を追加し、`nm/parser` 実装コードへの unsafe unwrap helper 再導入を source policy で固定した。
- CI/source policy と `doc/testing.md` に新しい guard を登録した。

## 検証

- `node nodesrc/test_stdlib_nm_parser_no_block_unwraps.js`: pass
- `node nodesrc/test_stdlib_nm_parser_no_inline_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl --no-tree -o tmp/nm-parser-block-allocation-docs.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-parser-block-allocation-focused.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-parser-block-allocation-suite.json -j 1`: 10/10 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-nm-parser-block-allocation.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-nm-parser-block-allocation.json -j 4`: 418/418 passed
