---
id: ISS-20260426T053112317Z-SELFHOST-REQ-HASHKEY-FIXTURE-FAILS-U-34A22E8C
title: "selfhost_req HashKey fixture fails under current Rust harness"
area: tests
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/tests/selfhost_req.rs
---

# ISS-20260426T053112317Z-SELFHOST-REQ-HASHKEY-FIXTURE-FAILS-U-34A22E8C: selfhost_req HashKey fixture fails under current Rust harness

## 概要

nepl-core/tests/selfhost_req.rs::test_req_trait_extensions returns 0 instead of 5 on current main even without the monomorphize trait lookup changes. The fixture contains #target std but uses run_main_i32, whose compile path hardcodes CompileTarget::Wasm, so the self-host HashKey/HashMap requirement is no longer a reliable green check.

## 対象

- `nepl-core/tests/selfhost_req.rs`

## 根拠

- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions` は現在の `main` 相当の `nepl-core/src/monomorphize.rs` に戻しても `left: 0, right: 5` で失敗する。
- 同 fixture は `#target std` を含むが、`run_main_i32` は `CompileTarget::Wasm` を指定している。
- 同じ `HashKey for Point` / `HashMap<Point,str,DefaultHash32>` 形の再現 source を std target で WAT 出力すると `HashKey::hash32__Point` と `Hasher<K>::hash32__DefaultHash32_Point` が生成されるため、Rust selfhost_req harness の target 契約と fixture の期待値を切り分ける必要がある。

## 問題

nepl-core/tests/selfhost_req.rs::test_req_trait_extensions returns 0 instead of 5 on current main even without the monomorphize trait lookup changes. The fixture contains #target std but uses run_main_i32, whose compile path hardcodes CompileTarget::Wasm, so the self-host HashKey/HashMap requirement is no longer a reliable green check.

## 影響

The HashKey user-defined key requirement can regress without a passing Rust selfhost_req check, and monomorphize / stdlib work cannot use this fixture as a trustworthy verification gate.

## 修正方針

Decide whether the requirement is std/WASI-only or bare-wasm compatible. If it is std/WASI, move this fixture to run_main_wasi_i32 or an equivalent std-target runner; if bare wasm must pass, fix the underlying HashMap/allocator/runtime path. Add a focused regression that confirms the selected target returns 5.

## 検証

cargo test -p nepl-core --test selfhost_req test_req_trait_extensions; node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 6
