---
id: ISS-20260429T005246036Z-SELF-HOST-CLI-LACKS-FILESYSTEM-FILE--1B6C9F63
title: "self-host CLI lacks filesystem file_io boundary"
area: selfhost
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/neplg2/cli/file_io.nepl, stdlib/neplg2/cli/driver.nepl, tests/stdlib/selfhost_cli_file_io.n.md"
---

# ISS-20260429T005246036Z-SELF-HOST-CLI-LACKS-FILESYSTEM-FILE--1B6C9F63: self-host CLI lacks filesystem file_io boundary

## 概要

doc/neplg2/self_host_plan.md S6 requires cli/file_io.nepl to own filesystem reads and artifact writes, but stdlib/neplg2/cli currently has args, reporter, driver, and main only. The driver can compile an in-memory VFS, yet the real CLI has no dedicated boundary for reading the input source into VFS or writing artifacts.

## 対象

- `stdlib/neplg2/cli/file_io.nepl, stdlib/neplg2/cli/driver.nepl, tests/stdlib/selfhost_cli_file_io.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S6 は `cli/file_io.nepl` を input file / stdlib root / artifact 書き出しの橋渡し層として定義している。
- `stdlib/neplg2/cli/driver.nepl` は VFS を受け取る pure driver まで実装済みだが、実 filesystem から root source を読み込む CLI 層が存在しない。
- `stdlib/neplg2/cli/` に `args.nepl`、`args/types.nepl`、`reporter.nepl`、`driver.nepl`、`main.nepl` はあるが、`std/fs` を閉じ込める専用 module がない。

## 問題

doc/neplg2/self_host_plan.md S6 requires cli/file_io.nepl to own filesystem reads and artifact writes, but stdlib/neplg2/cli currently has args, reporter, driver, and main only. The driver can compile an in-memory VFS, yet the real CLI has no dedicated boundary for reading the input source into VFS or writing artifacts.

## 影響

Self-host CLI integration would either duplicate std/fs calls in main/driver or let core-facing orchestration depend directly on filesystem details. That makes exit-code, diagnostics, and artifact-output parity harder to test independently.

## 修正方針

Add cli/file_io.nepl as the CLI filesystem bridge. Keep core and driver VFS-facing, expose Result-returning helpers for reading a root source into SelfhostVirtualFileSystem and for writing text/binary artifacts, and add source policy plus focused doctests.

## 対応

`stdlib/neplg2/cli/file_io.nepl` を追加し、`std/fs` 依存を CLI filesystem bridge に閉じ込めた。root source は `fs_read_to_string_checked` で UTF-8 検証してから `SelfhostVirtualFileSystem` に登録し、read failure / VFS allocation failure / artifact write failure は `SelfhostDiagnostic` として返す。

text artifact は `fs_write_to_string`、binary artifact は `fs_write_to_bytes` に委譲し、driver と core pipeline は VFS-facing のまま維持した。`nodesrc/test_selfhost_cli_file_io_boundary.js` で `std/fs` import が `cli/file_io.nepl` 以外の `stdlib/neplg2` module に漏れないことを固定した。

## 検証

Add focused file_io doctests/fixtures for source file read into VFS, read error mapping, text artifact write, and source policy that confines std/fs imports to cli/file_io.

- `node nodesrc/test_selfhost_cli_file_io_boundary.js`: passed
- `trunk build`: passed after rebasing onto `d764be7`
- `node nodesrc/tests.js -i stdlib\neplg2\cli\file_io.nepl --no-tree -o tmp\selfhost-cli-file-io-module-rebased.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_file_io.n.md --no-tree -o tmp\selfhost-cli-file-io-fixture-rebased.json -j 1`: total=4 passed=4
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\selfhost-cli-file-io-neplg2-rebased.json -j 2`: remote `d764be7` の Resource IR owner obligation leak により file_io 以外で failed。`hash32(str)` / `StringBuilder` の別 issue として分離する。
