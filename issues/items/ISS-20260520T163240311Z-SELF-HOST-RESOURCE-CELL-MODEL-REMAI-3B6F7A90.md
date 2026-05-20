---
id: ISS-20260520T163240311Z-SELF-HOST-RESOURCE-CELL-MODEL-REMAI-3B6F7A90
title: "self-host Resource cell model remains outside the resource/init tree"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/resource/move_state.nepl, stdlib/neplg2/core/resource/init/cell.nepl, nodesrc/test_selfhost_resource_tree_split_contract.js"
---

# ISS-20260520T163240311Z-SELF-HOST-RESOURCE-CELL-MODEL-REMAI-3B6F7A90: self-host Resource cell model remains outside the resource/init tree

## 概要

`doc/neplg2/self_host_source_tree_layout_review_20260518.md` fixes Resource IR initialized-cell work under `core/resource/init/`, but the self-host Resource cell state model still lived in `core/resource/move_state.nepl`. That legacy placement made the next initialized-cell implementation likely to grow a flat move-check-shaped file instead of the Resource IR source tree.

## 対象

- `stdlib/neplg2/core/resource/move_state.nepl`
- `stdlib/neplg2/core/resource/init/cell.nepl`
- `nodesrc/test_selfhost_resource_tree_split_contract.js`

## 根拠

- The layout review requires `resource/init/cell.nepl` for initialized-cell state and keeps `resource/*` root files out of implementation bodies.
- `move_state.nepl` held `SelfhostResourceCellState`, `SelfhostResourceCellEventKind`, and equality/stage helpers directly.
- `core/proof/fact/model.nepl` imported the legacy path, so proof payloads were coupled to the old flat Resource root placement.

## 問題

The self-host Resource cell model was already useful for real proof progress, but it was not in the final tree position. Leaving it there would make later raw cell lifecycle / initialized range work grow under a compatibility name and obscure the `resource/init` boundary.

## 影響

Future self-host Resource IR work could reintroduce a checker-local move-state module shape and make source-policy tests target legacy names rather than initialized-cell responsibilities.

## 修正方針

Move the Resource cell state/event model to `core/resource/init/cell.nepl`, keep `move_state.nepl` as an implementation-free compatibility facade, update proof payload imports to the final path, and add a source-policy test that blocks implementation from returning to the legacy facade or becoming a proof engine.

## 対応結果

- `core/resource/init/cell.nepl` を追加し、Resource cell state / event model を final tree path へ移した。
- `core/resource/move_state.nepl` は `./init/cell` の re-export だけを行う facade にした。
- `core/proof/fact/model.nepl`、`core/proof/evidence.nepl`、`core/proof/obligation.nepl`、`core/proof/refutation.nepl`、`core/proof/api/resource.nepl`、`core/proof/solver/resource.nepl` の Resource cell payload import を final path へ変更した。
- `nodesrc/test_selfhost_resource_tree_split_contract.js` を追加し、legacy facade への実装再導入、active proof core の legacy import、Resource cell model の文字列 authority / proof engine 化を拒否する。

## 完了条件

- Initialized-cell lifecycle の次作業は `core/resource/init/` 配下へ追加し、`move_state.nepl` へ実装を戻さない。
- Owner / borrow / lifetime も次に触るときは `resource/owner/*`、`resource/borrow/*`、`resource/init/*` などの最終階層へ分割してから実装する。
