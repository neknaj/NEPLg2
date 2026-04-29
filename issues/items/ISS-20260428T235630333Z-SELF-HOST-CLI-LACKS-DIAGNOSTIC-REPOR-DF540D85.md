---
id: ISS-20260428T235630333Z-SELF-HOST-CLI-LACKS-DIAGNOSTIC-REPOR-DF540D85
title: "self-host CLI lacks diagnostic reporter boundary"
area: selfhost
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-29
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

## 対応結果

`stdlib/neplg2/cli/reporter.nepl` を追加し、`SelfhostDiagnostic` を human stderr text と compact JSON object へ変換する API を実装した。primary label は `file_id:start..end` の byte span と message を出し、note は human / JSON の両方へ反映する。

単一 diagnostic 用の `selfhost_cli_write_human_diagnostic_stderr` / `selfhost_cli_write_json_diagnostic_stdout` と、collection 用の `selfhost_cli_write_human_diagnostics_stderr` / `selfhost_cli_write_json_diagnostics_stdout` を追加した。書き出しは `stdio_write_stderr_str_result` / `stdio_write_str_result` に限定し、core diagnostic 側には stdio import を入れていない。

`tests/stdlib/selfhost_cli_reporter.n.md` を追加し、単一 diagnostic の render 結果、JSON escaping、stdout/stderr 分離、collection の human / JSON rendering を固定した。rebase 後の remote main で `Vec<T>.get_ref` raw memory provenance が修正されたため、collection fixture もこの issue 内で green にした。

## 検証

Add focused reporter doctests and/or fixture tests for human rendering, JSON escaping, stderr separation, and Result propagation.

- `node nodesrc/test_selfhost_cli_reporter_boundary.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib\neplg2\cli\reporter.nepl --no-tree -o tmp\selfhost-cli-reporter-rebased-module-collection.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\selfhost-cli-reporter-rebased-fixture-collection.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i stdlib\neplg2\cli\args.nepl -i stdlib\neplg2\cli\args\types.nepl -i stdlib\neplg2\cli\reporter.nepl -i tests\stdlib\selfhost_cliarg_parser.n.md -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\selfhost-cli-reporter-rebased-cli-focused.json -j 2`: total=19 passed=19
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-cli-reporter-rebased-neplg2.json -j 2`: total=34 passed=34
