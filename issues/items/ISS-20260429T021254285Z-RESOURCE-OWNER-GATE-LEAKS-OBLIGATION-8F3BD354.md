---
id: ISS-20260429T021254285Z-RESOURCE-OWNER-GATE-LEAKS-OBLIGATION-8F3BD354
title: "Resource owner gate leaks obligations in self-host hash and diagnostic rendering"
area: core
status: fixed
resolved: true
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

## 対応

- Resource owner summary が `MaybeFreed` return state を definite fresh owner として caller へ伝播しないようにした。これにより、recursive pure value helper の初回 fixed-point で未確定になった copy-like return が、存在しない free obligation として `hash32(str)` や lexer helper に漏れる問題を止めた。
- User-call summary に consumed parameter set を追加し、`sb_build` のような wrapper が owner 引数を戻り値へ返さず内部で消費する場合に caller 側の owner obligation を解放できるようにした。
- direct raw memory effect の `Call` では owner summary を適用せず、直後に lower される `ResourceOp::RawMemory` を authoritative な消費点として扱うようにした。これにより `dealloc_raw` / `realloc_raw` で call summary と raw memory op が同じ owner を二重に消費する問題を避けた。
- Regression として recursive copy summary、normal wrapper consume、direct raw memory consume を `nepl-core/tests/resource_ir.rs` に追加した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check -- --nocapture`: 21 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-owner-summary-rebased-move-effect.json -j 1`: total=110 passed=110
- `node nodesrc\tests.js -i stdlib\neplg2\cli\reporter.nepl --no-tree -o tmp\agent1-owner-summary-rebased-reporter.json -j 1`: total=1 passed=1
- `node nodesrc\tests.js -i stdlib\neplg2 --no-tree -o tmp\agent1-owner-summary-rebased-neplg2.json -j 2`: total=36 passed=36
