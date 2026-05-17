---
id: ISS-20260517T082244329Z-RESOURCE-IR-REGION-PTR-AT-LOWERING-H-F552468A
title: "Resource IR region_ptr_at lowering hardcodes Result Ok payload spelling"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/lower_raw_address.rs; nepl-core/src/resource/result_variant.rs; nodesrc/test_resource_checker_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T082244329Z-RESOURCE-IR-REGION-PTR-AT-LOWERING-H-F552468A: Resource IR region_ptr_at lowering hardcodes Result Ok payload spelling

## 概要

Resource IR の `region_ptr_at` lowering が Result success payload を投影するために、`lower_raw_address.rs` 内で直接 `"Ok"` を `enum_payload_type` と `PlaceProjection::EnumPayload` に渡している。Result payload contract が個別 consumer に直書きされ、shared typed domain から外れている。

## 対象

- `nepl-core/src/resource/lower_raw_address.rs; nepl-core/src/resource/result_variant.rs; nodesrc/test_resource_checker_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `lower_raw_address.rs` の `MemoryHelperPrimitive::RegionPtrAt` branch は、修正前に `enum_payload_type(env.types, output.ty, "Ok")` で payload 型を引き、`PlaceProjection::EnumPayload { variant: String::from("Ok") }` を直接構築していた。
- これは `Result` success payload projection の spelling と projection construction が Resource IR lowering の個別 branch に閉じており、他 consumer と同じ型付き contract を共有していない状態だった。

## 問題

Resource IR の `region_ptr_at` lowering が Result success payload を投影するために、`lower_raw_address.rs` 内で直接 `"Ok"` を `enum_payload_type` と `PlaceProjection::EnumPayload` に渡している。Result payload contract が個別 consumer に直書きされ、shared typed domain から外れている。

## 影響

Result success payload projection の spelling が他の Resource IR consumer と drift しても Rust の match 網羅性や source policy で検出しにくい。静的検査の証明器を汎用・型付きに寄せる方針に反し、個別 lowering に enum variant spelling が散る。

## 修正方針

Result success payload の variant spelling と projection construction を typed enum domain にまとめ、`region_ptr_at` lowering はその enum を消費する。`lower_raw_address.rs` から direct `"Ok"` projection を削除し、nodesrc policy で再導入を禁止する。

## 検証

cargo fmt -p nepl-core --check; cargo check -p nepl-core; focused Resource IR/unit tests; node nodesrc/test_resource_checker_responsibility.js; node nodesrc/issues.js check --dir issues; git diff --check

## 対応

2026-05-17 に修正した。`resource/result_variant.rs` を追加し、`ResultVariant::Ok` が success variant spelling と `PlaceProjection::EnumPayload` construction を所有するようにした。`lower_raw_address.rs` の `RegionPtrAt` lowering は `ResultVariant::Ok.payload_place` を消費し、Result success payload の raw address propagation を typed enum domain 経由にした。

`nodesrc/test_resource_checker_responsibility.js` には `result_variant.rs` の存在、`ResultVariant` enum、`payload_place`、および `lower_raw_address.rs` に direct `"Ok"` projection が戻らないことを監視する policy を追加した。

## 回帰テスト

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core result_variant --lib -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed with CRLF normalization warnings only
