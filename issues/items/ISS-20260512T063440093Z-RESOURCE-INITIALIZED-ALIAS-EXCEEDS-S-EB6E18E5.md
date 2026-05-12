---
id: ISS-20260512T063440093Z-RESOURCE-INITIALIZED-ALIAS-EXCEEDS-S-EB6E18E5
title: "Resource initialized alias exceeds split limit after i32 fact changes"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_alias.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T063440093Z-RESOURCE-INITIALIZED-ALIAS-EXCEEDS-S-EB6E18E5: Resource initialized alias exceeds split limit after i32 fact changes

## 概要

owner summary variant conditions 分割後、`nodesrc/test_resource_checker_responsibility.js` は次の blocker として `initialized_alias.rs has 524 lines; responsibility split limit is 520` を報告した。alias tracking 本体に小さな helper が残り、policy boundary をわずかに超えていた。

## 対象

- `nepl-core/src/resource/initialized_alias.rs`
- `nepl-core/src/resource/initialized_alias_utils.rs`
- `nepl-core/src/resource/mod.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `initialized_alias.rs has 524 lines; responsibility split limit is 520` で失敗した。
- `initialized_alias.rs` 末尾には group overlap、projected alias uniqueness、place suffix removal という alias table utility が残っていた。

## 問題

alias table 本体が alias state mutation だけでなく、small utility を抱えていた。小さい helper でも本体に戻すと Resource IR の alias tracking module が再び policy boundary を超える。

## 影響

Initialized alias tracking is a core part of memory-safety analysis. Letting it grow past the responsibility boundary makes alias facts and i32 condition tracking harder to audit.

## 修正方針

上限は緩めない。group overlap、projected alias uniqueness、place suffix removal を helper module へ分離し、`initialized_alias.rs` は alias state と fact propagation の本体に集中させる。

## 修正

- `initialized_alias_utils.rs` を追加した。
- `groups_overlap`、`push_unique_projected_alias`、`place_without_suffix` を `initialized_alias_utils.rs` へ移した。
- `mod.rs` と `nodesrc/test_resource_checker_responsibility.js` に `initialized_alias_utils.rs` を登録した。
- line count は `initialized_alias.rs` 495、`initialized_alias_utils.rs` 37。

## 検証

- `cargo fmt -p nepl-core`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_alias.rs` blocker は解消。次の別件として `initialized_alias_i32_facts.rs has 318 lines; responsibility split limit is 180` に到達したため、`ISS-20260512T064001916Z-RESOURCE-INITIALIZED-ALIAS-I32-FACTS-4C84FD0F` を追加した。
