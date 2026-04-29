---
id: ISS-20260429T021254285Z-RESOURCE-OWNER-GATE-LEAKS-OBLIGATION-8F3BD354
title: "Resource owner gate leaks obligations in self-host hash and diagnostic rendering"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, stdlib/alloc/hash/hash32.nepl, stdlib/neplg2/cli/reporter.nepl"
---

# ISS-20260429T021254285Z-RESOURCE-OWNER-GATE-LEAKS-OBLIGATION-8F3BD354: Resource owner gate leaks obligations in self-host hash and diagnostic rendering

## 概要

Latest stdlib/neplg2 focused run fails 17 doctests with D3100 owner obligation leaks in hash32(str) and selfhost_cli_render_diagnostic_json. This blocks broad self-host regression even though targeted CLI file_io tests pass.

## 対象

- `nepl-core/src/resource, stdlib/alloc/hash/hash32.nepl, stdlib/neplg2/cli/reporter.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-cli-file-io-neplg2-final2.json -j 2` は 36 件中 19 passed / 17 failed。
- top issue は `hash32__str__i32__pure` の `h1` と `selfhost_cli_render_diagnostic_json__SelfhostDiagnostic__str__pure` の `sb` に対する `D3100 resource ir owner obligation leak`。
- 同じ checkout で `neplg2/cli/file_io.nepl` と `tests/stdlib/selfhost_cli_file_io.n.md` の focused test は通過しているため、filesystem 境界実装ではなく Resource IR owner flow の残件として切り分ける。

## 問題

Latest stdlib/neplg2 focused run fails 17 doctests with D3100 owner obligation leaks in hash32(str) and selfhost_cli_render_diagnostic_json. This blocks broad self-host regression even though targeted CLI file_io tests pass.

## 影響

Self-host CLI modules that import hash or reporter cannot be validated as a whole; future stdlib/selfhost refactors may be hidden behind the same Resource IR owner false positive or real obligation leak.

## 修正方針

Trace Resource IR owner state for pure string hash locals and StringBuilder return/ownership flow without weakening D3100. Add focused regression fixtures for hash32(str) and reporter JSON rendering before closing.

## 検証

trunk build; node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/selfhost-resource-owner-obligation.json -j 2; focused hash/reporter tests pass without suppressing D3100 diagnostics.
