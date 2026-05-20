---
id: ISS-20260520T045142622Z-SELF-HOST-NAME-RESOLVER-REMAINS-A-FL-8D5B56B9
title: "self-host name resolver remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/resolve/name_resolver.nepl; stdlib/neplg2/core/resolve/name_resolver/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T045142622Z-SELF-HOST-NAME-RESOLVER-REMAINS-A-FL-8D5B56B9: self-host name resolver remains a flat implementation file

## 概要

Selfhost name_resolver.nepl keeps DefId, DefKind, binding payload, scope owner operations, lookup loops, and stage smoke in one file. This makes the resolve stage harder to extend toward hoist/import/visibility without repeating Rust root-file flattening.

## 対象

- `stdlib/neplg2/core/resolve/name_resolver.nepl; stdlib/neplg2/core/resolve/name_resolver/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は resolve stage を `def_id`、`def_table`、`scope`、`hoist`、`import_resolver`、`name_resolver` へ分ける方針を持っている。
- `stdlib/neplg2/core/resolve/name_resolver.nepl` は分割前に DefId、DefKind、binding payload、scope owner operation、lookup loops、stage0 smoke を同じ file に持っていた。
- 今後 hoist、import visibility、trait/generic name resolution を追加する前に、DefId / DefKind / binding / scope を file 単位で分ける必要があった。

## 問題

Selfhost name_resolver.nepl keeps DefId, DefKind, binding payload, scope owner operations, lookup loops, and stage smoke in one file. This makes the resolve stage harder to extend toward hoist/import/visibility without repeating Rust root-file flattening.

## 影響

Future resolver, abstraction, trait, and checker work will add unrelated logic to name_resolver.nepl, weakening typed DefId/DefKind policies and making source policy regressions less precise.

## 修正方針

Keep name_resolver.nepl as a documentation/public facade, move implementation into name_resolver/id.nepl, kind.nepl, binding.nepl, scope.nepl, stage0.nepl, and add a source-policy regression for the split.

## 検証

Run the name resolver split source-policy test, DefId absence regression, numeric-kind-tag regression, name resolver report contract, name_resolver source doctests, issue check, and git diff check.

## 修正内容

- `name_resolver.nepl` を doctest と `pub #import` だけの public facade にした。
- 実装を `name_resolver/id.nepl`、`kind.nepl`、`binding.nepl`、`scope.nepl`、`stage0.nepl` へ分割した。
- `SelfhostDefKind` equality は numeric tag helper や wildcard arm にせず、exhaustive match を維持した。
- `SelfhostNameBinding.def_id` は `Option<SelfhostDefId>` のままとし、pending binding が invalid id sentinel を持たない構造を維持した。
- `nodesrc/selfhost_name_resolver_sources.js` と `nodesrc/test_selfhost_name_resolver_split_contract.js` を追加し、split 後 source policy を固定した。

## 検証結果

- `node nodesrc/test_selfhost_name_resolver_split_contract.js`: pass
- `node nodesrc/test_selfhost_def_id_absence.js`: pass
- `node nodesrc/test_selfhost_name_resolver_report_contract.js`: pass
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/agent1-name-resolver-split-core.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
