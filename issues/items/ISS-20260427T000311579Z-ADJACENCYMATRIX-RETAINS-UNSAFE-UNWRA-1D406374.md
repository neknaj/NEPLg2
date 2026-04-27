---
id: ISS-20260427T000311579Z-ADJACENCYMATRIX-RETAINS-UNSAFE-UNWRA-1D406374
title: "AdjacencyMatrix retains unsafe unwrap in owned bit storage cleanup"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/adjacency_matrix.nepl, tests/stdlib/adjacency_matrix_collections.n.md, nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js"
---

# ISS-20260427T000311579Z-ADJACENCYMATRIX-RETAINS-UNSAFE-UNWRA-1D406374: AdjacencyMatrix retains unsafe unwrap in owned bit storage cleanup

## 概要

AdjacencyMatrix.free still calls uwok on dealloc_ptr for its owned bit matrix storage.

## 対象

- `stdlib/alloc/collections/adjacency_matrix.nepl, tests/stdlib/adjacency_matrix_collections.n.md, nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js`

## 根拠

- `AdjacencyMatrix.new` は `nverts > 0` のときだけ `nverts * nverts` bit を格納する `nbytes > 0` の byte 配列を確保し、成功時の `bits` pointer を `AdjacencyMatrix` owner に格納する。
- `AdjacencyMatrix.free` はその owned `bits` を `dealloc_ptr<u8>` に渡し、`Result` を `uwok` していた。
- 通常 cleanup の前提は owner invariant で保証されるため、checked deallocation の Err arm を unsafe helper で握りつぶす必要はない。

## 問題

AdjacencyMatrix.free still calls uwok on dealloc_ptr for its owned bit matrix storage.

## 影響

Graph utilities needed by self-host dependency analysis can trap during cleanup instead of following an explicit owner-invariant raw cleanup path.

## 修正方針

Use dealloc_raw for owned matrix storage, document why the raw path is valid, add a free regression, and guard implementation code against unsafe unwrap helpers.

## 解決内容

- `AdjacencyMatrix.free` を `dealloc_ptr + uwok` から `dealloc_raw mem_ptr_addr bits nbytes` に変更した。
- `free` の doc comment に、`new` が確保した `nbytes > 0` の matrix byte 配列を `AdjacencyMatrix` が所有していること、`free` 後の再利用は禁止であることを明記した。
- `tests/stdlib/adjacency_matrix_collections.n.md` に `adjacency_matrix_free_releases_owned_storage` を追加し、free 後に再確保できることと再確保した owner も free できることを確認した。
- `nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js` を追加し、AdjacencyMatrix 実装に unsafe unwrap helper / unreachable が戻らないことと、`free` が raw owner cleanup を使うことを CI source policy に登録した。

## 検証

- `node nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js`: pass
- source policy regressions: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl --no-tree -o tmp/adjacency-matrix-owned-cleanup-docs.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i tests/stdlib/adjacency_matrix_collections.n.md --no-tree -o tmp/adjacency-matrix-owned-cleanup-focused.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-adjacency-matrix-owned-cleanup.json -j 4`: 288/288 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-adjacency-matrix-owned-cleanup.json -j 4`: 418/418 passed
