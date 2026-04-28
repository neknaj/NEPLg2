---
id: ISS-20260428T155038303Z-SELF-HOST-MODULE-LOADER-LACKS-STDLIB-B3D12B30
title: "self-host module loader lacks stdlib and user root path mapping"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/neplg2/core/module/graph.nepl, tests/stdlib/neplg2_stdlib_map.n.md"
---

# ISS-20260428T155038303Z-SELF-HOST-MODULE-LOADER-LACKS-STDLIB-B3D12B30: self-host module loader lacks stdlib and user root path mapping

## 概要

S2 の import graph は VFS logical path を辿れるが、`#import "core/result"` のような stdlib import と、`#import "./util"` のような user relative import を VFS logical path へ正規化する境界が無かった。

## 対象

- `stdlib/neplg2/core/module/stdlib_map.nepl, stdlib/neplg2/core/module/graph.nepl, tests/stdlib/neplg2_stdlib_map.n.md`

## 根拠

- `doc/neplg2/self_host_execution_plan.md` の S2 checkpoint は `selfhost/s2-stdlib-map` を stdlib root と user root の解決単位としている。
- 既存 `core/module/graph.nepl` は import spec の path をそのまま VFS path として扱っており、`core/result` を `stdlib/core/result.nepl` へ写す責務を持っていなかった。
- `doc/neplg2/self_host_plan.md` は core 層を filesystem 非依存にし、CLI が VFS を渡す設計を要求しているため、path mapping も filesystem path ではなく logical root 上で完結する必要がある。

## 問題

S2 の import graph は VFS logical path を辿れるが、`#import "std/..."` / `#import "core/..."` / user relative import / root file を module path に正規化する境界がない。resolver や CLI が import lexeme を場当たり的に連結すると、stdlib root と user root の対応が不統一になる。

## 影響

self-host compiler が stdlib と複数 input file を同じ graph に載せられず、S3 の name resolver が filesystem 依存の path 文字列へ漏れる。

## 修正方針

core/module に filesystem 非依存の stdlib/user root path mapper を追加し、import spec から VFS logical path を決める単一 API と diagnostic を用意する。

## 対応

- `stdlib/neplg2/core/module/stdlib_map.nepl` を追加し、`SelfhostModulePathMap`、`SelfhostModulePathKind`、`SelfhostResolvedModulePath` を定義した。
- 非 relative import は stdlib root 基準、`.` / `..` / `/` で始まる import は current module の root / directory 基準で解決し、拡張子なし path には `.nepl` を補うようにした。
- `/stdlib/...` のような absolute logical path も stdlib root 配下として分類し、resolver が user module と stdlib module を enum で区別できるようにした。
- `..` が user root / stdlib root を越える場合は `selfhost.module_path.escape_root` diagnostic にした。
- `core/module/graph.nepl` に `selfhost_build_module_graph_with_path_map` を追加し、既存の完全一致 graph API を残したまま stdlib/user root mapping 付き graph 構築を使えるようにした。
- `tests/stdlib/neplg2_stdlib_map.n.md` に stdlib import、relative import、mapped graph、root escape diagnostic の回帰を追加した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/neplg2-stdlib-map-doctest.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_stdlib_map.n.md --no-tree -o tmp/neplg2-stdlib-map-focused.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg2-module-graph-after-stdlib-map.json -j 1`: total=3 passed=3
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_stdlib_map.n.md -i tests/stdlib/neplg2_module_graph.n.md -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-stdlib-map-syntax.json -j 1`: total=53 passed=53
- remote main の `1166ee3 fix(core): support enum match wildcard arms` まで rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2/core/module/stdlib_map.nepl --no-tree -o tmp/neplg2-stdlib-map-doctest-after-rebase-1166ee3.json -j 1`: total=1 passed=1
- rebase 後、`node nodesrc/tests.js -i tests/stdlib/neplg2_stdlib_map.n.md --no-tree -o tmp/neplg2-stdlib-map-focused-after-rebase-1166ee3.json -j 1`: total=3 passed=3
- rebase 後、`node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/neplg2-module-graph-after-stdlib-map-rebase-1166ee3.json -j 1`: total=3 passed=3
