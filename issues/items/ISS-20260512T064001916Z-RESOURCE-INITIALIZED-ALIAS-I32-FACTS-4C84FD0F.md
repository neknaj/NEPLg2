---
id: ISS-20260512T064001916Z-RESOURCE-INITIALIZED-ALIAS-I32-FACTS-4C84FD0F
title: "Resource initialized alias i32 facts exceeds split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_alias_i32_facts.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T064001916Z-RESOURCE-INITIALIZED-ALIAS-I32-FACTS-4C84FD0F: Resource initialized alias i32 facts exceeds split limit

## 概要

`initialized_alias_i32_facts.rs` は i32 fact の記録、scalar canonicalization、条件真偽の派生推論を同居させており、責務分割上限 180 行に対して 318 行まで肥大化していた。

## 対象

- `nepl-core/src/resource/initialized_alias_i32_facts.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `initialized_alias_i32_facts.rs has 318 lines; responsibility split limit is 180` を報告した。
- 分割後の行数は `initialized_alias_i32_facts.rs` 145 行、`initialized_alias_i32_condition.rs` 176 行。

## 問題

条件真偽の派生推論は relation / scale / contradiction を辿る判定ロジックであり、i32 fact の保存・更新 API と同じ module にある必要がなかった。単一 module に残すと Resource IR の memory safety 証明で使う i32 条件の根拠が監査しにくくなる。

## 影響

i32 fact propagation は computed raw size や variant condition の Resource IR 証明に使われるため、責務境界が曖昧だと型安全・メモリ安全の検査根拠を追跡しにくくなる。

## 修正方針

上限は緩和せず、条件真偽の派生推論を `initialized_alias_i32_condition.rs` に分離した。`initialized_alias_i32_facts.rs` は scalar canonicalization、fact 記録、relation/difference/scale の問い合わせに集中させ、`resource/mod.rs` と responsibility test に新 module を登録した。

## 検証

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `git diff --check`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_alias_i32_facts.rs` は解消済み。次の未対応問題として `initialized_alias_tests.rs` の 139 行 > 120 行を検出し、`ISS-20260512T065044708Z-RESOURCE-INITIALIZED-ALIAS-TESTS-EXC-3A0BF130` を追加した。
