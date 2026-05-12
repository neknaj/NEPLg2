---
id: ISS-20260512T065044708Z-RESOURCE-INITIALIZED-ALIAS-TESTS-EXC-3A0BF130
title: "Resource initialized alias tests exceeds split limit"
area: core
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_alias_tests.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T065044708Z-RESOURCE-INITIALIZED-ALIAS-TESTS-EXC-3A0BF130: Resource initialized alias tests exceeds split limit

## 概要

`initialized_alias_tests.rs` は i32 relation/scale propagation、raw address view origin、i32 condition derivation の回帰テストを同居させており、責務分割上限 120 行に対して 139 行まで肥大化していた。

## 対象

- `nepl-core/src/resource/initialized_alias_tests.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `initialized_alias_tests.rs has 139 lines; responsibility split limit is 120` を報告した。
- 分割後の行数は `initialized_alias_tests.rs` 66 行、`initialized_alias_raw_view_tests.rs` 39 行、`initialized_alias_i32_condition_tests.rs` 32 行、`initialized_alias_test_support.rs` 11 行。

## 問題

initialized alias の regression test が単一 file に集まっていたため、どの Resource IR の安全性根拠を守る test なのかが追いにくくなっていた。production module の責務分割に合わせて test module も関心ごとに分ける必要があった。

## 影響

Resource IR の memory-safety regression coverage が監査しにくくなり、relation/scale propagation、raw view origin、condition derivation のどれが壊れたのかを切り分けにくくなる。

## 修正方針

上限は緩和せず、共通の `local` helper を `initialized_alias_test_support.rs` に分離したうえで、raw address view origin を `initialized_alias_raw_view_tests.rs`、i32 condition derivation を `initialized_alias_i32_condition_tests.rs` に分けた。既存 `initialized_alias_tests.rs` は i32 relation/scale propagation と path merge proof の回帰に集中させた。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo check -p nepl-core --tests`: passed
