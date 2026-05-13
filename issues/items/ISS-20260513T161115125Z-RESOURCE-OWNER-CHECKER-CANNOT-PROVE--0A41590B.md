---
id: ISS-20260513T161115125Z-RESOURCE-OWNER-CHECKER-CANNOT-PROVE--0A41590B
title: "resource owner checker cannot prove repeated generic allocation extents equal"
area: core
status: open
resolved: false
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

## 問題

After generic raw-memory fixtures run inside an explicit raw-memory boundary, tests that allocate with add size_of<T> size_of<V> and deallocate with the same expression still fail with Resource(Owner(Unavailable)) and Resource(Owner(Leak)). The owner checker records the allocation extent as a temporary place and requires the deallocation extent to reuse that exact place, so two syntactically identical pure generic size expressions are not recognized as the same allocation extent.

## 影響

Valid raw-memory-boundary compiler-owned code can be rejected unless authors manually store allocation sizes in a local. This is not a memory-safety hole, but it makes Resource IR proof depend on incidental temporary identity rather than a typed extent proof and can obscure generic/codegen regressions.

## 修正方針

Audit the Resource owner extent model and represent allocation/deallocation extents as typed, comparable extent expressions or require lowering to preserve a stable extent value through an explicit local. Do not weaken deallocation checks or accept unknown extents; the checker must prove equality structurally.

## 検証

Run the two focused generic raw extent tests, the full generic neplg2 filter, resource/static-check source policies, and issue index validation.
