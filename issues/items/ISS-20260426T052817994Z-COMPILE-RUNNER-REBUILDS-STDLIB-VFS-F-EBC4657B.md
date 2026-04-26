---
id: ISS-20260426T052817994Z-COMPILE-RUNNER-REBUILDS-STDLIB-VFS-F-EBC4657B
title: "compile runner rebuilds stdlib VFS for every test case"
area: nodesrc
status: open
resolved: false
priority: P2
type: performance
created: 2026-04-26
updated: 2026-04-26
target: "nodesrc/run_test.js, nodesrc/cli.js"
---

# ISS-20260426T052817994Z-COMPILE-RUNNER-REBUILDS-STDLIB-VFS-F-EBC4657B: compile runner rebuilds stdlib VFS for every test case

## 概要

`nodesrc/run_test.js` と `nodesrc/cli.js` は compile invocation ごとに `stdlib/**/*.nepl` を走査して読み込み、同じ stdlib VFS を毎回作り直している。
`node nodesrc/tests.js -i stdlib --no-tree -j 4` のような compile-heavy doctest run では、404 件の doctest が worker process 内で同じ stdlib VFS を繰り返し再構築する。

## 対象

- `nodesrc/run_test.js, nodesrc/cli.js`

## 根拠

- `nodesrc/run_test.js:356` の `loadStdlibVfsFromFs` は `walkFiles(root)` と `fs.readFileSync` で stdlib 全体を object に詰める。
- `nodesrc/run_test.js:371` の `compileWithFsStdlib` は各 `runSingle` ごとに `loadStdlibVfsFromFs()` を呼ぶ。
- `nodesrc/cli.js:359` / `nodesrc/cli.js:381` も同じ構造で、同一 process 内の複数 compile でも cache を持たない。
- この workspace で stdlib VFS 構築を 20 回測ると平均約 50ms、対象ファイルは 88 件、約 1.1MiB だった。
- `tmp/stdlib-stdio-result-stderr-import-merge.json` では stdlib doctest 404 件の合計 duration が約 1767.5 秒、平均約 4375ms であり、VFS 再構築は compiler hot path 測定に混ざる固定コストになっている。

## 問題

compile 処理そのものの前に、同一内容の stdlib VFS を毎回 filesystem から再構築している。
これは compiler phase の実行時間ではないが、compile-heavy test の wall time を増やし、monomorphize / typecheck など本来の compile bottleneck の測定を曇らせる。

## 影響

繰り返しの stdlib filesystem scan が doctest 件数に比例して増え、compile-heavy run の wall time に避けられる固定コストを足している。
現在の stdlib scan はこの workspace で 1 回あたり約 50ms なので、404 件では約 20 秒規模の無駄になり得る。
セルフホスト作業で stdlib doctest や focused compile suite が増えるほど、compiler 本体の改善効果を見積もりにくくなる。

## 修正方針

stdlib root ごとに process-local cache を持ち、同一 command invocation 内では cached VFS を返す。
cache は process 内だけに閉じ、新しい `node` / CLI invocation では必ず filesystem を読み直すため、開発中のファイル変更は次の command で観測できる。
compile API 呼び出し側が VFS を mutate しない前提を守り、念のため caller 側で stdlib VFS と local VFS を merge する fallback では cached object を直接破壊しない。

## 検証

- `nodesrc/run_test.js` / `nodesrc/cli.js` の stdlib VFS cache が同一 root で同じ object を返し、別 root では別 object を返すことを確認する。
- `node nodesrc/tests.js -i tests/stdlib/stdio_result_stderr.n.md --no-tree -o tmp/compile-vfs-cache-focused.json -j 1` を通す。
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/streamio.nepl -i stdlib/std/io.nepl --no-tree -o tmp/compile-vfs-cache-stdio-suite.json -j 1` を通す。
- 必要に応じて VFS 構築の focused timing を修正前後で比較する。
