---
id: ISS-20260506T025727360Z-REMOVE-LEGACY-MOVE-CHECK-FALLBACK-AF-C143E79B
title: "Remove legacy move_check fallback after Resource IR gates"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/compiler.rs, nepl-core/src/passes/mod.rs, nepl-core/src/resource/mod.rs, nepl-core/src/resource/borrow_*.rs, nepl-core/src/resource/lower*.rs, nepl-core/src/resource/coverage_hir*.rs, nodesrc/test_resource_gate_order.js, nodesrc/test_resource_checker_responsibility.js, nepl-core/tests/check_pipeline.rs, nepl-core/tests/layout.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T025727360Z-REMOVE-LEGACY-MOVE-CHECK-FALLBACK-AF-C143E79B: Remove legacy move_check fallback after Resource IR gates

## 概要

Stage 4 still keeps passes::move_check::run as a post-Resource fallback after Resource IR lowering/cell/borrow/effect/owner gates. This preserves a second static-check authority and encourages self-host to copy legacy HIR traversal.

## 対象

- `nepl-core/src/compiler.rs, nepl-core/src/passes/mod.rs, nepl-core/src/resource/mod.rs, nodesrc/test_resource_gate_order.js, nepl-core/tests/check_pipeline.rs, nepl-core/tests/layout.rs`

## 根拠

- `compiler::run_move_check` は Resource IR lowering / cell / borrow / effect / owner gate の後で `passes::move_check::run` を呼び、Resource IR と旧 HIR checker の二重 authority を残していた。
- `nepl-core/src/passes/mod.rs` は旧 checker module を compiler pass として公開し続け、`nepl-core/tests/check_pipeline.rs` も旧 checker を直接呼んでいた。
- fallback 削除後に deep prefix chain を Resource IR gate だけで検証すると、user function return の raw-address alias を lowering が各 call に展開して owner gate の alias group を必要以上に膨張させていた。function return identity は initialized / owner / effect summary gate の責務であり、lowering で二重 materialize する設計は Resource IR authority を複雑化させる。

## 問題

Stage 4 still keeps passes::move_check::run as a post-Resource fallback after Resource IR lowering/cell/borrow/effect/owner gates. This preserves a second static-check authority and encourages self-host to copy legacy HIR traversal.

## 影響

Resource IR cannot be the final static-check authority while an old HIR checker remains in the compiler pipeline. The old checker also keeps raw alias and borrow logic alive outside the enum-first Resource IR model.

## 修正方針

Remove the compiler pipeline fallback to passes::move_check::run, remove the legacy move_check module from compiled passes, update the source policy to require Resource IR-only gates, and update tests to exercise Resource IR gates instead of the removed HIR checker.

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core --test check_pipeline resource_static_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture; node nodesrc/test_resource_gate_order.js; trunk build; node nodesrc/tests.js -i tests/compiler/move_check.n.md --runner wasm --no-tree -j 1; node nodesrc/issues.js check

## 対応結果

- compiler pipeline の resource gate 関数を `run_resource_static_check` に改名し、旧 `passes::move_check::run` fallback を削除した。
- `nepl-core/src/passes/move_check.rs` と配下 module を削除し、compiled pass として旧 HIR checker が残らないようにした。
- deep prefix chain の direct old checker test は Resource IR lowering / cell / borrow / effect / owner gate を直接確認する test に置き換えた。
- user function return の raw-address alias materialization は削除し、plain user call の identity / owner transfer は Resource IR summary gate で扱う設計へ戻した。core mem wrapper と named raw helper の explicit raw semantics は引き続き lowering で表す。
- `coverage_hir_projection` と `lower_aggregate` は helper module へ分割し、Resource IR source policy が責務肥大を検出できる状態を保った。
- `nodesrc/test_resource_gate_order.js` は Resource IR gate が揃っていることと、compiler が legacy fallback を呼ばないことを監視する policy に更新した。

## 検証結果

- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_rejects -- --nocapture`: 7 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check -- --nocapture`: 16 passed
- `cargo test -p nepl-core --test check_pipeline resource_static_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: passed
- `cargo test -p nepl-core --test check_pipeline prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: passed
- `node nodesrc/test_resource_gate_order.js`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -o output/move_check_after_legacy_fallback_removal.json --runner wasm --no-tree -j 1`: total=52, passed=52

## 追加で分離した残件

- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md -o output/move_effect_after_legacy_fallback_removal.json --runner wasm --no-tree -j 1` は total=110, passed=105, failed=5。
- 失敗は raw address helper の literal offset false positive と higher-order / aggregate / enum payload function value raw write false negative であり、旧 fallback 削除ではなく Resource IR effect/cell summary 側の残件であるため `ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2` を再オープンして追跡する。
