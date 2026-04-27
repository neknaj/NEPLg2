---
id: ISS-20260427T000311579Z-ADJACENCYMATRIX-RETAINS-UNSAFE-UNWRA-1D406374
title: "AdjacencyMatrix retains unsafe unwrap in owned bit storage cleanup"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/adjacency_matrix.nepl, tests/stdlib/adjacency_matrix_collections.n.md"
---

# ISS-20260427T000311579Z-ADJACENCYMATRIX-RETAINS-UNSAFE-UNWRA-1D406374: AdjacencyMatrix retains unsafe unwrap in owned bit storage cleanup

## 概要

AdjacencyMatrix.free still calls uwok on dealloc_ptr for its owned bit matrix storage.

## 対象

- `stdlib/alloc/collections/adjacency_matrix.nepl, tests/stdlib/adjacency_matrix_collections.n.md`

## 根拠

- 未記入

## 問題

AdjacencyMatrix.free still calls uwok on dealloc_ptr for its owned bit matrix storage.

## 影響

Graph utilities needed by self-host dependency analysis can trap during cleanup instead of following an explicit owner-invariant raw cleanup path.

## 修正方針

Use dealloc_raw for owned matrix storage, document why the raw path is valid, add a free regression, and guard implementation code against unsafe unwrap helpers.

## 検証

Run AdjacencyMatrix doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
