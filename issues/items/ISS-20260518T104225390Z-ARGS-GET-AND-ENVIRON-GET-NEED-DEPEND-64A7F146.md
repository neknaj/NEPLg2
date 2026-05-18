---
id: ISS-20260518T104225390Z-ARGS-GET-AND-ENVIRON-GET-NEED-DEPEND-64A7F146
title: "args_get and environ_get need dependent host span proof"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/owner_external_io.rs"
---

# ISS-20260518T104225390Z-ARGS-GET-AND-ENVIRON-GET-NEED-DEPEND-64A7F146: args_get and environ_get need dependent host span proof

## 概要

WASI args_get/environ_get do not carry argc or buffer-size lengths in the call itself. HostMemorySpan can model same-call direct and iovec spans, but these two operations need a proof artifact that connects args_sizes_get/environ_sizes_get output cells to the later pointer-array and byte-buffer owners.

## 対象

- `nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/owner_external_io.rs`

## 根拠

- `args_get(argv, argv_buf)` / `environ_get(environ, environ_buf)` の call ABI には `argc` / `argv_buf_size` / `envc` / `environ_buf_size` が引数として存在しない。
- 同じ WASI family の `args_sizes_get` / `environ_sizes_get` がそれらの長さを host output cell として返すため、後続 call の span は同一 call の引数だけではなく prior-call proof artifact に依存する。
- これを operation 名で許可すると、実際に十分な pointer table / byte buffer owner extent が確保されたことを証明しない特例になる。

## 問題

WASI args_get/environ_get do not carry argc or buffer-size lengths in the call itself. HostMemorySpan can model same-call direct and iovec spans, but these two operations need a proof artifact that connects args_sizes_get/environ_sizes_get output cells to the later pointer-array and byte-buffer owners.

## 影響

If args_get/environ_get remain modeled as empty spans, static checking cannot prove initialized range or owner extent for the argv/environ pointer table and string buffer. Accepting them by operation name would reintroduce an unsound special case instead of a source/IR proof.

## 修正方針

Record typed host sizing facts when args_sizes_get/environ_sizes_get initialize their output cells, propagate those facts through scalar loads and allocation extents, and require args_get/environ_get to consume matching pointer-array and byte-buffer owner extents before applying output initialization.

## 検証

Add Resource IR tests for args_get/environ_get rejecting missing or mismatched prior sizing proof and accepting buffers allocated from proven sizes; run focused initialized/owner checks, policy checks, and cargo check.
