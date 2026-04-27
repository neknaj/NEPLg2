---
id: ISS-20260427T000313426Z-LIST-REVERSE-HIDES-ALLOCATION-FAILUR-E4B68FAA
title: "List reverse hides allocation failure behind unsafe unwraps"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md, nodesrc/test_stdlib_list_no_unsafe_unwraps.js"
---

# ISS-20260427T000313426Z-LIST-REVERSE-HIDES-ALLOCATION-FAILUR-E4B68FAA: List reverse hides allocation failure behind unsafe unwraps

## 概要

List.reverse allocates a new list with unwrap_ok/uwok even though cons/new can fail, so a normal allocation-bearing operation has no Result surface.

## 対象

- `stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md`

## 根拠

- `reverse` は `Result<List<.T>, Diag>` を返す API だが、内部では `new` と `cons` を `unwrap_ok` / `uwok` で取り出していた。
- `new` は現状では空 list の構築だけだが、`cons` はノード確保を行うため `Err(Diag)` になり得る。
- 途中まで reverse list を作った後に次の `cons` が失敗すると、従来実装では `uwok` で trap し、呼び出し側へ `Err` を返せず、部分 list の cleanup もできなかった。

## 問題

List.reverse allocates a new list with unwrap_ok/uwok even though cons/new can fail, so a normal allocation-bearing operation has no Result surface.

## 影響

Self-host linked-list utilities can trap on allocation pressure and the API shape hides a real failure mode from callers.

## 修正方針

Add a Result-returning reverse variant or change reverse to return Result if compatible, update callers/tests, and add a regression that prevents unsafe helpers in List implementation allocation paths.

## 解決内容

- `cons` と `reverse` が共有する `list_alloc_node` を追加し、ノード layout と allocation failure 診断を 1 箇所に集約した。
- `reverse` は `List` owner を `cons` に移動せず、内部では `new_head` raw pointer を更新し、成功時だけ `List new_head` を `Ok` で返すようにした。
- `list_alloc_node` が失敗した場合は、それまでに構築した部分 reverse list を `free` してから `Err(Diag)` を返すようにした。
- `nodesrc/test_stdlib_list_no_unsafe_unwraps.js` を追加し、List 実装に unsafe unwrap / checked deallocation helper が戻らないことと、reverse の cleanup 分岐を source policy として固定した。

## 検証

- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-tree -o tmp/list-reverse-docs.json -j 1`: 15/15 passed
- `node nodesrc/tests.js -i tests/stdlib/list_collections.n.md -i stdlib/tests/list.n.md --no-tree -o tmp/list-reverse-focused.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-list-reverse.json -j 4`: 300/300 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-list-reverse.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
