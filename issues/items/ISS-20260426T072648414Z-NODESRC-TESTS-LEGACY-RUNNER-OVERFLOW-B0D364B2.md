---
id: ISS-20260426T072648414Z-NODESRC-TESTS-LEGACY-RUNNER-OVERFLOW-B0D364B2
title: "nodesrc tests legacy runner overflows stack on std/io facade"
area: nodesrc
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nodesrc/tests.js, nodesrc/run_test.js, tests/stdlib/io.n.md"
---

# ISS-20260426T072648414Z-NODESRC-TESTS-LEGACY-RUNNER-OVERFLOW-B0D364B2: nodesrc tests legacy runner overflows stack on std/io facade

## 概要

`origin/main` `96a4d19` 取り込み後、`node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -j 1` が `tests/stdlib/io.n.md::doctest#1` の compile 中に `RangeError: Maximum call stack size exceeded` で失敗する。
同じ suite は `-j 2` の worker 実行では通るため、single-job legacy runner と worker runner の実行条件が一致していない。

## 対象

- `nodesrc/tests.js, nodesrc/run_test.js, tests/stdlib/io.n.md`

## 根拠

- `stdlib/mem-bulk-copy` branch で `origin/main` を同期した後、`tests/stdlib/io.n.md` は `-j 1` で `total=6`, `passed=5`, `failed=1` になった。
- 同じ作業 tree で `node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/mem-bulk-copy-io-j2.json -j 2` は `total=6`, `passed=6`, `failed=0`。
- detached `origin/main` worktree でも、同じ compiler dist を使った `-j 1` 実行で同じ失敗を再現したため、bulk copy 実装とは独立している。

## 問題

`nodesrc/tests.js` の single-job path が legacy runner を使うことで、worker path と異なる JS stack 条件または compile invocation 条件になっている。
そのため、同じ NEPL source と同じ compiler dist でも `-j 1` だけが stack overflow し、検証結果が jobs 数に依存する。

## 影響

single-job focused verification が `std/io` facade の false failure を報告し、実際の回帰と runner 由来の失敗を区別しづらくなる。
issue ごとの検証で `-j 1` と `-j 2` の結果が食い違うため、commit 前確認の再現性も落ちる。

## 修正方針

single-job path も worker-based execution model へ揃えるか、legacy runner 側の stack 条件を worker と同等にする。
あわせて、`tests/stdlib/io.n.md::doctest#1` を single-job entry で実行して worker mode と同じ結果になる回帰テストを追加する。

## 対応

- `nodesrc/tests.js` の wasm runner が `jobs <= 1` または `cases.length <= 1` のときに in-process legacy runner へ戻る分岐をなくした。
- default の `NEPL_WASM_THREAD_POOL=1` では、`-j 1` でも worker thread based runner を使い、compiler wasm の JS stack 条件を `-j 2` 以上と揃えるようにした。
- `nodesrc/test_tests_runner_jobs.js` を追加し、`tests/stdlib/io.n.md` が `-j 1` と `-j 2` の両方で 6/6 pass することを固定した。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/io-legacy-runner-fixed-j1.json -j 1`: `total=6`, `passed=6`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/io-legacy-runner-fixed-j2-alone.json -j 2`: `total=6`, `passed=6`, `failed=0`
- `node nodesrc/test_tests_runner_jobs.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/mem_bulk_copy.n.md --no-tree -o tmp/nodesrc-runner-mem-bulk-copy-j1.json -j 1`: `total=6`, `passed=6`, `failed=0`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/nodesrc-runner-postfix-stdlib-full.json -j 4`: `total=404`, `passed=404`, `failed=0`
- `node nodesrc/test_cli_args.js`: pass
- `node nodesrc/issues.js check`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-nodesrc-runner.json`: `13/13 passed`
