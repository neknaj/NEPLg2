---
id: ISS-20260515T225118666Z-RESOURCE-OWNER-RAW-USE-CALL-SUMMARY--3E21FFD7
title: "resource owner raw use call summary helpers exceed responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/resource/owner_summary_raw_use_call.rs, nepl-core/src/resource/owner_summary_raw_use_return.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T225118666Z-RESOURCE-OWNER-RAW-USE-CALL-SUMMARY--3E21FFD7: resource owner raw use call summary helpers exceed responsibility split limit

## 概要

After splitting raw owner alias branch propagation, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: owner_summary_raw_use_call.rs has 136 lines while the enforced limit is 90. Direct-call raw owner use detection and returned raw owner alias materialization are concentrated in one helper module.

## 対象

- `nepl-core/src/resource/owner_summary_raw_use_call.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_summary_raw_alias_walk.rs` の branch 分割後に `owner_summary_raw_use_call.rs has 136 lines; responsibility split limit is 90` を報告した。
- `owner_summary_raw_use_call.rs` は direct call が raw owner alias を消費するかの検査と、summary から returned raw owner alias を materialize する処理を同居させている。
- consumption detection と return alias propagation は Resource IR summary の別境界であり、同じ helper module に残すと静的検査のレビュー単位が崩れる。

## 問題

After splitting raw owner alias branch propagation, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: owner_summary_raw_use_call.rs has 136 lines while the enforced limit is 90. Direct-call raw owner use detection and returned raw owner alias materialization are concentrated in one helper module.

## 影響

Resource IR raw owner use summaries can grow without a clear boundary between consumption detection and returned alias propagation. This weakens reviewability of memory-safety proof code and can hide regressions in raw owner summary application.

## 修正方針

Split direct-call raw owner consumption detection from returned raw owner alias materialization into separate modules, register the new module in resource/mod.rs and nodesrc/test_resource_checker_responsibility.js, and keep line limits tight.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused nepl-core raw owner summary tests, node nodesrc/issues.js check --dir issues, and git diff --check.

## 2026-05-16 修正

direct call raw owner summary helper を consumption detection と returned alias materialization に分割した。

- `owner_summary_raw_use_call.rs` は direct call が raw owner alias を消費するかの検査だけを担当する。
- summary から returned raw owner alias を materialize する処理は `owner_summary_raw_use_return.rs` へ分離した。
- `owner_summary_raw_alias_walk.rs` と `owner_summary_raw_use_walk.rs` は returned alias propagation を新 module から import する。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在、`mod` 宣言、100 行上限を追加した。

検証:

- `cargo test -p nepl-core owner_summary_raw_transfer -- --nocapture`: 4 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_accepts_vec_owner_result_return_identity -- --nocapture`: 1 passed
- `node nodesrc/test_resource_checker_responsibility.js`: `owner_summary_raw_use_call.rs` blocker は解消。次の別 issue として `owner_variant_utils.rs has 223 lines; responsibility split limit is 220` を検出したため `ISS-20260515T225627980Z-RESOURCE-OWNER-VARIANT-UTILS-EXCEEDS-E7268B15` に分離した。
