---
id: ISS-20260428T203303662Z-SELF-HOST-CORE-OPTIONS-ONLY-EXPOSE-M-C4A0FEAF
title: "self-host core options only expose marker API"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/options.nepl
---

# ISS-20260428T203303662Z-SELF-HOST-CORE-OPTIONS-ONLY-EXPOSE-M-C4A0FEAF: self-host core options only expose marker API

## 概要

stdlib/neplg2/core/options.nepl still returns only selfhost_options_stage0 and does not model CompileOptions, CompileTarget, or BuildProfile. The CLI parser has its own target/profile values, but the pure core pipeline has no stable option boundary equivalent to the Rust compiler.

## 対象

- `stdlib/neplg2/core/options.nepl`

## 根拠

- Rust 側 `nepl-core/src/compiler.rs` には `CompileTarget`、`BuildProfile`、`CompileOptions` がある。
- `stdlib/neplg2/core/options.nepl` は修正前 `selfhost_options_stage0` だけで、core pipeline に渡す option value を持っていなかった。
- CLI parser 側には target/profile enum があるが、core が CLI module に依存すると `doc/neplg2/self_host_plan.md` の core/CLI 分離に反する。

## 問題

stdlib/neplg2/core/options.nepl still returns only selfhost_options_stage0 and does not model CompileOptions, CompileTarget, or BuildProfile. The CLI parser has its own target/profile values, but the pure core pipeline has no stable option boundary equivalent to the Rust compiler.

## 影響

Pipeline/check/codegen work would either depend on CLI argv structures or pass ad hoc booleans and strings across stages. That breaks the core/CLI separation in doc/neplg2/self_host_plan.md and blocks parity work for target/profile resolution.

## 修正方針

core-owned な `SelfhostCompileTarget`、`SelfhostBuildProfile`、`SelfhostCompileOptions` を追加しました。`selfhost_compile_options_default`、`selfhost_compile_options_with_target`、`selfhost_compile_options_with_profile`、`selfhost_compile_options_with_verbose` により、pipeline/check/codegen へ渡す pure option value を組み立てられます。

また、Rust 実装と同じ優先順位で option target / module target / default Wasm を解決する `selfhost_compile_resolve_target` と、profile override / caller default を解決する `selfhost_compile_resolve_profile` を追加しました。CLI から core option への変換は後続 issue に分け、core から cli への依存は入れていません。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\options.nepl --no-tree -o tmp\selfhost-core-options.json -j 1`: total=1 passed=1
