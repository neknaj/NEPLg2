---
id: ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B
title: "Resource IR gate regresses neplg2 generic aggregate and collection integration tests"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, nepl-core/tests/neplg2.rs, stdlib/alloc/collections"
---

# ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B: Resource IR gate regresses neplg2 generic aggregate and collection integration tests

## 概要

cargo test -p nepl-core --test neplg2 -- --nocapture currently passes 52 tests and fails 8 tests. The failures are not caused by assignment diagnostic construction; they report resource.raw.ownership_violation as owner obligation leaks for generic aggregate store/load tests and RawMemoryLoadCell Uninit for List/HashMap collection paths.

## 対象

- `nepl-core/src/resource, nepl-core/tests/neplg2.rs, stdlib/alloc/collections`

## 根拠

- `cargo test -p nepl-core --test neplg2 -- --nocapture` は 60 件中 52 passed / 8 failed。
- generic aggregate 系は `generic_intrinsic_store_load_struct_preserves_fields`、`generic_store_uses_nested_address_call_without_stealing_value_arg`、`generic_hashkey_value_survives_hash_before_store`、`generic_hashkey_eq_after_load_uses_concrete_impl`、`generic_store_after_generic_trait_probe_preserves_struct` が、`Place { root: Local("p"), ... } still owns StorageId(0)` の owner obligation leak で失敗した。
- raw-memory-backed collection 系は `list_get_out_of_bounds_err`、`hashmap_custom_struct_key_roundtrips_value`、`llvm_hashmap_string_key_preserves_explicit_hasher_type_args` が、`List` / `HashMap` header や slot の `RawMemoryLoadCell ... found Uninit` で失敗した。
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md -i tests/compiler/neplg2.n.md --no-tree -o tmp/agent1-assignment-diagnostics-after-trunk.json -j 1` も 207 件中 206 passed / 1 failed で、失敗は `tests/compiler/neplg2.n.md::doctest#33` の `get__List_T_T_i32__Option_T_T__pure_i32` RawMemoryLoadCell Uninit だった。
- 同じ checkout で assignment diagnostic の focused regression `cargo test -p nepl-core --test neplg2 set_type_mismatch_is_error -- --nocapture` と `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 3 --dist web/dist` は pass しているため、assignment diagnostic code-first 化とは別の Resource IR 問題として扱う。

## 問題

cargo test -p nepl-core --test neplg2 -- --nocapture currently passes 52 tests and fails 8 tests. The failures are not caused by assignment diagnostic construction; they report resource.raw.ownership_violation as owner obligation leaks for generic aggregate store/load tests and RawMemoryLoadCell Uninit for List/HashMap collection paths.

## 影響

The full Rust neplg2 integration suite is no longer a clean regression gate. Generic aggregate storage and raw-memory-backed collections are directly relevant to self-host data structures, so this must be tracked without weakening RawMemoryLoadCell or owner obligation diagnostics.

## 修正方針

Trace Resource IR owner and cell-state propagation for generic aggregate parameters, raw collection headers, and slot loads. Preserve the strict diagnostics, but separate owned value obligations from initialized raw backing storage so valid generic store/load, List get, and HashMap roundtrip tests prove initialized cells instead of tripping D3100.

## 検証

cargo test -p nepl-core --test neplg2 generic_intrinsic_store_load_struct_preserves_fields generic_store_uses_nested_address_call_without_stealing_value_arg generic_hashkey_value_survives_hash_before_store generic_hashkey_eq_after_load_uses_concrete_impl generic_store_after_generic_trait_probe_preserves_struct list_get_out_of_bounds_err hashmap_custom_struct_key_roundtrips_value llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture; then cargo test -p nepl-core --test neplg2 -- --nocapture

## 2026-04-29 generic aggregate subcase 追記

現行 main で generic aggregate 系を再確認したところ、`roundtrip`、`same_after_store`、`hash_then_store`、`write_after_probe`、`write_nested` の test helper は `alloc_raw` で確保した一時 storage へ `store<T>` / `load<T>` した後、storage を `dealloc_raw` していなかった。

これは Resource IR owner checker の誤検出ではなく、Stage 4 の free obligation model が正しく検出した storage owner leak である。検査を弱めるのではなく、test helper 側で `load<T>` の戻り値を local に受け、raw storage を `dealloc_raw` してから戻り値を返すように修正した。

検証:

- `cargo test -p nepl-core --test neplg2 generic_intrinsic_store_load_struct_preserves_fields -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_hashkey_eq_after_load_uses_concrete_impl -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_hashkey_value_survives_hash_before_store -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_ -- --nocapture`: 8 passed

collection 系の `List` / `HashMap` `RawMemoryLoadCell ... found Uninit` はこの test helper leak とは別の stdlib raw-memory-backed collection / Resource IR lowering 問題として残る。stdlib 側修正は別作業方針のため、この commit では generic aggregate subcase のみを整理する。

## 2026-04-29 Resource IR projection/raw owner subcase 追記

関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check。

generic aggregate subcase の後に残っていた Resource IR 側の誤検出を再調査し、所有者 summary と raw alias が root だけを前提にしていたことを確認した。

修正した根本原因:

- owner return summary を `TypeCtx` ベースにし、struct / tuple / enum / Apply の所有者 leaf projection を構造的に列挙するようにした。
- `Result::Ok(Boxed).ptr` のような parameter projection 由来の返却と、返却されず消費された projection を区別した。これにより aggregate の一部だけを消費した関数が caller 側で aggregate 全体を消費する誤りを防ぐ。
- aggregate construct の Resource IR が struct / tuple field offset を 0 固定で作っていたため、field access 側の正しい offset と所有者 place が一致していなかった。`AggregateKind` に field offset を持たせ、construct / alias / owner / initialized / effect 系で同じ projection を使うようにした。
- match bind は enum payload の親 place を move するが、raw alias は payload 内 field leaf だけを持つことがある。leaf alias から親 prefix alias を復元して、payload 全体の move が field owner を正しく移動できるようにした。
- `alloc_raw` の zero / non-zero 分岐について、`eq/ne/lt/le/gt/ge` から `EqZero` / `NeZero` / `Positive` / `NonPositive` の enum fact を Lowering で作り、到達不能な owner obligation を分岐ごとに落とすようにした。
- raw memory `store` / `load` で owner を raw cell へ移動し、cell から戻すようにした。collection node に格納した tail owner を後で load/free できる focused regression を追加した。

検証:

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 26 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_ -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test neplg2 generic_ -- --nocapture`: 8 passed

残件:

- `list_get_out_of_bounds_err` と HashMap 系の残りは、Resource IR が raw cell owner を追跡できるようになったことで表面化した stdlib の fallible owning collection contract 問題として `ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB` に分離した。
