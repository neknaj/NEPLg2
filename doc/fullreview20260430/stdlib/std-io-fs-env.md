# stdlib std / io / fs / env review

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 概要

`stdlib/std` は WASI-facing layer であり、selfhost CLI、test runner、diagnostic output に直結する。`fs.nepl`、`stdio.nepl`、`streamio.nepl` は巨大で、raw out-pointer、ByteBuf、StringBuilder、Result owner summary の影響を強く受ける。

## Actions で確認した状態

`stdlib-test` artifact では std/io/fs/stdio/env 関連が 33 件失敗している。

主な傾向:

- `stdio_write_fd_mem_result` の `resource.owner.maybe_leak`。
- `fs_open_with_flags` / read/write path の owner summary failure。
- `cliarg_count` / `cliarg_get` 周辺の owner/out-pointer failure。
- `stdlib/neplg2` CLI doctest timeout は `std/test` / selfhost driver とも絡む。

## 良い点

- stdio は stdout/stderr の Result API を持ち、lossy helper と Result helper の分離が進んでいる。
- `std/text.nepl` は external bytes を UTF-8 checked `str` にする役割を持つ。
- `std/test.nepl` は `AssertionStatus` / `AssertionKind` / `TestAssertion` / `TestReport` を導入し、stdout report へ移行する方向にある。
- `fs.nepl` は path normalization、dir read、read/write bytes/string を持ち、selfhost CLI に必要な surface を提供している。

## 問題

### raw out-pointer と Resource IR

WASI は out pointer API が多く、`fd_read` / `fd_write` / `path_open` / `args_sizes_get` などで raw scratch cell が出る。現行 stdlib は raw address local と `MemPtr` projection を混ぜるため、Resource IR が initialized-cell / owner consumption を証明できない経路が残る。

### `std/test` は移行中

`std/test` は structured report に向かっているが、Actions failure では core/collection doctest が `std/test` owner issue を拾う。`.n.md` 共通運用では stdout report と exit code の両方を固定する必要がある。

### 巨大 file

`stdio.nepl` と `fs.nepl` は selfhost CLI の正規 layer として重要だが、WASI shim、buffer handling、formatting、debug/ANSI が同居している。selfhost では reporter / file_io / stream / ANSI を分けるべきである。

## selfhost への示唆

selfhost CLI は `std/fs` と `std/stdio` を使う必要があるが、core compiler は WASI を持たない。`stdlib/neplg2/core` は in-memory source / VFS / diagnostics を受け取り、`stdlib/neplg2/cli` だけが stdio/fs に依存する構造を維持する。
