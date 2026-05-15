---
id: ISS-20260515T004634650Z-KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E-1F2C94F7
title: "kpgraph unsafe-unwrap policy still expects removed raw BFS API"
area: test
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js, stdlib/kp/kpgraph.nepl, nodesrc/test_stdlib_kpgraph_owner_boundary.js"
---

# ISS-20260515T004634650Z-KPGRAPH-UNSAFE-UNWRAP-POLICY-STILL-E-1F2C94F7: kpgraph unsafe-unwrap policy still expects removed raw BFS API

## 概要

nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js still requires legacy kp_i32_empty_vec, kp_push_i32, and dense_graph_bfs_dist_raw even though Stage 6 replaced kpgraph with a DenseGraph AdjacencyMatrix owner wrapper and typed Result<Vec<i32>, Diag> BFS API.

## 対象

- `nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js, stdlib/kp/kpgraph.nepl, nodesrc/test_stdlib_kpgraph_owner_boundary.js`

## 根拠

- `ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB` で `DenseGraph` は `AdjacencyMatrix` owner wrapper へ移行済みである。
- 現行 `stdlib/kp/kpgraph.nepl` は `dense_graph_bfs_dist_raw` を持たず、`dense_graph_bfs_dist(&DenseGraph, i32) -> Result<Vec<i32>, Diag>` で BFS 結果 owner を返す。
- `nodesrc/test_stdlib_kpgraph_owner_boundary.js` はこの owner boundary を固定しているが、`nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js` は古い `kp_i32_empty_vec` / `kp_push_i32` / `dense_graph_bfs_dist_raw` を要求し続けていた。

## 問題

nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js still requires legacy kp_i32_empty_vec, kp_push_i32, and dense_graph_bfs_dist_raw even though Stage 6 replaced kpgraph with a DenseGraph AdjacencyMatrix owner wrapper and typed Result<Vec<i32>, Diag> BFS API.

## 影響

The source-policy suite reports a kpgraph warning after the safe owner-boundary migration, and the stale check could encourage reintroducing raw BFS helpers instead of preserving the compiler-checkable typed owner API.

## 修正方針

Update the source policy to reject the legacy raw helper API and verify the current DenseGraph owner API handles allocation and storage errors through Result, Vec API access, and explicit owner cleanup.

## 検証

Run node nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js, node nodesrc/test_stdlib_kpgraph_owner_boundary.js, node nodesrc/run_source_policy_regressions.js --warn-only, node nodesrc/issues.js check --dir issues, and git diff --check.

## 2026-05-15 Agent 1 解決

`nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js` を現行 Stage 6 の typed owner API に合わせた。古い raw BFS fallback helper が存在することを要求する検査を削除し、逆に `KpI32PushRes` / `kp_i32_empty_vec` / `kp_push_i32` / `dense_graph_bfs_dist_raw` が戻らないことを固定した。

新しい検査では、`dense_graph_bfs_dist` が `(&DenseGraph, i32) -> Result<Vec<i32>, Diag>` を公開し、距離配列と queue を `v::filled<i32>` で確保し、失敗時には `Diag` を返し、queue allocation 失敗では `dist` owner を解放することを確認する。BFS 中の読み書きは `v::get` / `v::replace` 経由であり、raw storage helper や growable push sentinel を要求しない。

検証:

- `node nodesrc/test_stdlib_kpgraph_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_kpgraph_owner_boundary.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
  - kpgraph source policy warning は解消した。
  - 残警告は stdlib documentation contract。既存 issue `ISS-20260514T154316014Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-5F916C0F` の範囲で扱う。
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
