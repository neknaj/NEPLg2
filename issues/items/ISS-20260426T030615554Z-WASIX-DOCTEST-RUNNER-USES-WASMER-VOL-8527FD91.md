---
id: ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91
title: "wasix doctest runner uses wasmer --volume unsupported by Wasmer 1.x"
area: cli
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nodesrc/run_test.js, nodesrc/tui_regression.js, nodesrc/wasmer_args.js"
---

# ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91: wasix doctest runner uses wasmer --volume unsupported by Wasmer 1.x

## 概要

#target wasix の doctest 実行時、nodesrc/run_test.js が wasmer run --volume=<host>:<guest> を固定で渡す。ローカルの wasmer 1.0.0 では --volume が未対応で、features_tui の compile 修正後に run phase が wasmer argument error で失敗する。

## 対象

- `nodesrc/run_test.js, nodesrc/tui_regression.js, nodesrc/wasmer_args.js`

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

## 対応結果

`nodesrc/wasmer_args.js` を追加し、`wasmer run --help` から mount option を検出して `--volume` / `--mapdir` / `--dir` を選択するようにした。
Wasmer 1.x の `--mapdir` は可変個数 option のため `--mapdir=/::C:/...` のように `=` 付きで渡し、Windows drive colon と guest/host 区切りが衝突しないよう `::` を使う。

`nodesrc/run_test.js` と `nodesrc/tui_regression.js` は共通 helper を使い、host temp dir を guest `/` として preopen する。
これによりローカルの Wasmer 1.0.0 でも `--volume` unsupported では落ちず、WASIX smoke が実行できる。

`features_tui` は次に `wasix_32v1.tty_get` unknown import へ進んだため、別 Issue `ISS-20260426T031747615Z-WASIX-DOCTEST-RUNNER-LACKS-HOST-SUPP-EC9A9094` として分離した。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1`
- `node --check nodesrc/run_test.js; node --check nodesrc/tui_regression.js; node --check nodesrc/wasmer_args.js`
- `node -e "const { wasmerRunMountArgs } = require('./nodesrc/wasmer_args'); console.log(JSON.stringify(wasmerRunMountArgs(process.env.WASMER_BIN || 'wasmer', 'C:/tmp/host', '/')));"` は `["--mapdir=/::C:/tmp/host"]` を返す。
- WASIX smoke を `nodesrc/run_test.js` へ直接渡した実行が `ok=true` で完了。
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1` は `--volume` error を解消し、`wasix_32v1.tty_get` unknown import で失敗。host import fallback は別 Issue で追跡。
