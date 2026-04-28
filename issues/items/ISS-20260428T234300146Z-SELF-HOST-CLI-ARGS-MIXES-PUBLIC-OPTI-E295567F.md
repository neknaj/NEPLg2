---
id: ISS-20260428T234300146Z-SELF-HOST-CLI-ARGS-MIXES-PUBLIC-OPTI-E295567F
title: "self-host CLI args mixes public option types with parser implementation"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/args.nepl, stdlib/neplg2/cli/args/types.nepl"
---

# ISS-20260428T234300146Z-SELF-HOST-CLI-ARGS-MIXES-PUBLIC-OPTI-E295567F: self-host CLI args mixes public option types with parser implementation

## 概要

stdlib/neplg2/cli/args.nepl contains the public target/emit/profile/error/options type model, Copy impls, default constructors, string classifiers, parser loop, and doctests in one large file. This conflicts with the self_host_plan.md S6 split between argv parser, driver, reporter, and typed option boundary, and makes later CLI/reporting work touch the whole parser file.

## 対象

- `stdlib/neplg2/cli/args.nepl, stdlib/neplg2/cli/args/types.nepl`

## 根拠

- `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF` の 2026-04-28 再レビューで、`stdlib/neplg2/cli/args.nepl` は parser state、enum、doctest、classifier が同居する分割候補として記録されている。
- `doc/neplg2/self_host_plan.md` の S6 は `cli/args.nepl` を pure parser とし、`cli/reporter.nepl`、`cli/driver.nepl`、`cli/file_io.nepl` と分ける計画を示している。
- 現状の `stdlib/neplg2/cli/args.nepl` には public enum/struct (`SelfhostCliTarget`、`SelfhostCliEmit`、`SelfhostCliEmitSet`、`SelfhostCliProfile`、`SelfhostCliErrorKind`、`SelfhostCliOptions`) と parser loop / string classifier が同居している。

## 問題

stdlib/neplg2/cli/args.nepl contains the public target/emit/profile/error/options type model, Copy impls, default constructors, string classifiers, parser loop, and doctests in one large file. This conflicts with the self_host_plan.md S6 split between argv parser, driver, reporter, and typed option boundary, and makes later CLI/reporting work touch the whole parser file.

## 影響

Self-host CLI changes have a larger conflict surface and the public option contract is harder to reuse from driver/reporter modules without dragging parser implementation details with it.

## 修正方針

Create a focused cli/args/types.nepl module for public CLI option enums/structs and Copy impls. Keep neplg2/cli/args as the compatibility facade and parser implementation by re-exporting the types module with pub #import. Do not change public import path or parser behavior.

## 検証

Run self-host CLI args focused doctests, tests/stdlib/selfhost_cliarg_parser.n.md, the source policy regressions for CLI args/outcome, node nodesrc/issues.js check, and git diff --check.

## 修正内容

- `stdlib/neplg2/cli/args/types.nepl` を追加し、`SelfhostCliTarget`、`SelfhostCliEmit`、`SelfhostCliEmitSet`、`SelfhostCliProfile`、`SelfhostCliErrorKind`、`SelfhostCliOptions` と Copy / Clone impl を parser 実装から分離した。
- `stdlib/neplg2/cli/args.nepl` は `pub #import "./args/types" as *` により既存 import path を保つ compatibility facade とし、parser 固有の `SelfhostCliArgKind` / classifier / parse loop を残した。
- `nodesrc/test_selfhost_cli_args_types_split.js` を追加し、public option 型が `args/types.nepl` にあり、`args.nepl` へ戻らないことを固定した。
- `stdlib/neplg2/README.md` に S6 CLI boundary と `cli/args/types.nepl` の役割を追記した。

## 検証結果

- `node nodesrc/test_selfhost_cli_args_types_split.js`: pass
- `node nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`: pass
- `node nodesrc/tests.js -i stdlib\neplg2\cli\args\types.nepl --no-tree -o tmp\selfhost-cli-args-types-split-types.json -j 1`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib\neplg2\cli\args.nepl --no-tree -o tmp\selfhost-cli-args-types-split-args.json -j 1`: total=5, passed=5
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cliarg_parser.n.md --no-tree -o tmp\selfhost-cli-args-types-split-fixture.json -j 1`: total=10, passed=10
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-cli-args-types-split-neplg2.json -j 1`: total=33, passed=24, failed=9。追加した `args/types` doctest と `args.nepl` は pass。残件は既知の Vec element provenance 系。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
