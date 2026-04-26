---
id: ISS-20260426T010006Z-CLIARG-TESTED-ARGS-A8D0E229
title: "self-host CLI needs verified argv and option parsing surface"
area: selfhost
status: open
resolved: false
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
