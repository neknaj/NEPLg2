---
id: ISS-20260428T235630333Z-SELF-HOST-CLI-LACKS-DIAGNOSTIC-REPOR-DF540D85
title: "self-host CLI lacks diagnostic reporter boundary"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/reporter.nepl, stdlib/neplg2/cli/main.nepl, tests/stdlib/selfhost_cli_reporter.n.md"
---

# ISS-20260428T235630333Z-SELF-HOST-CLI-LACKS-DIAGNOSTIC-REPOR-DF540D85: self-host CLI lacks diagnostic reporter boundary

## 概要

doc/neplg2/self_host_plan.md S6 requires cli/reporter.nepl to separate stderr human diagnostics from JSON output, but stdlib/neplg2/cli/ has only args and main. Future driver work would have to duplicate diagnostic formatting or mix artifact stdout with human diagnostics.

## 対象

- `stdlib/neplg2/cli/reporter.nepl, stdlib/neplg2/cli/main.nepl, tests/stdlib/selfhost_cli_reporter.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S6 は `cli/reporter.nepl` が stderr human diagnostic と JSON output を分けると定義している。
- `stdlib/neplg2/cli/` には `args.nepl` / `args/types.nepl` / `main.nepl` だけがあり、diagnostic rendering の責務境界が存在しない。
- `ISS-20260426T010003Z-STDIO-RESULT-STDERR-E48B51D0` で Result 付き stdout/stderr API は整備済みだが、self-host CLI はまだそれを使う reporter 層を持たない。

## 問題

doc/neplg2/self_host_plan.md S6 requires cli/reporter.nepl to separate stderr human diagnostics from JSON output, but stdlib/neplg2/cli/ has only args and main. Future driver work would have to duplicate diagnostic formatting or mix artifact stdout with human diagnostics.

## 影響

Self-host CLI parity cannot compare diagnostic JSON, stderr, and exit code independently. Compiler core diagnostics also risk depending on stdio if reporting is added ad hoc.

## 修正方針

Add cli/reporter.nepl as the single CLI-layer diagnostic rendering boundary. Keep core diagnostics pure, render human text for stderr and compact JSON for machine output, and expose Result-returning write helpers that use existing stdout/stderr stdio interfaces.

## 検証

Add focused reporter doctests and/or fixture tests for human rendering, JSON escaping, stderr separation, and Result propagation.
