---
id: ISS-20260430T154415527Z-TYPED-BLOCK-CONTEXT-LOSES-VEC-LEN-RE-46F57311
title: "overload fixture still calls obsolete Vec len_ref API"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-05-01
target: "tests/compiler/overload.n.md, stdlib/alloc/collections/vec.nepl"
---

# ISS-20260430T154415527Z-TYPED-BLOCK-CONTEXT-LOSES-VEC-LEN-RE-46F57311: overload fixture still calls obsolete Vec len_ref API

## 概要

tests/compiler/overload.n.md::doctest#19 fails with `type.overload.no_match` because the fixture calls `len_ref<i32> &v` for `Vec<i32>`, while current `Vec` exposes the borrowed length observer as `len<i32> &v`.

## 対象

- `tests/compiler/overload.n.md, stdlib/alloc/collections/vec.nepl`

## 根拠

- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 19 --dist web/dist` で `doctest#19` が `type.overload.no_match` を返した。
- `stdlib/alloc/collections/stack.nepl` は `len_ref <.T> <(&Stack<.T>)->i32>` を持つ。
- `stdlib/alloc/collections/vec.nepl` は `len <.T> <(&Vec<.T>)->i32>` を持ち、`len_ref` は持たない。

## 問題

The fixture was using an obsolete `Vec` observer name. Treating this as a typechecker overload failure would hide a correct `type.overload.no_match` diagnostic.

## 影響

The overload fixture incorrectly reported a compiler failure even though the current stdlib API requires `len<i32> &v`.

## 修正方針

Update the fixture to call `len<i32> &v` for `Vec<i32>` and keep `len_ref<i32> &st` for `Stack<i32>`. Do not add a backward-compatible `Vec::len_ref` alias.

## 解決

- `tests/compiler/overload.n.md::doctest#19` の `Vec` observer を `len<i32> &v` に更新した。
- stdlib 側に obsolete alias は追加せず、現行 API と overload fixture の整合性を取った。

## 検証

- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 19 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-typed-block-vec-len-ref-agent1.json -j 1 --dist web/dist`: total=45, passed=45, failed=0
