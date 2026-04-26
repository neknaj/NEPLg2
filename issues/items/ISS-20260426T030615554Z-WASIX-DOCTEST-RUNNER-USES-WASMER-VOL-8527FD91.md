---
id: ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91
title: "wasix doctest runner uses wasmer --volume unsupported by Wasmer 1.x"
area: cli
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nodesrc/run_test.js
---

# ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91: wasix doctest runner uses wasmer --volume unsupported by Wasmer 1.x

## 概要

#target wasix の doctest 実行時、nodesrc/run_test.js が wasmer run --volume=<host>:<guest> を固定で渡す。ローカルの wasmer 1.0.0 では --volume が未対応で、features_tui の compile 修正後に run phase が wasmer argument error で失敗する。

## 対象

- `nodesrc/run_test.js`

## 根拠

- `wasmer --version` はローカルで `wasmer 1.0.0` を返す。
- `wasmer run --help` には `--dir <DIR>` と `--mapdir <GUEST_DIR:HOST_DIR>` があり、`--volume` は存在しない。
- `ISS-20260426T023638576Z-FEATURES-TUI-DOCTEST-HELPER-988788D1` の修正後、`node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1` は compile phase を通過し、run phase で `error: Found argument '--volume' which wasn't expected` に変化した。

## 問題

#target wasix の doctest 実行時、nodesrc/run_test.js が wasmer run --volume=<host>:<guest> を固定で渡す。ローカルの wasmer 1.0.0 では --volume が未対応で、features_tui の compile 修正後に run phase が wasmer argument error で失敗する。

## 影響

wasix doctest が compiler/runtime 本体ではなく test runner の wasmer CLI 差分で失敗し、TUI や WASIX stdlib の回帰確認ができない。

## 修正方針

wasmer run --help または version を検出し、--volume をサポートしない Wasmer 1.x では --mapdir <guest:host> または --dir を使う互換 layer を nodesrc/run_test.js に実装する。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1`
