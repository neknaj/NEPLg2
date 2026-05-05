---
id: ISS-20260505T185456921Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-06DA441F
title: "Resource initialized alias module exceeds responsibility split limit again"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_i32.rs, nepl-core/src/resource/initialized_alias_rank.rs, nepl-core/src/resource/initialized_alias_flow.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T185456921Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-06DA441F: Resource initialized alias module exceeds responsibility split limit again

## 概要

After resolving the Resource IR lowering traversal split, the direct Resource checker responsibility policy now reaches initialized_alias.rs and reports 619 lines over the 550-line limit. Raw alias table logic has grown again after prior alias-flow extraction, so initialized alias canonicalization and summary support need another semantic split instead of raising the limit.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_i32.rs, nepl-core/src/resource/initialized_alias_rank.rs, nepl-core/src/resource/initialized_alias_flow.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は `lower.rs` 分割後、次の未解決責務違反として `initialized_alias.rs has 619 lines; responsibility split limit is 550` を報告する。
- `ISS-20260428T180803802Z-RESOURCE-INITIALIZED-RAW-ALIAS-LOGIC-E8D87FFA` で raw alias flow は一度 `initialized_alias_flow.rs` へ分離済みだが、その後の unknown-offset alias / aggregate field alias / MemPtr raw view 対応により `initialized_alias.rs` 本体が再び上限を超えた。
- Stage 4 Resource check では initialized alias fact が CellState と owner check の根拠になるため、table API、canonicalization、projection/query helper、summary support が同じ module に戻ると memory-safety audit が難しくなる。

## 問題

After resolving the Resource IR lowering traversal split, the direct Resource checker responsibility policy now reaches initialized_alias.rs and reports 619 lines over the 550-line limit. Raw alias table logic has grown again after prior alias-flow extraction, so initialized alias canonicalization and summary support need another semantic split instead of raising the limit.

## 影響

Initialized raw alias facts feed CellState, owner checks, and raw memory safety diagnostics. If initialized_alias.rs keeps growing, changes to unknown-offset aliases, MemPtr raw views, and aggregate field aliases become harder to audit and can hide memory-safety regressions.

## 修正方針

Review initialized_alias.rs by responsibility and extract the next coherent boundary, such as raw alias canonicalization, alias query/projection helpers, or summary/call application helpers, while keeping initialized_alias.rs focused on the RawCellAddressAliases table API and preserving source-policy guards.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused Resource IR raw alias/cell state tests.

## 2026-05-06 対応結果

`initialized_alias.rs` から i32 value/condition fact と canonical/owner alias ranking を分離した。

- `initialized_alias_i32.rs`: raw address に付随する i32 exact value fact、condition fact、condition implication を担当する。
- `initialized_alias_rank.rs`: alias group の canonical ordering、owner cell alias ranking、owner alias が raw projection を持つかの判定を担当する。
- `initialized_alias.rs`: `RawCellAddressAliases` table API、alias group merge/clear/copy/move、projected alias query に集中する。

分割後の行数は `initialized_alias.rs` 513 lines、`initialized_alias_i32.rs` 34 lines、`initialized_alias_rank.rs` 90 lines で、`initialized_alias.rs` の上限を 550 から 520 に下げて再肥大化を検出できるようにした。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_tracks_external_non_copy_raw_load_after_first_move -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_canonicalizes_raw_address_local_reads -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check -- --nocapture`: 29 passed / 17 failed。失敗は既存の `ShadowSameSignatureCallable` warning を `typecheck_resource_source` helper が失敗扱いする問題。
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_alias.rs` 超過は解消。次の別件として `initialized_summary.rs has 83 lines; responsibility split limit is 80` を検出したため、`ISS-20260505T185947027Z-RESOURCE-INITIALIZED-SUMMARY-MODEL-E-63167AA8` を追加した。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: downstream policy は継続実行。`initialized_summary.rs` 別件を warning として確認した。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
