---
id: ISS-20260506T175807290Z-SELFHOST-STDLIB-MAP-AND-MODULE-GRAPH-981662BF
title: "selfhost stdlib_map and module graph doctests time out on current main"
area: selfhost
status: fixed
resolved: true
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
- `selfhost_module_path_last_slash_loop` の recursive scan を while scan に変更すると `stdlib_map.nepl` は timeout ではなく `resolved.path` 未検証の owner leak 診断まで進んだ。doctest を resolved kind だけでなく resolved path まで検証する形に直すと `stdlib/neplg2/core/module/stdlib_map.nepl` focused doctest は passing になった。
- 一方で `stdlib/neplg2/core/module/graph.nepl` は継続して timeout する。切り分けでは `stdlib/neplg2/core/module/loader.nepl`、`stdlib/neplg2/core/syntax/parser/module_parser.nepl`、空入力の `lex_all_with_file_id` smoke も default 60000ms timeout になり、graph 残件は `stdlib_map` ではなく lexer/parser/loader 側が根である。
- 残る graph blocker は `ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D` として分離した。

## 問題

After syncing current main 5b989c56 and rebuilding the compiler bundle, node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-baseline-head.json -j 1 times out after 60000ms even with the import-spec range changes stashed. The broader focused run also times out in stdlib/neplg2/core/module/graph.nepl and stdlib_map.nepl doctests.

## 影響

Selfhost module path mapping and graph regressions can no longer be validated by local doctests. This blocks safe import graph work and can hide whether future selfhost changes broke path resolution or merely hit an unrelated runtime budget issue.

## 修正方針

`stdlib_map` 側は recursive scan と不十分な doctest 検証を修正済み。残る graph timeout は lexer/parser/loader の `lex_next` / `lex_all` 経路を `ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D` で解消してから、graph DFS 自体の focused doctest を再確認する。

## 検証

`node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/selfhost-stdlib-map-after-trunk.json -j 1` は passing。`node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl --no-tree -o tmp/selfhost-module-graph-after-trunk.json -j 1` は `ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D` の lexer timeout により継続 blocked。

## 2026-05-07 closeout

`ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D` の closeout 後、current `web/dist` で stdlib_map と module graph の focused doctest を再確認した。

結果として、stdlib_map と graph はどちらも default 60000ms budget で passed になった。stdlib_map 側の recursive scan / doctest verification 問題と、graph 側の lexer timeout blocker は解消済みのため、この issue は fixed とする。

検証:

- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree --dist web/dist -o tmp/selfhost_stdlib_map_closeout.json -j 1 --assert-io`: 1/1 passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl --no-tree --dist web/dist -o tmp/selfhost_graph_timeout_closeout.json -j 1 --assert-io`: 1/1 passed
