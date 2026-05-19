---
id: ISS-20260519T154609873Z-RESOURCE-RAW-IDENTITY-SUMMARY-BLOCKS-E4C8EDF4
title: "Resource raw identity summary blocks Result owner token payload provenance"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/effect_return_summary_filter.rs; nepl-core/src/resource/effect_checked_memptr.rs; nepl-core/tests/resource_ir.rs"
---

# ISS-20260519T154609873Z-RESOURCE-RAW-IDENTITY-SUMMARY-BLOCKS-E4C8EDF4: Resource raw identity summary blocks Result owner token payload provenance

## 概要

Result<RegionToken<T>, E> return summaries are treated as structural owner carriers at the enum root, so allocator/region raw identity provenance is not propagated into Result::Ok payloads. checked MemPtr wrapper calls then report resource.raw.memory_outside_boundary even when the pointer is derived from alloc_region/region_ptr.

## 対象

- `nepl-core/src/resource/effect_return_summary_filter.rs; nepl-core/src/resource/effect_checked_memptr.rs; nepl-core/tests/resource_ir.rs`

## 根拠

- `effect_return_summary_filter` の unit test `summary_filter_keeps_owner_token_payload_internal_provenance` が失敗し、`Result<RegionToken, str>` が enum root の時点で structural owner carrier として遮断されていた。
- focused Resource IR regressions `compile_accepts_checked_region_pointer_from_region_provenance` / `compile_accepts_checked_region_ptr_at_from_region_provenance` が `resource.raw.memory_outside_boundary` で失敗し、checked MemPtr proof の raw identity operation が空になっていた。
- pointer alias table には `RegionToken.raw` と `MemPtr.raw` の alias group が残っていたため、型付き enum payload summary と alias group を組み合わせれば stdlib 名の allowlist なしで証明できる。

## 問題

Result<RegionToken<T>, E> return summaries are treated as structural owner carriers at the enum root, so allocator/region raw identity provenance is not propagated into Result::Ok payloads. checked MemPtr wrapper calls then report resource.raw.memory_outside_boundary even when the pointer is derived from alloc_region/region_ptr.

## 影響

Safe RegionToken-derived MemPtr store/load/fill paths fail static checking, which pressures callers toward raw helper bypasses and blocks the Stage 6 owner-span proof work.

## 修正方針

Keep struct/tuple owner-carrier summary suppression, but allow enum payload provenance to be summarized through typed enum variants. Make checked MemPtr proof consult pointer alias groups so wrapper arguments can follow returned/projection aliases without module allowlists.

## 検証

Run effect_return_summary_filter unit tests and focused Resource IR checked MemPtr provenance regressions.

## 解決内容

`raw_identity_projection_has_summary_owner_carrier_protection` を、struct/tuple などの structural owner carrier root は遮断するが、enum root は variant payload を辿れるように変更した。これにより `Result::Ok(RegionToken<T>)` のような fallible owner return は、enum variant によって gate された payload provenance として raw identity summary を持てる。一方で `StringBuilder` / `ByteBuilder` のような owner carrier aggregate root は引き続き summary carrier にならない。

checked MemPtr proof は `MemPtr.raw` の exact place だけでなく pointer alias group も確認するようにした。`region_ptr` / `region_ptr_at` / Result payload bind / local read で生じる projection alias を追跡するためで、特定 stdlib module 名や関数名の allowlist ではない。

Resource IR unit の null sentinel regression は、user source から `core/mem/internal::mem_ptr_wrap` を呼べるという古い前提をやめ、compiler-owned boundary source として実行するように修正した。通常 source から internal helper を使う経路は引き続き source capability gate で拒否される。

## 検証結果

- `cargo test -p nepl-core --lib effect_return_summary_filter -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_mem_ptr_wrapper_with_null_sentinel -- --exact --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_region_pointer_from_region_provenance -- --exact --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_region_ptr_at_from_region_provenance -- --exact --nocapture`: pass。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
