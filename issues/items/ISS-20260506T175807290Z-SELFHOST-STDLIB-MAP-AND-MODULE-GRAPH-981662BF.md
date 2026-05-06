---
id: ISS-20260506T175807290Z-SELFHOST-STDLIB-MAP-AND-MODULE-GRAPH-981662BF
title: "selfhost stdlib_map and module graph doctests time out on current main"
area: selfhost
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/neplg2/core/module/graph.nepl, tests/stdlib/neplg2_stdlib_map.n.md"
---

# ISS-20260506T175807290Z-SELFHOST-STDLIB-MAP-AND-MODULE-GRAPH-981662BF: selfhost stdlib_map and module graph doctests time out on current main

## 概要

After syncing current main 5b989c56 and rebuilding the compiler bundle, node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-baseline-head.json -j 1 times out after 60000ms even with the import-spec range changes stashed. The broader focused run also times out in stdlib/neplg2/core/module/graph.nepl and stdlib_map.nepl doctests.

## 対象

- `stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/neplg2/core/module/graph.nepl, tests/stdlib/neplg2_stdlib_map.n.md`

## 根拠

- `git stash push -m temp-selfhost-import-spec-ranges-check` で今回の import-spec range 化差分を退避した状態でも、`node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-baseline-head.json -j 1` は `wasm test case timeout after 60000ms` になった。
- 差分を戻した後の broader focused run でも `stdlib/neplg2/core/module/graph.nepl::doctest#1` と `stdlib/neplg2/core/module/stdlib_map.nepl::doctest#1` が同じ timeout を報告した。
- 以前の `ISS-20260428T155038303Z-SELF-HOST-MODULE-LOADER-LACKS-STDLIB-B3D12B30` では同じ `stdlib_map.nepl` doctest が passing と記録されているため、current main で再発した検証不能状態として分離する。
- `origin/main` `824ada60` で fs/stdio scratch owner issue が修正された後も、`node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-after-fs-stdio-fix.json -j 1` と `node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl --no-tree -o tmp/selfhost-module-graph-after-fs-stdio-fix.json -j 1` はそれぞれ 60000ms timeout のまま。

## 問題

After syncing current main 5b989c56 and rebuilding the compiler bundle, node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-baseline-head.json -j 1 times out after 60000ms even with the import-spec range changes stashed. The broader focused run also times out in stdlib/neplg2/core/module/graph.nepl and stdlib_map.nepl doctests.

## 影響

Selfhost module path mapping and graph regressions can no longer be validated by local doctests. This blocks safe import graph work and can hide whether future selfhost changes broke path resolution or merely hit an unrelated runtime budget issue.

## 修正方針

Profile the stdlib_map and graph doctests on current ResourceIR/runtime, identify whether the timeout is compile-time or WASI runtime execution, and either fix the hot path or split the doctests into smaller focused cases without weakening import graph coverage.

## 検証

Run node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-after-fix.json -j 1 and the corresponding module graph focused tests without timeout.
