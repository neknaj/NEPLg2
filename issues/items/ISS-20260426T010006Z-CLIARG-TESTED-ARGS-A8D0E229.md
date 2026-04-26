---
id: ISS-20260426T010006Z-CLIARG-TESTED-ARGS-A8D0E229
title: "self-host CLI needs verified argv and option parsing surface"
area: selfhost
status: verified
resolved: true
priority: P2
type: test
created: 2026-04-26
updated: 2026-04-26
target: stdlib/std/env/cliarg.nepl
source: doc/neplg2/self_host_plan.md
---

# ISS-20260426T010006Z-CLIARG-TESTED-ARGS-A8D0E229: self-host CLI needs verified argv and option parsing surface

## 概要

self-host CLI は `--target`、`--emit`、`--stdlib-root`、`-o`、input path を安定して解釈する必要があるが、現行 argv API とその回帰テストは self-host CLI の要求を満たすほど固定されていない。

## 対象

- `stdlib/std/env/cliarg.nepl`
- `stdlib/neplg2/cli/args.nepl`
- `tests/stdlib/*cliarg*`

## 根拠

- `cliarg_count` / `cliarg_get` は存在する。
- 旧 review には fs/cliarg の主要テストが skip されている問題が記録されている。
- self-host CLI は複数 option、値必須 option、unknown option、複数 input の診断を持つ必要がある。

## 問題

argv 取得が target ごとに不安定なまま CLI parser を積むと、compiler core の問題と CLI option layer の問題を切り分けられない。

## 影響

Pass A / Pass B の比較コマンドや CI job が、argv layer の未検証挙動で失敗する可能性がある。
ユーザー向け CLI の usage error と compile error の exit code も混ざる。

## 修正方針

`stdlib/neplg2/cli/args.nepl` に pure な argv parser を置き、`std/env/cliarg` は raw argv provider として扱う。
Node / Rust test harness から argv を注入できる fixture を追加し、unknown option、missing value、multiple input、output path をテーブル駆動で検証する。

## 検証

- `cliarg_count` / `cliarg_get` の WASI integration test を skip 解除する。
- pure args parser の doctest を `ReadStream::Text` なしで実行できる形で追加する。
- self-host CLI smoke test で `--check`、`--emit wasm`、`-o out.wasm` を確認する。

## 解決内容

- `stdlib/neplg2/cli/args.nepl` に `SelfhostCliOptions`、`SelfhostCliTarget`、`SelfhostCliEmit`、`SelfhostCliProfile`、`SelfhostCliErrorKind` を追加し、CLI option surface を enum / struct で表現した。
- `selfhost_cli_parse_args` は argv[0] を含まない pure parser、`selfhost_cli_parse_argv` は raw argv から argv[0] を飛ばす入口として分離した。
- `--target` / `--emit` / `--profile` / `--stdlib-root` / `-o` / `--output` / `-i` / `--input` / positional input / `--` run args boundary を解析するようにした。
- unknown option、missing value、multiple input、output path、argv[0] skip、run args boundary の回帰テストを `tests/stdlib/selfhost_cliarg_parser.n.md` に追加した。
- `stdlib/tests/cliarg.n.md` に argv injection の値読み取りと out-of-range rejection を追加し、`std/env/cliarg` を raw argv provider として固定した。
- 実装中に aggregate の複数 field 読みが by-value `get` で `D3053` になることを確認し、`ISS-20260426T142242010Z-BORROWED-FIELD-PROJECTION-API-MISSIN-3010781E` として分離した。

## 検証結果

- `node nodesrc/tests.js -i stdlib/neplg2/cli/args.nepl -i tests/stdlib/selfhost_cliarg_parser.n.md --no-tree -o tmp/selfhost-cliarg-parser-after-borrow-sync.json -j 1`: 10/10 passed
- `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-tree -o tmp/cliarg-provider-after-borrow-sync.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/selfhost-neplg2-after-cliargs.json -j 2`: 22/22 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-after-cliargs.json -j 4`: 411/411 passed
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-cliargs.json`: 13/13 passed
- `cargo fmt --all --check`: pass
- `cargo run -p nepl-cli -- --check -i examples/counter.nepl --target std`: pass
- `cargo run -p nepl-cli -- -i examples/counter.nepl --target std --emit wasm -o tmp/selfhost-cliargs-smoke-out.wasm`: pass, wasm output created
