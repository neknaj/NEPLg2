---
id: ISS-20260505T231409659Z-RESOURCE-HOST-EFFECTS-KEEP-OPERATION-B5A32D01
title: "Resource host effects keep operation identity as strings"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/effects.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/initialized_external_io.rs"
---

# ISS-20260505T231409659Z-RESOURCE-HOST-EFFECTS-KEEP-OPERATION-B5A32D01: Resource host effects keep operation identity as strings

## 概要

Stage 5 raw memory effects now use typed enums, but ExternalIo and Nondet still carry operation names as String. Resource IR initialized effects still branch on string literals such as fd_read and fd_write, so host effect handling lacks exhaustive match coverage.

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/initialized_external_io.rs`

## 根拠

- `InternalEffect::{ExternalIo,Nondet}` と `EffectOp::{ExternalIo,Nondet}` が operation を `String` で保持していた。
- Resource IR の initialized external IO handling が `fd_read` / `fd_write` / `random_get` などの文字列 literal で分岐していた。
- `IMPURE_IO_EFFECT_MARKERS` に新しい host operation を追加しても、typed operation enum の追加漏れを compiler が検出できなかった。

## 問題

Stage 5 raw memory effects now use typed enums, but ExternalIo and Nondet still carry operation names as String. Resource IR initialized effects still branch on string literals such as fd_read and fd_write, so host effect handling lacks exhaustive match coverage.

## 影響

Effect safety and Resource IR memory initialization around WASI calls rely on exact host operation identity. String matching weakens compiler-checked coverage and makes future host operation additions easy to miss.

## 修正方針

Introduce ExternalIoOp and NondetOp enums, carry them through InternalEffect and EffectOp, and update initialized external IO handling and tests to match on typed operations.

## 対応

- `ExternalIoOp` / `NondetOp` を追加し、known host effect marker を enum variant へ写像する `external_io_op_from_name` / `nondet_op_from_name` を導入した。
- `InternalEffect` と Resource IR `EffectOp` の ExternalIo / Nondet は `String` ではなく typed enum を保持するようにした。
- Resource IR initialized external IO handling は typed operation の match で fd read/write/pread/pwrite、directory/stat/env/args/random output initialization を扱うようにした。
- `IMPURE_IO_EFFECT_MARKERS` が必ず `ExternalIoOp` または `NondetOp` に分類されることを回帰テストで固定した。

## 検証

- `cargo test -p nepl-core --test effects -- --nocapture`: 24 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 155 passed
- `node nodesrc/issues.js check`: commit 前に実行
