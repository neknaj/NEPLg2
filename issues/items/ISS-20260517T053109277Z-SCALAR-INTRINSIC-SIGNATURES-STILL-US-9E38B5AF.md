---
id: ISS-20260517T053109277Z-SCALAR-INTRINSIC-SIGNATURES-STILL-US-9E38B5AF
title: "scalar intrinsic signatures still use duplicated string classifiers"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/typecheck/prefix_check.rs
---

# ISS-20260517T053109277Z-SCALAR-INTRINSIC-SIGNATURES-STILL-US-9E38B5AF: scalar intrinsic signatures still use duplicated string classifiers

## 概要

typecheck/prefix_check.rs still computes scalar/cast/string intrinsic result types with one intrin.name string if-chain and validates argument types with a second string if-chain. This duplicates intrinsic spelling and signature facts outside a typed enum domain.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs`

## 根拠

- `prefix_check.rs` は `i32_to_f32` / `reinterpret_i32_f32` / `str_addr` などの scalar intrinsic について、result type 決定と argument type validation を別々の `intrin.name` string if-chain で実装していた。
- この構造では intrinsic name、arity、input type、output type が 1 つの型付き contract にならず、片方の if-chain だけを更新する drift を Rust の `match` 網羅性で検出できない。
- 静的検査自体も静的に監査しやすくする方針に反し、checker の signature table が文字列分岐として分散していた。

## 問題

typecheck/prefix_check.rs still computes scalar/cast/string intrinsic result types with one intrin.name string if-chain and validates argument types with a second string if-chain. This duplicates intrinsic spelling and signature facts outside a typed enum domain.

## 影響

A scalar intrinsic can have its result type changed without updating argument validation, or vice versa. The compiler cannot use Rust match exhaustiveness to verify that every intrinsic has a coherent arity, input type, and output type, which weakens the static checker itself.

## 修正方針

- `ScalarIntrinsicKind` と `ScalarIntrinsicType` を導入し、scalar intrinsic の spelling、arity、input type、output type を enum domain に集約する。
- `prefix_check.rs` は `ScalarIntrinsicKind::from_intrinsic_name` を 1 回だけ計算し、result type と argument validation の両方で同じ typed signature を読む。
- source policy で `prefix_check.rs` への direct scalar intrinsic branch spelling 再導入を拒否する。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core scalar_intrinsic --lib -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test neplg2 intrinsic_arg_type_mismatch_has_type_code -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
