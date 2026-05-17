---
id: ISS-20260517T114805544Z-WASI-EXTERN-TARGET-GATE-HARDCODES-IM-26DFA510
title: "WASI extern target gate hardcodes import module spelling"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/extern_import.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T114805544Z-WASI-EXTERN-TARGET-GATE-HARDCODES-IM-26DFA510: WASI extern target gate hardcodes import module spelling

## 概要

typecheck driver rejects WASI imports on non-WASI targets by checking the extern module string directly against wasi_snapshot_preview1. The target gate condition is not represented as a typed import module contract, so adding or changing host import modules can drift from diagnostics and policy.

## 対象

- `nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/extern_import.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `typecheck/driver.rs` が `m == "wasi_snapshot_preview1"` と `CompileTarget::Wasm | CompileTarget::Llvm` の local branch で WASI extern import を拒否していた。
- host import module の spelling と allowed target の contract が同じ typed domain に載っておらず、別の host import module を追加した場合に driver 側の条件が更新漏れし得る。
- 既存 regression は WASI import rejection の結果だけを確認しており、driver 内の direct string guard 再導入を拒否していなかった。

## 問題

typecheck driver rejects WASI imports on non-WASI targets by checking the extern module string directly against wasi_snapshot_preview1. The target gate condition is not represented as a typed import module contract, so adding or changing host import modules can drift from diagnostics and policy.

## 影響

Compile-target safety for host imports depends on a local string branch instead of an exhaustive enum domain. That weakens the static-check implementation policy because the compiler cannot force every host-import class to define its allowed targets.

## 修正方針

Introduce a typecheck extern import module enum that owns host import module spelling and target allowance. Make driver consume it through match-based classification, and add policy/tests rejecting direct WASI module string checks in driver.

## 対応内容

- `typecheck/extern_import.rs` を追加し、`ExternImportModule::WasiSnapshotPreview1` が host import module spelling と allowed target を所有するようにした。
- `typecheck.rs` に `extern_import` module を追加した。
- `driver.rs` は `ExternImportModule::from_module_name(m)` で分類し、`module.is_allowed_for_target(target)` の exhaustive match を通して target gate を行うようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` に `extern_import.rs`、enum、target gate、driver 側の consumer、旧 direct string guard 禁止を追加した。

## 検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core typecheck::extern_import::tests --lib -- --nocapture`: 2/2 passed
- `cargo test -p nepl-core --test neplg2 wasi_import_rejected_on_wasm_target -- --exact --nocapture`: 1/1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
