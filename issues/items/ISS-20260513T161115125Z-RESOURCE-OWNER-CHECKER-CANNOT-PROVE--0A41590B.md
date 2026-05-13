---
id: ISS-20260513T161115125Z-RESOURCE-OWNER-CHECKER-CANNOT-PROVE--0A41590B
title: "resource owner checker cannot prove repeated generic allocation extents equal"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/tests/neplg2.rs
---

# ISS-20260513T161115125Z-RESOURCE-OWNER-CHECKER-CANNOT-PROVE--0A41590B: resource owner checker cannot prove repeated generic allocation extents equal

## 概要

After generic raw-memory fixtures run inside an explicit raw-memory boundary, tests that allocate with add size_of<T> size_of<V> and deallocate with the same expression still fail with Resource(Owner(Unavailable)) and Resource(Owner(Leak)). The owner checker records the allocation extent as a temporary place and requires the deallocation extent to reuse that exact place, so two syntactically identical pure generic size expressions are not recognized as the same allocation extent.

## 対象

- `nepl-core/tests/neplg2.rs`

## 根拠

- `ISS-20260513T160802076Z-GENERIC-RAW-MEMORY-REGRESSION-FIXTUR-1F871A8E` の修正で generic raw-memory fixture を明示 raw boundary として実行した後、raw boundary diagnostic は消えた。
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture` は `generic_store_after_generic_trait_probe_preserves_struct` と `generic_store_uses_nested_address_call_without_stealing_value_arg` で `Resource(Owner(Unavailable))` / `Resource(Owner(Leak))` に進んだ。
- どちらも allocation extent と deallocation extent に `add size_of<.T> size_of<.V>` を使っているが、owner checker は allocation 時の temporary place と deallocation 時の temporary place を別物として扱っている。
- raw memory 操作自体は明示 boundary 内でのみ許可されているため、これは boundary 権限の問題ではなく Resource owner extent proof の問題である。
- Resource IR dump では `size_of<Point>` / `size_of<i32>` は `LiteralI32(8)` / `LiteralI32(4)` へ下がっていたが、`add` call に付随する `raw_address_view offset` が source を既知 raw address と証明できない場合に `raw_aliases.clear(target)` を呼び、直前に得た scalar fact まで消していた。

## 問題

After generic raw-memory fixtures run inside an explicit raw-memory boundary, tests that allocate with add size_of<T> size_of<V> and deallocate with the same expression still fail with Resource(Owner(Unavailable)) and Resource(Owner(Leak)). The owner checker records the allocation extent as a temporary place and requires the deallocation extent to reuse that exact place, so two syntactically identical pure generic size expressions are not recognized as the same allocation extent.

## 影響

Valid raw-memory-boundary compiler-owned code can be rejected unless authors manually store allocation sizes in a local. This is not a memory-safety hole, but it makes Resource IR proof depend on incidental temporary identity rather than a typed extent proof and can obscure generic/codegen regressions.

## 修正方針

Audit the Resource owner extent model and represent allocation/deallocation extents as typed, comparable extent expressions or require lowering to preserve a stable extent value through an explicit local. Do not weaken deallocation checks or accept unknown extents; the checker must prove equality structurally.

## 対応

- `record_direct_call_i32_facts` で `add` / `sub` / `mul` の既知 i32 定数結果を scalar fact として記録するようにした。
- `RawCellAddressAliases::clear_raw_address_facts` を追加し、raw-address metadata と scalar metadata の削除責務を分離した。
- `apply_raw_address_view` は source が既知 raw address でない場合、raw-address facts だけを消し、同じ output に直前の call analysis が記録した scalar facts を保持するようにした。
- これにより `add size_of<Point> size_of<i32>` の結果が 12 bytes と証明され、allocation extent と deallocation extent が同じ payload byte count と判定される。
- deallocation check そのものは緩めておらず、unknown extent や mismatch を許可する変更は入れていない。

## 検証

- `cargo fmt --package nepl-core --check`: passed
- `cargo test -p nepl-core i32_call_facts -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 generic_store -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`: `10 passed`
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: passed
