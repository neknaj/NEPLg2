---
id: ISS-20260426T104115640Z-FS-DOCTEST-FIXTURE-IS-LINE-ENDING-SE-910C88C4
title: "fs doctest fixture is line-ending sensitive"
area: stdlib
status: verified
resolved: true
priority: P2
type: test
created: 2026-04-26
updated: 2026-04-26
target: stdlib/std/fs.nepl
---

# ISS-20260426T104115640Z-FS-DOCTEST-FIXTURE-IS-LINE-ENDING-SE-910C88C4: fs doctest fixture is line-ending sensitive

## 概要

RV-STDLIB-006 の fs_read_to_bytes / fs_read_to_string doctest が tests/fixtures/fs/read_sample.txt の改行を LF 固定で比較しており、core.autocrlf=true の Windows checkout では CRLF になって失敗する。

## 対象

- `stdlib/std/fs.nepl`

## 根拠

- `core.autocrlf=true` の Windows checkout では `tests/fixtures/fs/read_sample.txt` が `CRLF` になり、doctest の期待値 `"fs fixture text\n"` と一致しなかった。
- `fs_open_read` と `fs_read_fd_bytes` の doctest は成功しており、失敗は読み込み実装ではなく fixture text 比較の改行依存に限定された。

## 問題

RV-STDLIB-006 の fs_read_to_bytes / fs_read_to_string doctest が tests/fixtures/fs/read_sample.txt の改行を LF 固定で比較しており、core.autocrlf=true の Windows checkout では CRLF になって失敗する。

## 影響

Windows agent で verified issue が再び赤くなり、fs runtime 境界の回帰テストが環境依存になる。

## 修正方針

fixture 内容の比較を LF と CRLF のどちらでも通るようにし、checkout の改行変換に依存しない doctest にする。

## 検証

node nodesrc/tests.js -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --no-tree -o tmp/main-fs-cliarg-doctests.json -j 1

## 対応

- `fs_read_to_bytes` / `fs_read_to_string` の doctest で fixture 内容を LF と CRLF のどちらでも受け入れるようにした。
- fixture の checkout 改行変換に依存しない形で、fs read 成功経路の検証を維持した。

## 検証結果

- `node nodesrc/tests.js -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --no-tree -o tmp/main-fs-cliarg-doctests.json -j 1`: total=10, passed=10
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/fs-fixture-crlf-stdlib-full.json -j 4`: total=404, passed=404
- `cargo fmt --all --check`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-fs-fixture-crlf.json`: 13/13 passed
- `node nodesrc/issues.js index` / `node nodesrc/issues.js check`: pass
