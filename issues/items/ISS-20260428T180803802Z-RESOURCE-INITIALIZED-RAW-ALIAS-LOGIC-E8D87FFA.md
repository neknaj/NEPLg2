---
id: ISS-20260428T180803802Z-RESOURCE-INITIALIZED-RAW-ALIAS-LOGIC-E8D87FFA
title: "Resource initialized raw alias logic reintroduces monolithic checker size"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-05-01
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_flow.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/mod.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260428T180803802Z-RESOURCE-INITIALIZED-RAW-ALIAS-LOGIC-E8D87FFA: Resource initialized raw alias logic reintroduces monolithic checker size

## 概要

Recent CellState raw address alias fixes placed raw alias tables, summary fixed-point computation, aggregate field propagation, and call summary application directly in initialized.rs. The file grew to 1322 lines and main CI fails Source policy regressions because initialized.rs exceeds the 750-line responsibility split limit.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/mod.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T142719860Z-RESOURCE-CHECKER-IS-BECOMING-A-NEW-M-5F78F7E4](./ISS-20260428T142719860Z-RESOURCE-CHECKER-IS-BECOMING-A-NEW-M-5F78F7E4.md)
- main の GitHub Actions run `25069432337` は Source policy regressions で `initialized.rs has 1323 lines; responsibility split limit is 750` として失敗している。

## 問題

Recent CellState raw address alias fixes placed raw alias tables, summary fixed-point computation, aggregate field propagation, and call summary application directly in initialized.rs. The file grew to 1322 lines and main CI fails Source policy regressions because initialized.rs exceeds the 750-line responsibility split limit.

## 影響

This blocks main CI and repeats the static-check complexity problem: the initialized/moved checker again mixes traversal, diagnostics, raw alias state, raw alias summary computation, and function alias propagation in one module. Further RawMemoryLoadCell work would add more logic to the wrong boundary.

## 修正方針

Split raw address alias state and raw address return summary propagation into initialized_alias.rs. Keep initialized.rs responsible for CellState traversal and diagnostics only, and keep behavior unchanged through the same Resource IR regression tests.

## 修正内容

- `RawCellAddressAliases`、raw address return summary の固定点計算、call / indirect call summary 適用、aggregate field alias propagation を `nepl-core/src/resource/initialized_alias.rs` へ分離した。
- raw memory operation の CellState 処理を `nepl-core/src/resource/initialized_raw_memory.rs` へ分離し、`initialized.rs` は traversal、diagnostic emission、通常 ResourceOp の initialized/moved state 更新に集中させた。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在と line limit を追加し、同じ退行を CI の Source policy で検出できるようにした。
- 分割後の行数は `initialized.rs` 609 行、`initialized_alias.rs` 501 行、`initialized_raw_memory.rs` 237 行で、既存 guard の上限内に戻した。

## 検証

node nodesrc/test_resource_checker_responsibility.js; cargo test -p nepl-core --test resource_ir -- --nocapture; cargo check -p nepl-core --tests; trunk build; focused move_effect/move_check doctests; node nodesrc/issues.js check

- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 73 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\resource-initialized-split-move-effect.json -j 1`: total=110, passed=110
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\resource-initialized-split-move-check.json -j 1`: total=52, passed=52
- `node nodesrc\issues.js check`: pass

## 2026-04-29 再発防止追記

GitHub Actions run `25078350935` の Source policy regressions で `initialized_alias.rs has 557 lines; responsibility split limit is 550` が検出された。直近の raw address helper summary / unknown offset alias 改善により、`initialized_alias.rs` が alias table と return-summary flow analysis を再び同居させた状態で上限を超えていた。

根本対策として、raw alias table と canonicalization は `initialized_alias.rs` に残し、関数 return summary の fixed-point 計算、ResourceOp 上の raw alias propagation、call-site summary 適用、aggregate field alias propagation を `initialized_alias_flow.rs` へ分離した。`nodesrc/test_resource_checker_responsibility.js` も新 module の存在と line limit を監視するよう更新した。

分割後の行数は `initialized_alias.rs` 209 行、`initialized_alias_flow.rs` 353 行で、既存の 550 行上限内に戻った。

確認:

- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --test resource_ir`: 82/82 pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\initialized-alias-flow-split-move-effect.json -j 1`: 110/110 pass
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\initialized-alias-flow-split-move-check.json -j 1`: 52/52 pass
- `node nodesrc\issues.js check`: pass

## 2026-05-01 再発

Resource checker responsibility policy の再確認で、initialized alias 系が再び上限を超えている。

- `initialized_alias.rs`: 723/550
- `initialized_alias_flow.rs`: 581/550

raw alias table、canonicalization、summary flow、call-site propagation、aggregate alias propagation の責務が再び寄り始めている。owner/lower 側の責務分割とは別 issue として、initialized alias / flow の境界を再分割する。
