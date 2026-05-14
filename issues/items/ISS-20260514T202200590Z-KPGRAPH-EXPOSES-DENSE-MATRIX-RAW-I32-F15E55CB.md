---
id: ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB
title: "kpgraph exposes dense matrix raw i32 storage handles"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/kp/kpgraph.nepl, nodesrc/test_stdlib_kpgraph_owner_boundary.js"
---

# ISS-20260514T202200590Z-KPGRAPH-EXPOSES-DENSE-MATRIX-RAW-I32-F15E55CB: kpgraph exposes dense matrix raw i32 storage handles

## 概要

kp/kpgraph stores DenseGraph.mat as a public i32 raw address, constructs it with alloc_raw, and exposes dense_graph_bfs_dist_raw(n, mat, start). The doctest also teaches callers to read Vec storage through mem_ptr_addr and load_i32. This erases the graph owner's free obligation and makes BFS consumers depend on raw storage identity instead of compiler-checkable typed owners.

## 対象

- `stdlib/kp/kpgraph.nepl, nodesrc/test_stdlib_kpgraph_owner_boundary.js`

## 根拠

- `DenseGraph` が `mat <i32>` を public field として持ち、raw address を graph owner として公開していた。
- `dense_graph_new` / `dense_graph_read_undirected_1indexed` が `alloc_raw` で matrix を確保し、`dense_graph_free` が raw address と `n*n` byte count を使って解放していた。
- `dense_graph_bfs_dist_raw(n, mat, start)` が raw matrix pointer を引数に取り、BFS の距離配列と queue も raw `i32` allocation と load/store で扱っていた。
- doctest が returned `Vec<i32>` の内部 storage を `mem_ptr_addr` / `load_i32` で読む例になっており、ordinary caller に raw storage identity 依存を教えていた。

## 問題

kp/kpgraph stores DenseGraph.mat as a public i32 raw address, constructs it with alloc_raw, and exposes dense_graph_bfs_dist_raw(n, mat, start). The doctest also teaches callers to read Vec storage through mem_ptr_addr and load_i32. This erases the graph owner's free obligation and makes BFS consumers depend on raw storage identity instead of compiler-checkable typed owners.

## 影響

Memory-safety checks cannot prove that the dense matrix allocation is uniquely owned or freed once, and ordinary KP code can keep/copy raw matrix addresses or read returned Vec storage through raw pointers. This contradicts the Stage 6 requirement that raw-memory-backed implementation details stay behind typed public owner APIs.

## 修正方針

Replace DenseGraph raw matrix storage with an AdjacencyMatrix owner wrapper, make construction/read/update/BFS return Result-based typed owners, delete dense_graph_bfs_dist_raw and raw mat field access from public examples, and add source-level regression coverage forbidding raw memory imports/helpers and raw i32 graph signatures in kpgraph.

## 検証

Run the kpgraph source policy test, focused doctest for stdlib/kp/kpgraph.nepl, issue metadata check, and git diff whitespace check.

## 修正結果

- `DenseGraph` を `matrix <AdjacencyMatrix>` の typed owner wrapper に変更した。
- `dense_graph_new` は `Result<DenseGraph, Diag>` を返し、`dense_graph_free` は `AdjacencyMatrix` owner を閉じる。
- `dense_graph_add_undirected` は `DenseGraph` owner を消費して `Result<DenseGraph, DenseGraphUpdateError>` を返す。失敗時は owner を `DenseGraphUpdateError` に戻す。
- `dense_graph_read_undirected_1indexed` は borrowed `&StreamScanner` を読み、構築・更新失敗を `Result<DenseGraph, Diag>` として返す。
- `dense_graph_bfs_dist_raw` を削除し、`dense_graph_bfs_dist(&DenseGraph, i32) -> Result<Vec<i32>, Diag>` に変更した。距離配列と queue は `Vec<i32>` で確保し、`Vec` API で読み書きする。
- doctest は raw memory import と raw Vec storage read をやめ、`v::get<i32>` で結果を表示する。

## 回帰テスト

- `nodesrc/test_stdlib_kpgraph_owner_boundary.js` を追加した。
- このテストは `kpgraph` の raw memory import/helper、`mat <i32>` field、`dense_graph_bfs_dist_raw`、raw `i32` graph signature の再導入を拒否し、typed owner API になっていることを検査する。

## 検証結果

- `node nodesrc/test_stdlib_kpgraph_owner_boundary.js`
- `node nodesrc/tests.js -i stdlib/kp/kpgraph.nepl --no-tree -o tmp/agent1-kpgraph-owner-boundary-module.json -j 1 --dist web/dist --assert-io`
