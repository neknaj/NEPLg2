---
id: ISS-20260429T002234025Z-SELF-HOST-CLI-LACKS-DRIVER-BOUNDARY--8BEB31A0
title: "self-host CLI lacks driver boundary from parsed options to pipeline"
area: selfhost
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/neplg2/cli/driver.nepl, stdlib/neplg2/cli/args.nepl, stdlib/neplg2/cli/reporter.nepl, tests/stdlib/selfhost_cli_driver.n.md"
---

# ISS-20260429T002234025Z-SELF-HOST-CLI-LACKS-DRIVER-BOUNDARY--8BEB31A0: self-host CLI lacks driver boundary from parsed options to pipeline

## 概要

doc/neplg2/self_host_plan.md S6 requires cli/driver.nepl to integrate compile result, exit code, artifact write, and diagnostics, but stdlib/neplg2/cli currently has args, reporter, and main only. Future CLI work would have to call core pipeline and reporter ad hoc.

## 対象

- `stdlib/neplg2/cli/driver.nepl, stdlib/neplg2/cli/args.nepl, stdlib/neplg2/cli/reporter.nepl, tests/stdlib/selfhost_cli_driver.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S6 は `cli/driver.nepl` が compile result、exit code、artifact write を統合すると定義している。
- `stdlib/neplg2/cli/` には `args.nepl`、`args/types.nepl`、`reporter.nepl`、`main.nepl` だけがあり、parsed options から core pipeline へ渡す orchestration 層が存在しない。
- `core/pipeline.nepl` には `SelfhostCompileRequest` と `selfhost_pipeline_load_root` があり、`cli/args.nepl` には `selfhost_cli_options_to_compile_options` があるため、driver 境界を filesystem 非依存の VFS 入口として先に作れる。

## 問題

doc/neplg2/self_host_plan.md S6 requires cli/driver.nepl to integrate compile result, exit code, artifact write, and diagnostics, but stdlib/neplg2/cli currently has args, reporter, and main only. Future CLI work would have to call core pipeline and reporter ad hoc.

## 影響

Self-host CLI parity cannot stabilize exit-code and diagnostic behavior, and file_io/artifact work has no single boundary to connect parsed argv, VFS loading, pipeline diagnostics, and reporter output.

## 修正方針

Add cli/driver.nepl as the CLI orchestration boundary. Start with a filesystem-independent VFS driver that converts SelfhostCliOptions into SelfhostCompileRequest, handles missing input as diagnostic, calls core pipeline, owns diagnostics, and exposes exit code plus reporter-ready diagnostics.

## 対応結果

`stdlib/neplg2/cli/driver.nepl` を追加し、filesystem 非依存の VFS driver 境界を実装した。`SelfhostCliOptions.input` を root path として `SelfhostCompileRequest` を作り、`selfhost_cli_options_to_compile_options` で core options へ変換し、`selfhost_pipeline_load_root` から root module load を開始する。

driver result は `SelfhostCliDriverResult` とし、exit code と `SelfhostDiagnostics` を所有する。missing input は `selfhost.cli.missing_input` diagnostic、missing file は loader diagnostic、root load 成功は exit code 0 / empty diagnostics として返す。diagnostic output は `selfhost_cli_driver_write_human_stderr` / `selfhost_cli_driver_write_json_stdout` から reporter 境界へ委譲し、driver は `std/fs` / `std/stdio` を直接 import しない。

`tests/stdlib/selfhost_cli_driver.n.md` を追加し、successful VFS root load、missing input JSON diagnostic、missing file loader diagnostic を固定した。`nodesrc/test_selfhost_cli_driver_boundary.js` で driver が args / reporter / loader / pipeline 境界を使い、stdio/fs を直接持たないことを固定した。

## 検証

Add focused driver doctests/fixtures for successful VFS compile request loading, missing input diagnostic, missing file diagnostic, exit code, and reporter JSON rendering.

- `node nodesrc/test_selfhost_cli_driver_boundary.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib\neplg2\cli\driver.nepl --no-tree -o tmp\selfhost-cli-driver-rebased-module.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_driver.n.md --no-tree -o tmp\selfhost-cli-driver-rebased-fixture.json -j 1`: total=3 passed=3
- `node nodesrc/tests.js -i stdlib\neplg2\cli\args.nepl -i stdlib\neplg2\cli\args\types.nepl -i stdlib\neplg2\cli\reporter.nepl -i stdlib\neplg2\cli\driver.nepl -i tests\stdlib\selfhost_cliarg_parser.n.md -i tests\stdlib\selfhost_cli_reporter.n.md -i tests\stdlib\selfhost_cli_driver.n.md --no-tree -o tmp\selfhost-cli-driver-rebased-focused.json -j 2`: total=24 passed=24
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-cli-driver-rebased-neplg2.json -j 2`: total=35 passed=35
