---
id: ISS-20260517T212438890Z-SELFHOST-STDLIB-MAP-GRAPH-DOCTEST-EX-DACB7C6E
title: "selfhost stdlib_map graph doctest exceeds default compile timeout"
area: CORE
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-17
updated: 2026-05-18
target: "tests/stdlib/neplg2_stdlib_map.n.md, stdlib/neplg2/core/module/graph.nepl, stdlib/neplg2/core/module/loader.nepl, stdlib/neplg2/core/module/vfs.nepl, stdlib/neplg2/core/module/import_scan.nepl"
---

# ISS-20260517T212438890Z-SELFHOST-STDLIB-MAP-GRAPH-DOCTEST-EX-DACB7C6E: selfhost stdlib_map graph doctest exceeds default compile timeout

## 概要

tests/stdlib/neplg2_stdlib_map.n.md::doctest#2 passes when NEPL_TEST_CASE_TIMEOUT_MS is raised to 300000, but under the default 60000ms wasm case budget it times out in compile phase. The focused long run measured compile_ms about 74772 and run_ms about 29, so the bottleneck is compiler/static-check cost for the selfhost graph/std-test shape, not generated wasm execution.

## 対象

- `tests/stdlib/neplg2_stdlib_map.n.md, stdlib/neplg2/core/module/graph.nepl, stdlib/neplg2/core/module/loader.nepl, stdlib/neplg2/core/module/vfs.nepl, stdlib/neplg2/core/module/import_scan.nepl`

## 根拠

- `node nodesrc/tests.js -i tests\stdlib\neplg2_stdlib_map.n.md --no-tree -o tmp\agent1-neplg2-stdlib-map-owner-summary.json -j 1 --dist web\dist --assert-io` は total=3, passed=2, errored=1。`doctest#2` だけが compile phase で `wasm test case timeout after 60000ms` になった。
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='300000'; node nodesrc\run_doctest.js -i tests\stdlib\neplg2_stdlib_map.n.md -n 2 --assert-io --dist web\dist` は pass し、`compile_ms=74772`, `run_ms=29`, `total_ms=74850` だった。
- `doctest#1` と `doctest#3` は同じ dist で compile 約22秒、run 約20ms以下で pass しているため、今回の ResourceIR owner diagnostic 解消とは別に、graph/VFS/std-test fixture の compile-time cost が default timeout を超えている。

## 問題

tests/stdlib/neplg2_stdlib_map.n.md::doctest#2 passes when NEPL_TEST_CASE_TIMEOUT_MS is raised to 300000, but under the default 60000ms wasm case budget it times out in compile phase. The focused long run measured compile_ms about 74772 and run_ms about 29, so the bottleneck is compiler/static-check cost for the selfhost graph/std-test shape, not generated wasm execution.

## 影響

The stdlib_map graph regression cannot be used as a normal local/CI focused gate under the default timeout, and static-check complexity regressions can be mistaken for runtime behavior unless this path is optimized or split with a principled test strategy.

## 修正方針

Investigate compile-stage timing for the graph/VFS/std-test fixture and reduce ResourceIR/type/effect summary work from the compiler side. Do not solve by only raising timeout. If the test is inherently too broad, split it into smaller doctests while preserving stdout assertions.

## 検証

Run tests/stdlib/neplg2_stdlib_map.n.md::doctest#2 under the default 60000ms timeout with --assert-io and confirm it passes; keep doctest#1/#3 and ResourceIR owner summary regressions passing.

## 2026-05-18 修正

根本原因は generated wasm の実行時間ではなく、selfhost module graph の通常経路が `loader -> module_parser` を通じて parser / lexer / AST 全体を `stdlib_map` graph doctest に引き込み、静的検査対象の関数 summary 数を不必要に増やしていたことだった。timeout を延ばすのではなく、graph 構築が本当に必要とする source lookup と top-level import extraction を loader/parser から分離した。

- `SelfhostVirtualFile` / `SelfhostVirtualFileSystem` と VFS helper を `neplg2/core/module/vfs.nepl` へ分離し、`loader.nepl` は VFS を re-export しつつ parse 済み module loader の責務に絞った。
- `neplg2/core/module/import_scan.nepl` を追加し、module graph 用に行頭 0 column の top-level `#import` directive だけを source text から走査する lightweight path を用意した。directive の構文と path / alias 抽出は既存の `import_spec` parser を使うため、独自の緩い構文解釈にはしていない。
- `graph.nepl` の通常 DFS 経路は full module AST を作らず `SelfhostImportRecord` を edge に変換する。既存の AST helper は残るが、default graph build は parser 全体へ依存しない。
- 初期案として lexer/token stream を使う scanner も確認したが、selfhost lexer/token の関数群を graph doctest に引き込んで static check time が悪化したため破棄した。採用した line scanner は import graph の責務に必要な top-level import だけを扱う。

検証結果:

- default timeout のまま `node nodesrc\run_doctest.js -i tests\stdlib\neplg2_stdlib_map.n.md -n 2 --assert-io --dist web\dist` が pass。`compile_ms=48873`, `run_ms=23`, `total_ms=48948`。
- native timing では同じ doctest source の `resource_static_check` が約 30.5s から `20922ms` に下がった。owner summary count は 423 から 288 に下がり、不要に広い selfhost compiler surface を静的検査に渡していたことが確認できた。
- `stdlib/neplg2/core/module/vfs.nepl` doctest、`stdlib/neplg2/core/module/import_scan.nepl` doctest、`stdlib/neplg2/core/module/loader.nepl` doctest、`tests/stdlib/neplg2_stdlib_map.n.md::doctest#1/#3` も pass。

この修正は静的検査を弱めていない。検査器側の timeout や allowlist ではなく、selfhost module graph の責務境界を狭め、source から必要な import fact だけを得る設計へ戻した。
