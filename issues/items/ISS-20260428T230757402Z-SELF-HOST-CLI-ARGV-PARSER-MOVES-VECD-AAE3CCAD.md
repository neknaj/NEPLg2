---
id: ISS-20260428T230757402Z-SELF-HOST-CLI-ARGV-PARSER-MOVES-VECD-AAE3CCAD
title: "self-host CLI argv parser moves VecDataLen span fields under RawMemoryLoadCell gate"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/args.nepl, tests/stdlib/selfhost_cliarg_parser.n.md"
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

## 検証

Run selfhost CLI args focused doctests, tests/stdlib/selfhost_cliarg_parser.n.md, stdlib/neplg2 focused run, node nodesrc/issues.js check, and git diff --check.
