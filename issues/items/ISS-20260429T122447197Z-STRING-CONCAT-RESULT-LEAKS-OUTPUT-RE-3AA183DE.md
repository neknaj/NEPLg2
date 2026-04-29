---
id: ISS-20260429T122447197Z-STRING-CONCAT-RESULT-LEAKS-OUTPUT-RE-3AA183DE
title: "String concat_result leaks output region owner under Resource IR"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/lower_raw_address.rs, stdlib/alloc/string.nepl"
---

# ISS-20260429T122447197Z-STRING-CONCAT-RESULT-LEAKS-OUTPUT-RE-3AA183DE: String concat_result leaks output region owner under Resource IR

## 概要

Strict Resource IR owner checking reported concat_result__str_str__Result_T_E_str_str__pure leaking out_region's raw storage owner. The root cause was split between Resource IR not treating `str_from_addr_unchecked` as the inverse raw-address view of `str_addr`, and stdlib string constructors bypassing the `RegionToken`-consuming `string_finish` ownership boundary.

## 対象

- `nepl-core/src/resource/lower_raw_address.rs`
- `stdlib/alloc/string.nepl`

## 根拠

- `cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture` は `concat_result__str_str__Result_T_E_str_str__pure` の `Local("out_region").Field(0).Field(0)` owner leak で失敗する。
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` も同じ `concat_result` owner leak を報告し、その後 `hashmap_rehash_to...` の backing entries owner leak も報告する。
- `stdlib/alloc/string.nepl` の `concat_result` は `string_alloc_region total` の `Result::Ok out_region` から `out_base` / `out_data` を取り出し、`string_finish_base out_base total` を `Result::Ok` に包んで返す。Resource IR では `out_region` の region owner が戻り値 `str` に移ったことを証明できず、`out_region.ptr.ptr` 相当の owner が残る。
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check。

## 問題

Strict Resource IR owner checking reported concat_result__str_str__Result_T_E_str_str__pure leaking out_region's raw storage owner. List/HashMap tests failed earlier on this string helper before the collection-specific owner contract was reached.

## 影響

Any test path that formats diagnostics or strings through concat_result can fail the memory-safety gate, blocking collection and self-host validation. The issue must be fixed in stdlib ownership contracts rather than weakening Resource IR owner checking.

## 修正方針

Review concat_result's allocation, RegionToken/MemPtr ownership transfer, Ok/Err paths, and failure cleanup. Ensure every path either returns the output string with the region owner transferred into the returned value, or frees/rolls back the allocated region before returning Err.

## 修正内容

- Resource IR の raw address lowering に `str_from_addr_unchecked(addr)` を追加し、戻り値 `str` が `addr` と同じ raw storage を指すことを `RawAddressAlias` として表現した。
- `string_finish` は `RegionToken<u8>` を値で消費して `get region "ptr"` から基底 `MemPtr` を取り出す形にし、`RegionToken` の owner leaf が返却 `str` へ移る境界を明確にした。
- `string_from_mem_unchecked_result` / `concat_result` / `sb_build_result` / `from_u128_radix` は `string_finish_base` を直接呼ばず、確保した `RegionToken` を `string_finish` へ渡す形に統一した。
- `string_finish_base` は raw address を `str` に変換する内部 helper として残し、allocator からの公開経路は `RegionToken` を消費する `string_finish` に集約した。
- Resource IR owner 回帰として、`str_from_addr_unchecked` が raw owner を `str` へ移す経路と、`concat_result` が output region owner を `Result::Ok str` へ移す経路を追加した。

## 検証

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 28 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_ -- --nocapture`: 19 passed
- `cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: `concat_result` owner leak は解消済み。次の既知問題として `hashmap_rehash_to...` の backing entries owner leak に進むため、`ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB` で継続する。
