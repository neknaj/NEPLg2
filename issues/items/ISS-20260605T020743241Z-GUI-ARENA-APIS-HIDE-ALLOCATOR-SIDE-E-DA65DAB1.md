---
id: ISS-20260605T020743241Z-GUI-ARENA-APIS-HIDE-ALLOCATOR-SIDE-E-DA65DAB1
title: "GUI arena APIs hide allocator side effects behind pure signatures"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-05
updated: 2026-06-05
target: "stdlib/alloc/gui/tree/types.nepl, stdlib/alloc/gui/layout/arena.nepl, stdlib/alloc/gui/layout/stack.nepl"
---

# ISS-20260605T020743241Z-GUI-ARENA-APIS-HIDE-ALLOCATOR-SIDE-E-DA65DAB1: GUI arena APIs hide allocator side effects behind pure signatures

## 概要

Allocator-backed GUI arena constructors, add_child helpers, cleanup helpers, and layout connectors call Vec push/free but were exposed as pure fn.

## 対象

- `stdlib/alloc/gui/tree/types.nepl, stdlib/alloc/gui/layout/arena.nepl, stdlib/alloc/gui/layout/stack.nepl`

## 根拠

- Zenn 記事は NEPL の `fn` / `impure fn` による純粋性の静的検査を活用し、GUI/TUI の副作用は表層に整理する方針を求めている。
- `Vec::push` / `Vec::free` は owner-consuming allocation / cleanup を伴う `impure fn` であるため、これを呼ぶ arena helper を pure signature に置くと副作用境界が型に表れない。

## 問題

Allocator-backed GUI arena constructors, add_child helpers, cleanup helpers, and layout connectors call Vec push/free but were exposed as pure fn.

## 影響

The static effect checker cannot distinguish pure GUI traversal from owner-consuming allocation and cleanup boundaries, weakening Zenn-guided purity verification.

## 修正方針

Mark allocator-backed arena construction, mutation, cleanup, and dependent layout connectors as impure fn, while keeping borrowed query/focus/routing helpers pure.

## 検証

- `node nodesrc/tests.js -i tests/stdlib/gui_tree.n.md -i tests/stdlib/gui_layout.n.md -i tests/stdlib/gui_focus.n.md -i tests/stdlib/gui_routing.n.md -i tests/stdlib/gui_diff.n.md --no-tree -o tmp/gui-arena-effect.json -j 1 --dist web/dist --assert-io`

## 修正内容

- `ViewTreeArena` / `LayoutTreeArena` の constructor、add_child、free を `impure fn` に変更した。
- `layout_view_tree_arena_linear` / `layout_view_tree_arena_stack` と内部 loop を `impure fn` に変更した。
- arena を確保・解放するdoc test / markdown testだけを `impure fn` にし、bounded treeだけを扱う純粋テストは `fn` のまま維持した。
