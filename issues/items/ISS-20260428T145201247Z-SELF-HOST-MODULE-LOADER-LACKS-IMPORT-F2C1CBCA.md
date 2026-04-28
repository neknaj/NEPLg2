---
id: ISS-20260428T145201247Z-SELF-HOST-MODULE-LOADER-LACKS-IMPORT-F2C1CBCA
title: "self-host module loader lacks import graph and cycle diagnostics"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/module/graph.nepl, stdlib/neplg2/core/module/loader.nepl, tests/stdlib/neplg2_module_graph.n.md"
---

# ISS-20260428T145201247Z-SELF-HOST-MODULE-LOADER-LACKS-IMPORT-F2C1CBCA: self-host module loader lacks import graph and cycle diagnostics

## 概要

S2 の VFS loader と typed import spec は揃ったが、root module から import closure を構築する graph API が無い。後続 resolver が各 module を個別に load すると、missing import、重複 load、cycle detection の責務が分散する。

## 対象

- `stdlib/neplg2/core/module/graph.nepl, stdlib/neplg2/core/module/loader.nepl, tests/stdlib/neplg2_module_graph.n.md`

## 根拠

- `doc/neplg2/self_host_execution_plan.md` の S2 は `selfhost/s2-module-graph` を graph、cycle detection、missing module diagnostic の commit 単位として定義している。
- `core/module/loader.nepl` は VFS から単一 module を parse でき、`core/module/import_spec.nepl` は AST から typed import spec を抽出できるが、root module から import closure を構築する API は存在しなかった。
- `doc/neplg2/self_host_plan.md` は core 層を filesystem 非依存にし、CLI が VFS を渡す設計を要求しているため、graph 構築も VFS logical path 上で完結する必要がある。

## 問題

S2 の VFS loader と typed import spec は揃ったが、root module から import closure を構築する graph API が無い。後続 resolver が各 module を個別に load すると、missing import、重複 load、cycle detection の責務が分散する。

## 影響

self-host compiler が複数 file / stdlib import を一貫して扱えず、resolver や pipeline が parser/loader 内部へ直接依存する。cycle を診断できないまま再帰 load すると、後続 S2/S3 の実装が不安定になる。

## 修正方針

core/module/graph.nepl を追加し、VFS と typed import spec から root module の import closure を構築する。visited と active stack を分離し、missing module と cycle を SelfhostDiagnostic として返す。graph は core 層の純粋 data とし、filesystem へ依存しない。

## 検証

tests/stdlib/neplg2_module_graph.n.md に root からの transitive imports、missing import diagnostic、cycle diagnostic の回帰を追加する。

## 対応

- `stdlib/neplg2/core/module/graph.nepl` を追加し、`SelfhostModuleGraphNode` / `SelfhostModuleGraphEdge` / `SelfhostModuleGraph` を定義した。
- graph node は `Visiting` / `Done` の enum 状態を持ち、DFS 中に active module を再訪した場合は `selfhost.module_graph.cycle` を返すようにした。
- root path から VFS logical path を辿り、typed import spec から edge を追加する `selfhost_build_module_graph` を追加した。
- missing import は import directive の span を primary label にした `selfhost.module_graph.missing_module` diagnostic とし、missing path を note に入れるようにした。
- module AST は import spec 抽出後に解放し、graph は path / file_id / edge span の軽量データだけを所有する設計にした。

## 検証結果

- `node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl --no-tree -o tmp/neplg2-module-graph-doctest.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg2-module-graph-focused.json -j 1`: total=3 passed=3
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-module-graph-syntax.json -j 1`: total=49 passed=49
- remote main の `fc47035 refactor(core): reuse resource function aliases` まで rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-module-graph-syntax-after-rebase.json -j 1`: total=49 passed=49
- remote main の `017979d refactor(core): split resource check reports` まで再 rebase 後、`trunk build`: pass
- 再 rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-module-graph-syntax-after-rebase-017979d.json -j 1`: total=49 passed=49
- remote main の `1a8c8de refactor(core): split resource return summaries` まで再 rebase 後、`trunk build`: pass
- 再 rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-module-graph-syntax-after-rebase-1a8c8de.json -j 1`: total=49 passed=49
- remote main の `4bc7220 refactor(core): split resource shadow entry` まで再 rebase 後、`trunk build`: pass
- 再 rebase 後、`node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl --no-tree -o tmp/neplg2-module-graph-doctest-after-rebase-4bc7220.json -j 1`: total=1 passed=1
- 再 rebase 後、`node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg2-module-graph-focused-after-rebase-4bc7220.json -j 1`: total=3 passed=3
- `node nodesrc/issues.js check`: pass
- `git diff --check`: whitespace warning only for generated issue index / markdown line endings, no diff whitespace error
