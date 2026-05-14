---
id: ISS-20260514T182018325Z-STD-FS-AND-STDIO-ROOT-FACADES-RE-EXP-9492D2E7
title: "std fs and stdio root facades re-export raw ABI helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/std/fs.nepl, stdlib/std/stdio.nepl, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js, nodesrc/test_stdlib_stdio_read_boundary.js"
---

# ISS-20260514T182018325Z-STD-FS-AND-STDIO-ROOT-FACADES-RE-EXP-9492D2E7: std fs and stdio root facades re-export raw ABI helpers

## 概要

The safe std/fs and std/stdio root facades publicly re-export their raw submodules, so ordinary imports expose WASI/LLVM ABI helpers and raw scratch boundaries.

## 対象

- `stdlib/std/fs.nepl, stdlib/std/stdio.nepl, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js, nodesrc/test_stdlib_stdio_read_boundary.js`

## 根拠

- `stdlib/std/fs.nepl` は safe filesystem facade であるにもかかわらず、`pub #import "./fs/raw" as *` により WASI / LLVM raw syscall boundary を通常 import 面へ混ぜていた。
- `stdlib/std/stdio.nepl` も同様に、`pub #import "./stdio/raw" as *` により `fd_read` / `fd_write` / syscall fallback helper を root import から見える形にしていた。
- 実装 module はすでに `std/fs/raw` / `std/stdio/raw` を明示 import しており、root facade が raw ABI helper を再公開しなくても内部境界は維持できる。

## 問題

The safe std/fs and std/stdio root facades publicly re-export their raw submodules, so ordinary imports expose WASI/LLVM ABI helpers and raw scratch boundaries.

## 影響

Raw ABI helpers remain discoverable through safe stdlib imports, weakening Stage 6 internal/public separation and making raw-memory-backed public API migration harder to audit.

## 修正方針

Stop re-exporting ./fs/raw and ./stdio/raw from root facades, keep explicit std/fs/raw and std/stdio/raw submodules for implementation, and add source policy coverage that root facades do not expose raw ABI helpers.

## 検証

Run stdlib fs/stdio source policy tests and focused std/fs and std/stdio doctests.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / memory / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 2026-05-15 Agent 1 解決

`std/fs` root facade から `pub #import "./fs/raw" as *` を削除し、`std/stdio` root facade から `pub #import "./stdio/raw" as *` を削除した。通常の `#import "std/fs" as *` / `#import "std/stdio" as *` は safe public API だけを公開し、WASI / LLVM ABI helper や raw scratch helper は `std/fs/raw` / `std/stdio/raw` を明示 import した実装境界に残す。

source policy では root facade が raw submodule を再公開しないこと、raw helper 名を root に戻さないこと、fd/read/write 実装が raw ABI 境界を必要とする場合に explicit raw import を持つことを固定した。

検証:

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_stdio_read_boundary.js`
- `node nodesrc/test_stdlib_stdio_print_i32_boundary.js`
- `node nodesrc/test_stdlib_stdio_debug_boundary.js`
- `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/agent1-std-facade-raw-boundary-fs-root.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/agent1-std-facade-raw-boundary-stdio-root.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/std/fs/fd.nepl -i stdlib/std/fs/read/fd.nepl -i stdlib/std/fs/write/fd.nepl --no-tree -o tmp/agent1-std-facade-raw-boundary-fs-internal.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/std/stdio/write/fd.nepl -i stdlib/std/stdio/read/buffer.nepl --no-tree -o tmp/agent1-std-facade-raw-boundary-stdio-internal.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/agent1-std-facade-raw-boundary-tests-fs.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/stdout.n.md --no-tree -o tmp/agent1-std-facade-raw-boundary-tests-stdout.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/agent1-std-facade-raw-boundary-tests-stdin.json -j 1 --dist web/dist --assert-io`
