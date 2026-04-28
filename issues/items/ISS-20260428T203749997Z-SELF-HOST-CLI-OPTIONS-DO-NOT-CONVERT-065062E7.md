---
id: ISS-20260428T203749997Z-SELF-HOST-CLI-OPTIONS-DO-NOT-CONVERT-065062E7
title: "self-host CLI options do not convert to core compile options"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/args.nepl, stdlib/neplg2/core/options.nepl"
---

# ISS-20260428T203749997Z-SELF-HOST-CLI-OPTIONS-DO-NOT-CONVERT-065062E7: self-host CLI options do not convert to core compile options

## 概要

The self-host CLI parser returns SelfhostCliOptions and core/options.nepl now owns SelfhostCompileOptions, but there is no bridge from parsed CLI target/profile/verbose values into the pure core option record. Driver or pipeline work would need to read CLI internals directly.

## 対象

- `stdlib/neplg2/cli/args.nepl, stdlib/neplg2/core/options.nepl`

## 根拠

- `SelfhostCliOptions` は `target <Option<SelfhostCliTarget>>`、`profile <Option<SelfhostCliProfile>>`、`verbose <bool>` を持つ。
- `SelfhostCompileOptions` は core pipeline に渡す target/profile/verbose の pure option record として追加済み。
- 修正前は CLI parser の結果を core option record へ変換する API がなかった。

## 問題

The self-host CLI parser returns SelfhostCliOptions and core/options.nepl now owns SelfhostCompileOptions, but there is no bridge from parsed CLI target/profile/verbose values into the pure core option record. Driver or pipeline work would need to read CLI internals directly.

## 影響

The core/CLI separation remains incomplete: a future driver cannot hand options to core without ad hoc field reads, and target/profile parity rules can drift between CLI and core.

## 修正方針

CLI 側に `selfhost_cli_target_to_compile_target` と `selfhost_cli_profile_to_build_profile` を追加し、CLI enum から core enum へ明示的に変換できるようにしました。

さらに `selfhost_cli_options_to_compile_options` を追加し、`SelfhostCliOptions` の target/profile/verbose だけを `SelfhostCompileOptions` に移します。emit、input、output、run などは CLI/driver 層の責務として残すため、core option は CLI の実行制御や artifact writer に依存しません。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\cli\args.nepl --no-tree -o tmp\selfhost-cli-core-options.json -j 1`: total=5 passed=5
