# stdlib std io fs env test review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/std/{stdio,streamio,io,iotarget,fs,env,text,test,prelude_base}.nepl`
- `stdlib/std/stdio/**`
- `stdlib/std/streamio/**`
- `stdlib/std/fs/**`
- `stdlib/std/env/**`
- `stdlib/std/test/**`

## 良い点

`std/stdio` は root facade と `ansi/debug/print/raw/read/write` に分割されている。ANSI color/style は `AnsiColor`、`AnsiTextWeight`、`AnsiTextDecoration`、`AnsiTextStyle` を持ち、色付き文字を enum と exhaustive match で扱う。

`std/streamio` は scanner/writer/input/output/bytes に分かれ、scanner state、number parse、writer append/state が分離されている。selfhost CLI や tutorial の I/O には重要な基盤である。

`std/fs` は raw/wasi/llvm/fd/read/write/path/stat/dir に分割され、WASI/LLVM raw syscall 境界を module として分けている。

`std/test` は assertion/report/types に分割され、`AssertionStatus` / `AssertionKind` / `TestAssertion` のような enum/struct を持つ。stdout report と exit code を分ける方針は、`.n.md` test commonization に必要である。

## 問題とリスク

stdio/fs/env/streamio は外部 I/O と scratch buffer を扱うため、`alloc_raw`、`mem_ptr_addr`、WASI syscall buffer、raw read/write summary に依存する。これは stdlib の中でも ResourceIR と最も強く結びつく領域である。

`std/test` は良い方向へ進んでいるが、open issue `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` の通り、すべての `.n.md` test 運用が stdout assertion report + exit code に統一されたわけではない。

debug/stdio は color output と terminal capability の問題を含む。ANSI code は typed 化されているが、terminal が非対応の場合の挙動や raw mode restore は platform/TUI 側と連携して確認する必要がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `std/stdio/ansi` | typed ANSI style。 | 良い。色付き文字問題は再設計済み方向。 |
| `std/stdio/read/write` | WASI fd read/write scratch。 | ResourceIR境界。 |
| `std/streamio` | scanner/writer split。 | selfhost/tutorialに有用。 |
| `std/fs` | raw/fd/path/read/write/stat split。 | path/IO raw境界監視。 |
| `std/env/cliarg` | raw argv acquisition + cstr。 | CLI selfhost入力に必要。 |
| `std/test` | structured assertion/report。 | 方向は良い。n.md運用移行が残る。 |

## 推奨対応

- stdio/fs/env の raw scratch buffer は public safe API と raw syscall boundary を明確に分ける。
- `std/test` の stdout report は `.n.md` runner と Rust/selfhost common tests の計画に合わせて固定する。
- ANSI/TUI style は enum-firstを維持し、色 code を任意 i32/str に戻さない。
