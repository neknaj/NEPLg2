---
id: ISS-20260512T061056848Z-RESOURCE-CHECKER-POLICY-EXPECTS-RENA-E8F94116
title: "Resource checker policy expects renamed i32 facts module"
area: core
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-12
updated: 2026-05-12
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/mod.rs"
---

# ISS-20260512T061056848Z-RESOURCE-CHECKER-POLICY-EXPECTS-RENA-E8F94116: Resource checker policy expects renamed i32 facts module

## 概要

remote main commit `3487e386` で `nepl-core/src/resource/initialized_direct_call_scalar.rs` が `i32_call_facts.rs` に rename され、`mod.rs` も `mod i32_call_facts;` になった。しかし `nodesrc/test_resource_checker_responsibility.js` は旧 file/module 名を期待したままだったため、source policy が stale filename で止まっていた。

## 対象

- `nodesrc/test_resource_checker_responsibility.js`
- `nepl-core/src/resource/mod.rs`
- `nepl-core/src/resource/i32_call_facts.rs`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `missing resource module: initialized_direct_call_scalar.rs` で失敗した。
- `nepl-core/src/resource/mod.rs` は `mod i32_call_facts;` を宣言している。
- `nepl-core/src/resource/initialized.rs` と `nepl-core/src/resource/owner_check.rs` は `super::i32_call_facts::record_direct_call_i32_facts` を参照している。

## 問題

実装側の rename に source policy が追従していないため、責務分割の検査が実際の module 構成を見られず、旧 file 名の存在確認だけで停止していた。

## 影響

Resource checker responsibility policy が stale failure で止まり、実際に確認すべき line budget / module boundary の失敗が隠れる。

## 修正方針

- `nodesrc/test_resource_checker_responsibility.js` の resource module list、`mod` 宣言 list、line budget を `i32_call_facts.rs` / `mod i32_call_facts;` に更新する。
- direct-call i32 facts の責務境界は維持し、line budget は現行 file size に合わせて 180 行とする。

## 修正

- `initialized_direct_call_scalar.rs` の期待を `i32_call_facts.rs` に更新した。
- `mod initialized_direct_call_scalar;` の期待を `mod i32_call_facts;` に更新した。
- `i32_call_facts.rs` の line budget を 180 行として responsibility policy に登録した。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: stale file/module name failure は解消。次の別件として `owner_check.rs has 813 lines; responsibility split limit is 800` に到達したため、`ISS-20260512T061533036Z-RESOURCE-OWNER-CHECK-EXCEEDS-RESPONS-687898EE` を追加した。
