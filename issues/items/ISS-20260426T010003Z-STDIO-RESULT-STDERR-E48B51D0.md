---
id: ISS-20260426T010003Z-STDIO-RESULT-STDERR-E48B51D0
title: "diagnostic emitter needs Result-returning stdout and stderr interfaces"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: stdlib/std/stdio.nepl
source: doc/neplg2/self_host_plan.md
---

# ISS-20260426T010003Z-STDIO-RESULT-STDERR-E48B51D0: diagnostic emitter needs Result-returning stdout and stderr interfaces

## 概要

self-host CLI は diagnostics を stderr、通常 artifact や JSON を stdout / file に分けて出す必要がある。
現行 `std/stdio` の高水準 API は失敗を unit に丸め、stderr 用 public API も不足している。

## 対象

- `stdlib/std/stdio.nepl`
- `stdlib/std/streamio.nepl`
- `stdlib/neplg2/cli/reporter.nepl`

## 根拠

- `stdio_write_mem`、`print`、`println` は write error を caller へ返さない。
- `fd_write` は fd を受け取れるが、public facade は stdout 中心である。
- `streamio` のコメントにも「現在の stdio は error code を外部へ公開していない」とある。

## 問題

diagnostic 出力失敗、stdout への binary artifact 出力失敗、stderr と stdout の混線を CLI が検出できない。
セルフホスト compiler の CLI parity で exit code と出力 stream を正確に比較できない。

## 影響

コンパイル失敗時の user-visible diagnostics が欠落しても成功扱いになる可能性がある。
CI では JSON output と human diagnostic が混ざり、テスト runner が機械判定しにくくなる。

## 修正方針

`StdoutStream` / `StderrStream` を分離し、`write_bytes_result`、`write_str_result`、`flush_result` を `Result<Stream, StdErrorKind>` として公開する。
既存 `print` / `println` は互換 facade として残すが、self-host CLI は Result 版のみを使う。

## 検証

- stderr へ diagnostic を出す smoke test。
- stdout と stderr の期待値を分離する CLI JSON test。
- fd_write failure を注入できる harness を用意し、Result が Err になることを確認する。
