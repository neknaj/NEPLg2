---
id: ISS-20260515T230145475Z-RESOURCE-EFFECT-RETURN-ESCAPE-MODULE-2ED8211B
title: "resource effect return escape module exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/resource/effect_return_escape.rs, nepl-core/src/resource/effect_return_owner_type.rs, nepl-core/src/resource/effect_return_protection.rs, nepl-core/src/resource/effect_return_escape_tests.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T230145475Z-RESOURCE-EFFECT-RETURN-ESCAPE-MODULE-2ED8211B: resource effect return escape module exceeds responsibility split limit

## 概要

After splitting owner variant utility source-list helpers, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: effect_return_escape.rs has 363 lines while the enforced limit is 120. Public raw identity escape checking, protected owner provenance handling, and diagnostic classification appear to have re-accumulated in one effect module.

## 対象

- `nepl-core/src/resource/effect_return_escape.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が owner variant source-list 分割後に `effect_return_escape.rs has 363 lines; responsibility split limit is 120` を報告した。
- `effect_return_escape.rs` は public raw identity escape checking、protected owner provenance handling、diagnostic classification のような複数責務を持っている可能性が高い。
- effect return escape は public raw address identity leak を防ぐ静的検査の境界なので、monolithic helper に戻るとメモリ安全の監査単位が崩れる。

## 問題

After splitting owner variant utility source-list helpers, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: effect_return_escape.rs has 363 lines while the enforced limit is 120. Public raw identity escape checking, protected owner provenance handling, and diagnostic classification appear to have re-accumulated in one effect module.

## 影響

Effect return escape checks guard public raw address identity leaks. If this module stays monolithic, memory-safety and effect-safety proof rules become hard to audit and regressions can be hidden behind broad helper logic.

## 修正方針

Review effect_return_escape.rs, identify coherent sub-responsibilities, split them into dedicated modules without weakening static checks, and register each module in resource/mod.rs and nodesrc/test_resource_checker_responsibility.js.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused nepl-core effect_return_escape tests, ResourceIR raw identity escape tests, node nodesrc/issues.js check --dir issues, and git diff --check.

## 2026-05-16 修正

effect return escape の public escape entry point、owner protection projection、owner carrier type traversal、tests を分離した。

- `effect_return_escape.rs` は raw identity return projection が public escape かどうかの入口と public leaf 判定だけを担当する。
- suffix traversal と owner protection 判定を `effect_return_protection.rs` へ分離した。
- `str` / `RegionToken` / structural owner carrier の型探索を `effect_return_owner_type.rs` へ分離した。
- 既存 unit tests を `effect_return_escape_tests.rs` へ移し、`resource/mod.rs` では `#[cfg(test)]` module として宣言した。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在、`mod` 宣言、line budget、`effect_return_escape.rs` が protection module を使う policy を追加した。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core effect_return_escape -- --nocapture`: 4 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_rejects_mem_ptr_return_identity_escape -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_accepts_vec_owner_result_return_identity -- --nocapture`: 1 passed
- `node nodesrc/issues.js check --dir issues`
