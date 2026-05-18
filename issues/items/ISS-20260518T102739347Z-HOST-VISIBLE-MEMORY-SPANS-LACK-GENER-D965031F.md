---
id: ISS-20260518T102739347Z-HOST-VISIBLE-MEMORY-SPANS-LACK-GENER-D965031F
title: "Host-visible memory spans lack generic Resource IR extent proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/owner_external_io.rs, nepl-core/src/resource/initialized_external_io.rs"
---

# ISS-20260518T102739347Z-HOST-VISIBLE-MEMORY-SPANS-LACK-GENER-D965031F: Host-visible memory spans lack generic Resource IR extent proof

## 概要

Resource IR initialized and owner checks handled external host memory effects as scattered operation-specific branches. The owner checker only proved fd_read/fd_write iovec payloads and required exact extent equality, so safe subspans could be rejected while direct host output spans such as random_get buffers were not proven against backing owner extent.

## 対象

- `nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/owner_external_io.rs, nepl-core/src/resource/initialized_external_io.rs`

## 根拠

- `initialized_external_io.rs` / `initialized_external_io_input.rs` が host memory effect を直接 `ExternalIoOp` ごとの分岐で扱っており、iovec descriptor、iovec payload、direct pointer output が同じ span contract として表現されていなかった。
- `owner_external_io.rs` は fd iovec payload だけを owner extent 証明対象にしており、`random_get(buf, len)` のような direct host output span は backing owner extent と照合されていなかった。
- iovec payload 証明が exact equality を要求していたため、`alloc 16` の中へ `len 8` だけ host が書く安全な subspan まで拒否し得た。

## 問題

Resource IR initialized and owner checks handled external host memory effects as scattered operation-specific branches. The owner checker only proved fd_read/fd_write iovec payloads and required exact extent equality, so safe subspans could be rejected while direct host output spans such as random_get buffers were not proven against backing owner extent.

## 影響

Static memory safety for host calls depended on individual effect branches instead of a typed span contract. A host write could target more bytes than the allocation owner proves, and future ExternalIo/Nondet operations could bypass Resource IR extent checks.

## 修正方針

Introduce a typed host-memory span contract consumed by initialized and owner checkers. Use generic direct/iovec span handling, bounded initialized byte ranges for host output, and owner-extent coverage proof rather than exact equality for host-visible spans.

## 検証

Run focused Resource IR tests for iovec subspan coverage, random_get owner extent rejection, and random_get bounded initialized range; run cargo check, cargo fmt, Resource checker responsibility policy, issue validation, and diff check.

## 解決内容

- `host_memory_contract.rs` を追加し、host が読む/書く raw memory span を `HostMemorySpan` enum として typed contract 化した。
- initialized checker は direct output byte span を unknown-offset 全体ではなく bounded byte range として記録し、direct input / iovec input を同じ span contract から検査する。
- owner checker は direct span と iovec payload を同じ extent proof path で検査し、host-visible span については exact equality ではなく coverage proof を使う。
- regression として fd_read iovec subspan 許可、random_get output owner extent mismatch 拒否、random_get initialized range の境界外 read 拒否を追加した。
