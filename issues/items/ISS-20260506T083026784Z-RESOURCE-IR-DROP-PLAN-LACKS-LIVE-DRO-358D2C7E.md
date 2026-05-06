---
id: ISS-20260506T083026784Z-RESOURCE-IR-DROP-PLAN-LACKS-LIVE-DRO-358D2C7E
title: "Resource IR drop plan lacks live drop facts and parameter scope"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource, nepl-core/src/passes/drop_insertion.rs"
---

# ISS-20260506T083026784Z-RESOURCE-IR-DROP-PLAN-LACKS-LIVE-DRO-358D2C7E: Resource IR drop plan lacks live drop facts and parameter scope

## 概要

`ResourceDropPlan` は non-Copy の EndScope 候補を列挙しているが、その候補が実際にその EndScope 到達時点で `Initialized` かどうかは `CellState` traversal 側でしか分からなかった。また function parameter は HIR `insert_drops` の outer scope では drop されるが、Resource IR 上には対応する EndScope anchor がなかった。

## 対象

- `nepl-core/src/resource, nepl-core/src/passes/drop_insertion.rs`

## 根拠

- `ResourceDropPlan` の candidate は型と EndScope locals だけで作られるため、move 済み local も candidate として残り得る。
- 現行 HIR `insert_drops` は function parameter を別 scope として処理しているが、Resource IR lowering は body block の local EndScope しか生成していなかった。
- Stage 4 の drop elaboration 移行では、checker と codegen が同じ Resource IR state を authority にする必要がある。

## 問題

`ResourceDropPlan` の候補をそのまま codegen に接続すると、move 済み値を再 drop する危険がある。一方で function parameter の Drop obligation は Resource IR 上に insertion anchor がないため、HIR scope walker を消すと parameter drop が抜ける危険がある。

## 影響

codegen が candidate plan を直接消費すると double drop / missing drop のどちらも起き得る。これを避けるために HIR scope walker を残すと、Resource IR の `CellState` が drop elaboration authority にならず、Stage 4 の複雑化解消が完了しない。

## 修正方針

Resource IR initialized-state traversal が実際に `Initialized` と判定して auto-drop した point を `ResourceFunctionCheck::auto_drop_points` として記録する。あわせて non-Copy function parameter の EndScope anchor を terminator return 前に生成する。

## 検証

Resource IR regression で次を確認する。

- live local の auto-drop point が `ResourceFunctionCheck::auto_drop_points` に記録される。
- move 済み outer local は live auto-drop point に出ない。
- unused non-Copy parameter は Resource IR の EndScope anchor と live auto-drop point に出る。

## 対応結果

`ResourceFunctionCheck` に `auto_drop_points` を追加し、initialized-state checker の EndScope 処理が live initialized non-Copy local だけを `ResourceDropPoint` として記録するようにした。summary 計算用の補助 engine ではこの記録を使わないため、dummy path の記録は呼び出しごとに破棄する。

Resource IR lowering は non-Copy function parameter を function exit の EndScope として出すようにした。これにより、現在 HIR `insert_drops` が outer scope で扱っている parameter drop obligation が Resource IR 上にも typed anchor を持つ。

## 実施した検証

- `cargo test -p nepl-core --test resource_ir resource_ir_check_auto_drops_live_non_copy_local_at_scope_end -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_scope_auto_drop_keeps_same_type_shadowed_locals_distinct -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_live_auto_drop_points_include_function_parameters -- --nocapture`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_rejects -- --nocapture`: 8/8 passed
- `cargo test -p nepl-core --test drop -- --nocapture`: 17/17 passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/drop.n.md -o output/live_drop_points_drop.json --runner wasm --no-tree -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/compiler/shadowing.n.md -o output/live_drop_points_shadowing.json --runner wasm --no-tree -j 1`: 27/27 passed
- `node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md -o output/live_drop_points_drop_overwrite.json --runner wasm --no-tree -j 1`: 1/1 passed
