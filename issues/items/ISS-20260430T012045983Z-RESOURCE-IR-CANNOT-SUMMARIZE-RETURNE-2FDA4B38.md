---
id: ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38
title: "Resource IR cannot summarize returned raw headers with length-guarded dynamic ranges"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_return.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/kp.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38: Resource IR cannot summarize returned raw headers with length-guarded dynamic ranges

## 概要

Resource IR initialized-cell summaries can propagate returned raw header fields and unknown-offset initialized Copy cells, but they still cannot express a dependent invariant such as header.buf plus offsets below header.len are initialized after a loop that repeatedly fd_read's into buf + len.

## 対象

- `nepl-core/src/resource/initialized_return.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/kp.rs`

## 根拠

- `ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA` では direct `fd_read`、単発 read の returned header、`fill_i32` 済み prefix buffer の dynamic-offset read は通るようになった。
- 一方で full scanner style の loop は `write_ptr = add buf len` に対して `fd_read` を繰り返し、最後に `sc` header に `buf` / `len` / `cap` を詰めて返す。
- 現在の `initialized_return` summary は「raw cell が initialized」「raw cell の値が raw address」という事実を返せるが、「`header.len` 未満の offset は `header.buf` から initialized」という dependent fact を表現しない。
- そのため full scanner loop を source-level regression として戻すには、単なる alias rekey ではなく range owner、長さ field、loop write の関係を Resource IR の型付き summary として持つ必要がある。

## 問題

Resource IR initialized-cell summaries can propagate returned raw header fields and unknown-offset initialized Copy cells, but they still cannot express a dependent invariant such as header.buf plus offsets below header.len are initialized after a loop that repeatedly fd_read's into buf + len.

## 影響

A full scanner-style grow/read loop must be reduced or tracked outside Resource IR instead of being proven by the compiler. Leaving this implicit would hide a static-checking completeness gap for self-host input scanners.

## 修正方針

Design a typed range-summary model for returned raw headers: connect the pointer field, the length/capacity fields, and loop writes to dynamic offsets as a single initialized range fact without weakening RawMemoryLoadCell strictness.

具体的には、`initialized_return` の raw cell list だけではなく、次の関係を表す summary を追加する。

- pointer field: returned header のどの raw cell が buffer pointer か。
- length field: どの raw cell が initialized upper bound を表すか。
- capacity field: storage boundary と realloc 後の有効領域を表すか。
- write source: `fd_read` / copy / fill がどの dynamic offset range を initialized にしたか。

この summary は `load_u8 add buf i` のような caller 側の raw load を無条件に通すものではない。guard condition または Resource IR の condition fact により `i < len` が証明できる場合だけ initialized range として扱う。

## 検証

Add a source-level scanner regression that returns a header after a loop of fd_read/realloc and then reads bytes guarded by len. Keep direct fd_read and single-read returned-header regressions passing.
