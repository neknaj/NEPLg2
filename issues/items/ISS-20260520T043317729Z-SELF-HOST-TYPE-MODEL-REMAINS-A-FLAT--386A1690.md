---
id: ISS-20260520T043317729Z-SELF-HOST-TYPE-MODEL-REMAINS-A-FLAT--386A1690
title: "self-host type model remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/ty/ty.nepl; stdlib/neplg2/core/ty/ty/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T043317729Z-SELF-HOST-TYPE-MODEL-REMAINS-A-FLAT--386A1690: self-host type model remains a flat implementation file

## 概要

Selfhost type id, kind, record payload, arena owner operation, structural equality, and stage smoke are implemented in one flat ty.nepl file. This repeats the Rust types.rs flat-file risk and makes the next checker, abstraction, and proof work harder to audit.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl; stdlib/neplg2/core/ty/ty/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は Rust 側 `types.rs` の flat 構造を NEPL 側へ移植しない方針を明記している。
- `stdlib/neplg2/core/ty/ty.nepl` は分割前に 769 行あり、TypeId、TypeKind、record payload、arena owner operation、structural equality、stage0 smoke を同じ file に持っていた。
- S3 type checker、abstraction、trait coherence、Resource proof へ進む前に、型表現の所有境界と equality 境界を file 単位で分ける必要があった。

## 問題

Selfhost type id, kind, record payload, arena owner operation, structural equality, and stage smoke are implemented in one flat ty.nepl file. This repeats the Rust types.rs flat-file risk and makes the next checker, abstraction, and proof work harder to audit.

## 影響

Future type/effect/resource/static-check work will keep adding unrelated logic to ty.nepl, weakening file-level ownership and making enum/match coverage policies harder to enforce.

## 修正方針

Keep ty.nepl as a documentation/public facade, move implementation into ty/id.nepl, ty/kind.nepl or kind/*, ty/record.nepl, ty/arena.nepl, ty/eq.nepl, and ty/stage0.nepl, then add a source-policy regression for the split.

## 検証

Run the type split source-policy test, numeric-kind-tag regression, type-record payload regression, type-arena doctest contract, ty.nepl doctest, and type arena integration doctests.

## 修正内容

- `ty.nepl` を doctest と `pub #import` だけの public facade にした。
- 実装を `ty/id.nepl`、`ty/kind/{model,eq,name}.nepl`、`ty/record.nepl`、`ty/arena.nepl`、`ty/eq.nepl`、`ty/stage0.nepl` へ分割した。
- `SelfhostTypeKind` の equality は `kind/eq.nepl` の exhaustive match に残し、numeric tag helper や wildcard arm に置き換えていない。
- `SelfhostTypeRecord` の payload と accessor は分割後も enum payload を match してから読む構造を維持した。
- `nodesrc/selfhost_ty_sources.js` と `nodesrc/test_selfhost_ty_split_contract.js` を追加し、facade への実装再導入、split file の 450 行超過、submodule から ty facade への曖昧 import を監視する。

## 検証結果

- `node nodesrc/test_selfhost_ty_split_contract.js`: pass
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`: pass
- `node nodesrc/test_selfhost_type_record_payload.js`: pass
- `node nodesrc/test_selfhost_type_arena_report_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/agent1-ty-split-core.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/agent1-ty-split-arena.json -j 1 --dist web/dist --assert-io`: total=5, passed=5
