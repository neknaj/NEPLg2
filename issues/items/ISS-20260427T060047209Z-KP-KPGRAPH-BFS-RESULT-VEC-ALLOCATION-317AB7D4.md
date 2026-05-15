---
id: ISS-20260427T060047209Z-KP-KPGRAPH-BFS-RESULT-VEC-ALLOCATION-317AB7D4
title: "kp/kpgraph BFS result Vec allocation failure が unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/kp/kpgraph.nepl, nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js"
---

# ISS-20260427T060047209Z-KP-KPGRAPH-BFS-RESULT-VEC-ALLOCATION-317AB7D4: kp/kpgraph BFS result Vec allocation failure が unwrap_ok で trap する

## 概要

dense_graph_bfs_dist_raw creates and fills its Vec<i32> result with unwrap_ok new/push, so allocation or grow failure traps after BFS instead of returning a safe empty result.

## 対象

- `stdlib/kp/kpgraph.nepl, nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js`

## 根拠

- `dense_graph_bfs_dist_raw` は BFS 結果 `out` を `unwrap_ok new<i32>` で生成していた。
- 同関数は `dist` raw array から `Vec<i32>` へ詰め直す各 push も `unwrap_ok push<i32>` で行っていた。

## 問題

dense_graph_bfs_dist_raw creates and fills its Vec<i32> result with unwrap_ok new/push, so allocation or grow failure traps after BFS instead of returning a safe empty result.

## 影響

kpgraph is a stdlib graph helper and its doctest/stdin path can abort on memory pressure, keeping RV-STDLIB-010 unsafe helper debt in normal implementation code.

## 修正方針

Qualify Vec operations through a vec alias, replace unwrap_ok with explicit Result matches, stop result accumulation on push failure, and return an empty Vec sentinel on allocation failure.

## 検証

- `node nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/kp/kpgraph.nepl --no-tree -o tmp/kpgraph-bfs-allocation-docs.json -j 1`: 1/1 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-kpgraph-bfs-allocation.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass (CRLF conversion warning only)

## 解決内容

- `KpI32PushRes` / `kp_i32_empty_vec` / `kp_push_i32` を追加し、BFS 結果 `Vec<i32>` の push failure を `ok=false` と空 Vec sentinel に変換した。
- `dense_graph_bfs_dist_raw` の `new<i32>` / `push<i32>` から implementation `unwrap_ok` を除去した。
- result accumulation は `failed=true` で停止し、allocation failure 時は consumed owner を再利用しない空 Vec sentinel を返すようにした。
- `kpgraph` 内部の Vec 操作を `v::new` / `v::push` / `v::Vec` に限定し、caller/import 側の overload と混ざらないようにした。
- `nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js` を追加し、CI/source policy と `doc/testing.md` に登録した。

## 2026-05-15 Agent 1 Stage 6 後続整理

`ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB` により、`kpgraph` は旧 `dense_graph_bfs_dist_raw` / `kp_push_i32` 経路を削除し、`DenseGraph` owner と `dense_graph_bfs_dist(&DenseGraph, i32) -> Result<Vec<i32>, Diag>` へ移行済みである。

そのため、この issue で追加した `nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js` の役割も、旧 raw BFS helper の存在確認ではなく、unsafe unwrap を戻さず、typed owner API が allocation / storage invariant failure を `Result` と owner cleanup で扱うことの監視へ更新した。詳細は `ISS-20260515T004634650Z-KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E-1F2C94F7` に分離した。
