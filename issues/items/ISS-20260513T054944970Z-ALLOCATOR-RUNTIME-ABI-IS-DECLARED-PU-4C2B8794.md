---
id: ISS-20260513T054944970Z-ALLOCATOR-RUNTIME-ABI-IS-DECLARED-PU-4C2B8794
title: "Allocator runtime ABI is declared pure while returning internal raw identities"
area: core
status: open
resolved: false
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: stdlib/core/mem/allocator.nepl
---

# ISS-20260513T054944970Z-ALLOCATOR-RUNTIME-ABI-IS-DECLARED-PU-4C2B8794: Allocator runtime ABI is declared pure while returning internal raw identities

## 概要

raw identity escape を raw-memory-boundary で抑制しない設計にした結果、alloc_raw / realloc_raw / __nepl_rt_alloc / __nepl_rt_realloc が pure 関数として内部 allocator raw address を返しているため、stdlib 自身のコンパイル時に resource.raw.identity_escape が発火する。これは診断を黙らせるのではなく、allocator API と effect 設計を現在の静的検査に合わせる必要がある。

## 対象

- `stdlib/core/mem/allocator.nepl`

## 根拠

- 未記入

## 問題

raw identity escape を raw-memory-boundary で抑制しない設計にした結果、alloc_raw / realloc_raw / __nepl_rt_alloc / __nepl_rt_realloc が pure 関数として内部 allocator raw address を返しているため、stdlib 自身のコンパイル時に resource.raw.identity_escape が発火する。これは診断を黙らせるのではなく、allocator API と effect 設計を現在の静的検査に合わせる必要がある。

## 影響

tests/compiler/move_effect.n.md の 40 件が allocator.nepl 側の resource.raw.identity_escape で先に停止し、個別 doctest の期待診断に到達できない。allocator state mutation と raw identity return の責務が pure signature と矛盾している。

## 修正方針

allocator runtime ABI と公開 raw allocator API の effect を再評価し、allocator state を変更する関数を impure として表現するか、compiler-internal allocation と公開 raw allocator を分離した effect 設計へ更新する。raw identity を file/path suppress で許可する設計には戻さない。

## 検証

trunk build 後に node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree を再実行し、allocator.nepl 起点の resource.raw.identity_escape が消え、各 doctest が本来の期待診断へ到達することを確認する。
