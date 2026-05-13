---
id: ISS-20260513T160802076Z-GENERIC-RAW-MEMORY-REGRESSION-FIXTUR-1F871A8E
title: "generic raw-memory regression fixtures lack explicit boundary capability"
area: core
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/tests/neplg2.rs
---

# ISS-20260513T160802076Z-GENERIC-RAW-MEMORY-REGRESSION-FIXTUR-1F871A8E: generic raw-memory regression fixtures lack explicit boundary capability

## 概要

Generic aggregate raw load/store regression tests import core/mem/raw and exercise alloc_raw/store/load/dealloc_raw from ordinary inline test source. After source-based raw-memory-boundary proof was enforced, those tests correctly fail with Resource(Raw(MemoryOutsideBoundary)) before reaching the generic trait/codegen behavior they are intended to guard.

## 対象

- `nepl-core/tests/neplg2.rs`

## 根拠

- `cargo test -p nepl-core --test neplg2 generic_intrinsic_store_load_struct_preserves_fields -- --nocapture` が `Resource(Raw(MemoryOutsideBoundary))` で失敗した。
- 失敗箇所は `alloc_raw` / `store<.T>` / `load<.T>` / `dealloc_raw` で、test source は通常の `run_main_wasi_i32` 経路から読み込まれていた。
- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5/6 では、raw memory 操作は stdlib allowlist ではなく source capability proof で閉じる方針になっているため、テストのために compiler gate を緩めるのは誤りである。

## 問題

Generic aggregate raw load/store regression tests import core/mem/raw and exercise alloc_raw/store/load/dealloc_raw from ordinary inline test source. After source-based raw-memory-boundary proof was enforced, those tests correctly fail with Resource(Raw(MemoryOutsideBoundary)) before reaching the generic trait/codegen behavior they are intended to guard.

## 影響

The generic abstraction regression suite reports failures that are caused by fixture authority rather than generic or trait semantics. Weakening the compiler gate would compromise memory safety, while leaving the fixtures unchanged hides real abstraction regressions behind boundary diagnostics.

## 修正方針

Keep production raw-memory-boundary enforcement strict. Add an explicit test-only harness entry that marks only the inline entry source as SourceCapabilities::raw_memory_boundary(), then route only the raw-memory generic fixtures through that helper and add a source-policy regression to prevent accidental weakening of the ordinary harness path.

## 対応

- `compile_src_with_options_and_entry_capabilities` を追加し、inline entry source だけに明示的な `SourceCapabilities` を与えられるようにした。
- `run_main_wasi_i32_raw_memory_boundary` を追加し、compiler-owned raw-memory-boundary を検査する fixture だけが使う実行経路を分離した。
- 通常の `run_main_wasi_i32` は `SourceCapabilities::none()` のままにし、production loader / compiler gate の振る舞いを変えていない。
- `generic_intrinsic_store_load_struct_preserves_fields`、`generic_hashkey_eq_after_load_uses_concrete_impl`、`generic_hashkey_value_survives_hash_before_store`、`generic_store_after_generic_trait_probe_preserves_struct`、`generic_store_uses_nested_address_call_without_stealing_value_arg` を明示 raw boundary helper に移した。
- source policy に、通常 harness が raw boundary capability を与えないことと、上記 fixture が明示 helper を使うことを追加した。
- raw boundary 修正後、残る2件は `Resource(Owner(Unavailable))` / `Resource(Owner(Leak))` に進んだため、別 issue `ISS-20260513T161115125Z-RESOURCE-OWNER-CHECKER-CANNOT-PROVE--0A41590B` として切り分けた。

## 検証

- `cargo fmt --package nepl-core --check`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: passed
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`: `8 passed; 2 failed`
  - raw-memory-boundary 欠落の5件はすべて `Resource(Raw(MemoryOutsideBoundary))` を脱した。
  - 残る2件は extent equality proof の別問題であり、`ISS-20260513T161115125Z-RESOURCE-OWNER-CHECKER-CANNOT-PROVE--0A41590B` で追跡する。
