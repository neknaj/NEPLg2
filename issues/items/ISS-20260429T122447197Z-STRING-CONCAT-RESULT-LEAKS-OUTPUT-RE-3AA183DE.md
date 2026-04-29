---
id: ISS-20260429T122447197Z-STRING-CONCAT-RESULT-LEAKS-OUTPUT-RE-3AA183DE
title: "String concat_result leaks output region owner under Resource IR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, stdlib/core/mem"
---

# ISS-20260429T122447197Z-STRING-CONCAT-RESULT-LEAKS-OUTPUT-RE-3AA183DE: String concat_result leaks output region owner under Resource IR

## 概要

Strict Resource IR owner checking reports concat_result__str_str__Result_T_E_str_str__pure leaking out_region's raw storage owner. List/HashMap tests now fail earlier on this string helper before the collection-specific owner contract is reached.

## 対象

- `stdlib/alloc/string.nepl, stdlib/core/mem`

## 根拠

- `cargo test -p nepl-core --test neplg2 list_get_out_of_bounds_err -- --nocapture` は `concat_result__str_str__Result_T_E_str_str__pure` の `Local("out_region").Field(0).Field(0)` owner leak で失敗する。
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` も同じ `concat_result` owner leak を報告し、その後 `hashmap_rehash_to...` の backing entries owner leak も報告する。
- `stdlib/alloc/string.nepl` の `concat_result` は `string_alloc_region total` の `Result::Ok out_region` から `out_base` / `out_data` を取り出し、`string_finish_base out_base total` を `Result::Ok` に包んで返す。Resource IR では `out_region` の region owner が戻り値 `str` に移ったことを証明できず、`out_region.ptr.ptr` 相当の owner が残る。
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check。

## 問題

Strict Resource IR owner checking reports concat_result__str_str__Result_T_E_str_str__pure leaking out_region's raw storage owner. List/HashMap tests now fail earlier on this string helper before the collection-specific owner contract is reached.

## 影響

Any test path that formats diagnostics or strings through concat_result can fail the memory-safety gate, blocking collection and self-host validation. The issue must be fixed in stdlib ownership contracts rather than weakening Resource IR owner checking.

## 修正方針

Review concat_result's allocation, RegionToken/MemPtr ownership transfer, Ok/Err paths, and failure cleanup. Ensure every path either returns the output string with the region owner transferred into the returned value, or frees/rolls back the allocated region before returning Err.

## 検証

Run focused concat_result/string tests plus list_get_out_of_bounds_err and hashmap_custom_struct_key_roundtrips_value after the stdlib ownership contract is fixed.
