---
id: ISS-20260518T103254297Z-HOST-MEMORY-CONTRACT-STILL-NEEDS-FUL-BB27C558
title: "Host memory contract still needs full WASI ABI span coverage"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/owner_external_io.rs"
---

# ISS-20260518T103254297Z-HOST-MEMORY-CONTRACT-STILL-NEEDS-FUL-BB27C558: Host memory contract still needs full WASI ABI span coverage

## 概要

The new typed host-memory span contract centralizes direct and iovec memory proofs, but several less-used ExternalIoOp variants still intentionally return an empty span list because their ABI requires additional fixed-size or dependent-length modeling, such as args_get/environ_get, poll_oneoff, socket iovecs, and multi-path filesystem operations.

## 対象

- `nepl-core/src/resource/host_memory_contract.rs, nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/owner_external_io.rs`

## 根拠

- 未記入

## 問題

The new typed host-memory span contract centralizes direct and iovec memory proofs, but several less-used ExternalIoOp variants still intentionally return an empty span list because their ABI requires additional fixed-size or dependent-length modeling, such as args_get/environ_get, poll_oneoff, socket iovecs, and multi-path filesystem operations.

## 影響

Static checks now have a single proof entry point, but operations without span entries can still avoid initialized-range and owner-extent proof until their ABI spans are modeled. This must be finished in the generic contract rather than by adding module-specific allowlists.

## 修正方針

Extend HostMemorySpan coverage for the remaining ExternalIoOp variants using typed enum matches and shared length forms. For dependent spans such as args_get/environ_get, add an explicit proof artifact that connects prior sizes_get results to the later host call instead of accepting unknown lengths.

## 検証

Add Resource IR regression tests for each newly covered ABI shape, run focused initialized/owner checks, Resource checker responsibility policy, issue validation, and cargo check.
