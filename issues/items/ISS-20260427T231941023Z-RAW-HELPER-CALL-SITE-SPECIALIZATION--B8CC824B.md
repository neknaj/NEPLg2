---
id: ISS-20260427T231941023Z-RAW-HELPER-CALL-SITE-SPECIALIZATION--B8CC824B
title: "raw helper call-site specialization clones deep HIR during summary build"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/check_pipeline.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T231941023Z-RAW-HELPER-CALL-SITE-SPECIALIZATION--B8CC824B: raw helper call-site specialization clones deep HIR during summary build

## 概要

The raw helper call-site specialization added function-body lookup to `MoveCheckContext`, but stored cloned `HirFunction` values. Building raw alias summaries then cloned every function body for each context. A valid deep prefix chain in `main` could therefore overflow the native stack while summarizing an unrelated small helper such as `inc`.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/check_pipeline.rs, tests/compiler/move_effect.n.md`

## 根拠

- `cargo test -p nepl-core --test check_pipeline move_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture` failed with native stack overflow on the default test stack.
- Temporary isolation showed the crash happened during `build_function_raw_alias_summaries` before visiting `main`; summarizing `inc` cloned the whole module through `MoveCheckContext::new`.
- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture` still had to pass, because the previous raw helper constant-offset precision fix must remain intact.

## 問題

`MoveCheckContext::new` built `function_defs: Rc<BTreeMap<String, HirFunction>>` by cloning each monomorphized function. `HirFunction` owns its body, so a deep expression tree is recursively cloned even when only a different function is being summarized. In addition, the iterative simple-call summary path attempted to specialize a callee with no existing summary; during fixed-point construction this can re-enter argument alias extraction before the summary table has stabilized.

## 影響

A valid deep expression can crash the compiler in move-check preparation. This blocks `prepare_module_for_codegen` and `compile_wasm` even though the source program is type-correct and does not use raw memory.

## 修正方針

- Store `function_defs` as references into the already-owned `HirModule` instead of cloning `HirFunction` bodies into each context.
- Keep `Rc` cloning shallow for alias-summary contexts.
- During simple-call summary fixed-point construction, do not specialize functions whose base summary is not available yet; use the next fixed-point iteration to refine them.

## 検証

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test check_pipeline -- --nocapture`: 7/7 passed
- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 51/51 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-helper-specialization-deep-prefix.json -j 1`: 94/94 passed
