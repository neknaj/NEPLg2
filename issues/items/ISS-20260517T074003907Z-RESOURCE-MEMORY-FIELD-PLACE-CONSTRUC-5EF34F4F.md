---
id: ISS-20260517T074003907Z-RESOURCE-MEMORY-FIELD-PLACE-CONSTRUC-5EF34F4F
title: "Resource memory field place construction duplicates compiler memory layout"
area: compiler/resource-ir
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/place_utils.rs; nepl-core/src/resource/lower_raw_address_place.rs; nepl-core/src/resource/initialized_summary_indirect_release.rs"
---

# ISS-20260517T074003907Z-RESOURCE-MEMORY-FIELD-PLACE-CONSTRUC-5EF34F4F: Resource memory field place construction duplicates compiler memory layout

## 概要

Resource IR の MemPtr.raw / RegionToken.raw / RegionToken.size place 構築が index 0/1 と offset 0/4 を直接埋め込み、CompilerMemoryFieldSpec と aggregate layout proof から導出されていない。

## 対象

- `nepl-core/src/resource/place_utils.rs; nepl-core/src/resource/lower_raw_address_place.rs; nepl-core/src/resource/initialized_summary_indirect_release.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6 では、compiler memory type identity と field shape を source proof / typecheck registration / Resource IR semantic proof へ接続する方針を取っている。
- 直前の `CompilerMemoryFieldSpec` 集約後も、Resource IR の place 構築には `MemPtr.raw` / `RegionToken.raw` / `RegionToken.size` の index/offset 直書きと重複 helper が残っていた。

## 問題

Resource IR の MemPtr.raw / RegionToken.raw / RegionToken.size place 構築が index 0/1 と offset 0/4 を直接埋め込み、CompilerMemoryFieldSpec と aggregate layout proof から導出されていない。

## 影響

compiler memory type の field contract を shared enum に集約しても、Resource IR の place 構築だけが古い layout 仮定を持ち続け、型定義 shape proof と実際の owner/raw cell tracking が drift する危険がある。

## 修正方針

compiler memory field place helper を TypeCtx identity、CompilerMemoryType、CompilerMemoryFieldSpec、aggregate_fields_with_offsets に接続し、MemPtr / RegionToken の raw/size projection を shared helper から生成する。重複 helper と direct index/offset を削除し、責務検査で再導入を禁止する。

## 対応内容

- `compiler_memory_place.rs` を追加し、`MemPtr.raw` / `RegionToken.raw` / `RegionToken.size` の place projection を TypeCtx の compiler memory identity、`CompilerMemoryFieldSpec`、`aggregate_fields_with_offsets` から導出するようにした。
- `lower_raw_address_place.rs` と `initialized_summary_indirect_release.rs` にあった direct `PlaceProjection::Field { index: 0, offset_bytes: 0 }` helper を削除し、Resource IR の lowering / initialized summary / owner extent comparison が shared helper を使うようにした。
- `compiler_memory_type_field_offset_bytes` を shared field spec 側へ追加し、`RegionToken.raw` から `RegionToken.size` sibling へ移る処理も field spec から導出するようにした。
- 責務検査に `compiler_memory_place.rs` / tests を登録し、field index / offset / TypeCtx identity proof を使うことと、lowering 側の direct projection 再導入禁止を検査する。
- 関連計画: [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core compiler_memory_field_place --lib -- --nocapture`
- `cargo test -p nepl-core region_token_size_sibling --lib -- --nocapture`
- `cargo test -p nepl-core compiler_memory_type_field_specs_are_kind_owned --lib -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
