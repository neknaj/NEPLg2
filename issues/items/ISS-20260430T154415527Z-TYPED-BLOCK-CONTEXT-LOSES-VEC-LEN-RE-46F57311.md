---
id: ISS-20260430T154415527Z-TYPED-BLOCK-CONTEXT-LOSES-VEC-LEN-RE-46F57311
title: "Typed block context loses Vec len_ref overload after Result unwrap"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/typecheck, tests/compiler/overload.n.md"
---

# ISS-20260430T154415527Z-TYPED-BLOCK-CONTEXT-LOSES-VEC-LEN-RE-46F57311: Typed block context loses Vec len_ref overload after Result unwrap

## 概要

tests/compiler/overload.n.md::doctest#19 fails with type.overload.no_match at len_ref<i32> &v after Vec::new |> unwrap_ok in a typed block context, while the adjacent Stack path resolves.

## 対象

- `nepl-core/src/typecheck, tests/compiler/overload.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-owner-pipeline-agent1.json -j 1 --dist web/dist` で `doctest#19` が `type.overload.no_match` を返した。
- 失敗箇所は `let vn <i32> len_ref<i32> &v;` と、その結果を使う `and eq sn 0 eq vn 0` 周辺で、同じ fixture の直前にある `Stack<i32>` の `len_ref<i32> &st` は解決できている。

## 問題

tests/compiler/overload.n.md::doctest#19 fails with type.overload.no_match at len_ref<i32> &v after Vec::new |> unwrap_ok in a typed block context, while the adjacent Stack path resolves.

## 影響

Examples that rely on typed block context and explicit generic collection observers cannot compile reliably. This should be fixed in overload/type context propagation rather than by relaxing diagnostics.

## 修正方針

Audit typed block expected-type propagation through pipeline Result unwrap and overload candidate filtering for borrowed Vec observers. Preserve the explicit Vec<T> context through unwrap_ok and &v so len_ref<i32> resolves exhaustively.

## 検証

node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-typed-block-vec-len-ref.json -j 1 --dist web/dist
