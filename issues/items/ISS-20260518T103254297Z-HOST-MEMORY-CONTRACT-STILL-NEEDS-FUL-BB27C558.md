---
id: ISS-20260518T103254297Z-HOST-MEMORY-CONTRACT-STILL-NEEDS-FUL-BB27C558
title: "Host memory contract needs same-call WASI ABI span coverage"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/owner_host_memory_span.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/owner_external_io.rs"
---

# ISS-20260518T103254297Z-HOST-MEMORY-CONTRACT-STILL-NEEDS-FUL-BB27C558: Host memory contract needs same-call WASI ABI span coverage

## 概要

The new typed host-memory span contract centralizes direct and iovec memory proofs, but several less-used ExternalIoOp variants with same-call pointer/length ABI were still returning an empty span list. Dependent-length operations such as args_get/environ_get are tracked separately in ISS-20260518T104225390Z-ARGS-GET-AND-ENVIRON-GET-NEED-DEPEND-64A7F146.

## 対象

- `nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/owner_external_io.rs`

## 根拠

- `host_memory_contract.rs` returned `EMPTY_SPANS` for path_create_directory, path_link, path_readlink, path_rename, path_symlink, path_unlink_file, fd_seek, fd_tell, poll_oneoff, sock_accept, sock_recv, and sock_send even though these ABI shapes carry pointer/length or fixed-size output pointers in the same call.
- iovec descriptor storage was checked for initialized cells but not for owner extent, so a descriptor owner smaller than the ABI descriptor span could pass owner checking.

## 問題

The new typed host-memory span contract centralizes direct and iovec memory proofs, but several less-used ExternalIoOp variants with same-call pointer/length ABI still intentionally returned an empty span list.

## 影響

Static checks had a single proof entry point, but operations without span entries could still avoid initialized-range and owner-extent proof until their ABI spans were modeled. This had to be fixed in the generic contract rather than by adding module-specific allowlists.

## 修正方針

Extend HostMemorySpan coverage for same-call pointer/length ExternalIoOp variants using typed enum matches and shared length forms. Split dependent spans such as args_get/environ_get into a proof-artifact issue rather than accepting unknown lengths.

## 検証

Add contract-table regression tests for each newly covered ABI shape, add Resource IR initialized/owner regressions for representative safety failures, run focused initialized/owner checks, Resource checker responsibility policy, issue validation, and cargo check.

## 解決内容

- `HostMemoryLength::ArgScaled` を追加し、poll_oneoff や iovec descriptor の `count * size` span を typed contract で表現できるようにした。
- path 系、fd_seek/fd_tell、poll_oneoff、sock_accept/sock_recv/sock_send の same-call host memory span を `HostMemorySpan` へ追加した。
- owner checker が iovec descriptor owner extent も `count * 8` として証明するようにした。
- `owner_host_memory_span.rs` を分離し、contract span の owner 検査分岐を typed `HostMemorySpan` match に閉じ込めた。
- regression として全追加 span の contract-table 一致、iovec descriptor extent mismatch、path_open path owner extent mismatch、path_open uninitialized input rejection を追加した。
