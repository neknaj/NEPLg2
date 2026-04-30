---
id: ISS-20260430T141611950Z-DROP-REGRESSION-FIXTURES-USE-PURE-EN-D9FD2B4F
title: "Drop regression fixtures use pure entry points despite impure destructors"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "tests/compiler/drop.n.md, tests/compiler/drop_overwrite.n.md, nepl-core/tests/drop.rs"
---

# ISS-20260430T141611950Z-DROP-REGRESSION-FIXTURES-USE-PURE-EN-D9FD2B4F: Drop regression fixtures use pure entry points despite impure destructors

## 概要

Drop trait methods are impure, and Resource IR now checks auto-drop calls as real effectful calls. Existing drop regression fixtures still declare pure main functions, so they fail with effect.pure.calls_impure even though the intended scenario is impure drop execution.

## 対象

- `tests/compiler/drop.n.md`
- `tests/compiler/drop_overwrite.n.md`
- `nepl-core/tests/drop.rs`

## 根拠

- `Drop` trait の `drop` method は `fn drop <(&Self)*>()>` であり、destructor 実行は surface effect として impure である。
- Resource IR effect boundary gate は auto drop で挿入された `Drop::drop_*` call も通常の call effect として検査する。
- `tests/compiler/drop.n.md` / `drop_overwrite.n.md` と `nepl-core/tests/drop.rs` の多くの positive fixture は、Drop value をscope endで破棄するにもかかわらず `fn main <()->i32>` のpure entry pointを使っていた。
- compiler側で auto drop effect を無視すると、destructor内の外部I/Oやhost callを pure関数から観測できるため、静的検査の正確性を壊す。

## 問題

Drop trait methods are impure, and Resource IR now checks auto-drop calls as real effectful calls. Existing drop regression fixtures still declare pure main functions, so they fail with effect.pure.calls_impure even though the intended scenario is impure drop execution.

## 影響

The compiler and nodesrc drop regression suites fail for stale effect annotations, and changing the compiler to ignore auto-drop effects would allow pure functions to observe side effects through destructors.

## 修正方針

Keep Resource IR effect enforcement strict and update drop regression fixtures that exercise auto-drop to use impure entry points. Add a note that pure functions with impure destructors must remain rejected.

## 修正結果

- Dropの実行を期待するpositive fixtureのentry pointを `fn main <()*>i32>` へ更新した。
- `tests/compiler/drop.n.md` に `drop_impure_destructor_keeps_pure_main_rejected` を追加し、pure `main` がimpure auto dropを実行しようとする場合は `effect.pure.calls_impure` で拒否されることを固定した。
- `nepl-core/tests/drop.rs` に `pure_function_with_impure_auto_drop_is_rejected` を追加し、Rust integration test側でも同じeffect contractを確認するようにした。
- compiler / Resource IR 側のeffect gateは緩和していない。

## 検証

- `cargo test -p nepl-core --test drop -- --nocapture`: 18 passed
- `node nodesrc/tests.js -i tests/compiler/drop.n.md -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/drop-effect-fixtures-agent1.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/issues.js check`: ok, files=476
