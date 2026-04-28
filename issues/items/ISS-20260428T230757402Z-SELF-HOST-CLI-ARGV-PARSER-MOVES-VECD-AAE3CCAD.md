---
id: ISS-20260428T230757402Z-SELF-HOST-CLI-ARGV-PARSER-MOVES-VECD-AAE3CCAD
title: "self-host CLI argv parser moves VecDataLen span fields under RawMemoryLoadCell gate"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/args.nepl, tests/stdlib/selfhost_cliarg_parser.n.md, nodesrc/test_selfhost_cli_args_no_owner_field_reads.js"
---

# ISS-20260428T230757402Z-SELF-HOST-CLI-ARGV-PARSER-MOVES-VECD-AAE3CCAD: self-host CLI argv parser moves VecDataLen span fields under RawMemoryLoadCell gate

## 概要

selfhost_cli_parse_args and selfhost_cli_parse_argv obtain a VecDataLen<str> span and then read data and len with owner-consuming get. RawMemoryLoadCell reports the second field read as a moved span under the stricter gate.

## 対象

- `stdlib/neplg2/cli/args.nepl, tests/stdlib/selfhost_cliarg_parser.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-mono-instance-key-neplg2.json -j 1` で `total=32, passed=19, failed=13` になった。
- `stdlib\neplg2\cli\args.nepl::doctest#3/#4` は `selfhost_cli_parse_args` の `/stdlib/neplg2/cli/args.nepl:746` `let data <i32> mem_ptr_addr get span "data"` が D3100 になり、`RawMemoryLoadCell ... Local("span") ... found Moved` と報告された。
- `stdlib\neplg2\cli\args.nepl::doctest#5` は `selfhost_cli_parse_argv` の `/stdlib/neplg2/cli/args.nepl:796` で同じ `span` moved D3100 になった。
- `VecDataLen<str>` の `data` と `len` は Copy field だが、owned `get span "data"` / `get span "len"` が span owner を消費する形になっている。

## 問題

selfhost_cli_parse_args and selfhost_cli_parse_argv obtain a VecDataLen<str> span and then read data and len with owner-consuming get. RawMemoryLoadCell reports the second field read as a moved span under the stricter gate.

## 影響

stdlib/neplg2 focused tests fail before exercising later self-host stages, and CLI argv parser regressions are hidden behind raw memory ownership diagnostics.

## 修正方針

Read VecDataLen data and len through field get_ref, keeping the span owner available while copying its Copy fields. Add a source policy or focused regression so direct get span data/len is not reintroduced.

## 修正内容

- `selfhost_cli_parse_args` と `selfhost_cli_parse_argv` で、`VecDataLen<str>` の `data` / `len` を `get span "..."` ではなく `*get_ref &span "..."` で読むようにした。
- `nodesrc/test_selfhost_cli_args_no_owner_field_reads.js` を追加し、direct `get span "data|len"` の再導入を禁止した。
- `doc/testing.md` の source policy regression 一覧へ新しい確認コマンドを追加した。

## 検証

- `trunk build`: pass（`origin/main` の `eebb659 fix(core): separate region token raw cell state` へ rebase 後）
- `node nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`: pass
- `node nodesrc/tests.js -i stdlib\neplg2\cli\args.nepl --no-tree -o tmp\selfhost-cli-vecdatalen-ref-args-after-rebase.json -j 1`: total=5, passed=5
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cliarg_parser.n.md --no-tree -o tmp\selfhost-cli-vecdatalen-ref-fixture-after-rebase.json -j 1`: total=10, passed=10
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-cli-vecdatalen-ref-neplg2-after-rebase.json -j 1`: total=32, passed=22, failed=10。`selfhost_cli_parse_args` / `selfhost_cli_parse_argv` の `span` D3100 は解消し、残件は既知の Vec element provenance と SelfhostOutcome raw cell 系。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
