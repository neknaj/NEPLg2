---
id: ISS-20260426T121027631Z-NEPL-CLI-STACK-OVERFLOWS-WHEN-A-STD--D73CA3DF
title: "nepl-cli stack overflows when a std program calls std/fs fs_read_dir"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/compiler.rs, nepl-core/src/codegen_wasm.rs, nepl-cli/src/main.rs"
---

# ISS-20260426T121027631Z-NEPL-CLI-STACK-OVERFLOWS-WHEN-A-STD--D73CA3DF: nepl-cli stack overflows when a std program calls std/fs fs_read_dir

## 概要

A nepl-cli --run program that imports std/fs and calls fs_read_dir causes the nepl-cli process to abort with Rust stack overflow on Windows. Raw WASI fd_readdir support passes, so the failure appears in the compiler/run pipeline for the std/fs facade rather than the host syscall shim itself.

## 対象

- `nepl-core/src/compiler.rs, nepl-core/src/codegen_wasm.rs, nepl-cli/src/main.rs`

## 根拠

- 未記入

## 問題

A nepl-cli --run program that imports std/fs and calls fs_read_dir causes the nepl-cli process to abort with Rust stack overflow on Windows. Raw WASI fd_readdir support passes, so the failure appears in the compiler/run pipeline for the std/fs facade rather than the host syscall shim itself.

## 影響

self-host CLI code cannot safely use the new directory traversal facade through nepl-cli, which blocks stdlib discovery from being exercised end-to-end in the Rust CLI runner.

## 修正方針

Identify the recursive compiler/codegen/drop path triggered by std/fs fs_read_dir and convert it to an iterative or bounded traversal. Add a non-ignored nepl-cli regression that imports std/fs and validates fs_read_dir once the stack overflow is fixed.

## 検証

Run cargo test -p nepl-cli run_wasi_std_fs_read_dir_returns_stable_directory_entries -- --nocapture without stack overflow, then run the full nepl-cli run_wasi_ filter.
