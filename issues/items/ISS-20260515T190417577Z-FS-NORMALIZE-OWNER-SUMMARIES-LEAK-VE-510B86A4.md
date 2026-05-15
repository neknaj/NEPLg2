---
id: ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4
title: "FS normalize owner summaries leak Vec and StringBuilder owner payloads"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/resource; stdlib/std/fs/path/normalize/*.nepl; stdlib/std/fs/stat.nepl"
---

# ISS-20260515T190417577Z-FS-NORMALIZE-OWNER-SUMMARIES-LEAK-VE-510B86A4: FS normalize owner summaries leak Vec and StringBuilder owner payloads

## 概要

After the Result<Vec<T>, E> raw identity false positive is removed, std/fs/stat.nepl doctest reaches resource.owner.maybe_leak in fs_normalize_range_push, fs_normalize_build_ranges_builder, fs_normalize_relative_builder, and fs_path_filetype. The leaked places are nested RegionToken/raw owner projections under returned Vec or StringBuilder owner payloads.

## 対象

- `nepl-core/src/resource; stdlib/std/fs/path/normalize/*.nepl; stdlib/std/fs/stat.nepl`

## 根拠

- [ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD](./ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD.md) の raw identity false positive を修正した後、`node nodesrc/tests.js -i stdlib/std/fs/stat.nepl --no-tree -o tmp/agent1-fs-stat-after-raw-identity-rerun.json -j 1 --dist web/dist --assert-io` は `resource.raw.identity_escape` ではなく `resource.owner.maybe_leak` で停止するようになった。
- `fs_normalize_range_push__Vec_T_i32_i32_i32__Result_T_E_Vec_T_i32_i32__pure` は `/stdlib/std/fs/path/normalize/range_stack.nepl:30:8` で `Temporary(ResourceId(20)).field3.field0` の owner obligation leak と診断される。
- `fs_normalize_build_ranges_builder__str_ref_Vec_T_i32__Result_T_E_StringBuilder_i32__pure` は `/stdlib/std/fs/path/normalize/build.nepl:33:8` で nested `StringBuilder` / byte owner projection leak と診断される。
- `fs_normalize_relative_builder__str__Result_T_E_StringBuilder_i32__pure` と `fs_path_filetype__str__Result_T_E_i32_i32__imp` でも同系統の nested owner projection leak が出る。
- これは stdlib 関数名の許可リストで回避すべきではない。`Vec` / `StringBuilder` / `Result::Ok` の source-level owner transfer を Resource IR が証明できる必要がある。

## 問題

After the Result<Vec<T>, E> raw identity false positive is removed, std/fs/stat.nepl doctest reaches resource.owner.maybe_leak in fs_normalize_range_push, fs_normalize_build_ranges_builder, fs_normalize_relative_builder, and fs_path_filetype. The leaked places are nested RegionToken/raw owner projections under returned Vec or StringBuilder owner payloads.

## 影響

Filesystem path normalization remains blocked after the raw identity fix. Weakening resource.owner.maybe_leak would hide real leaks, so the compiler must prove owner transfer through Result<Vec<T>, E> and Result<StringBuilder, E> from source-level ownership rather than whitelist stdlib functions.

## 修正方針

Review Resource IR owner summary generation and return-boundary owner transfer for enum payloads carrying structural owners. Preserve exact projections for RegionToken/raw owner fields through Vec/StringBuilder construction, push/build helper calls, and Result::Ok wrapping, while keeping public raw i32 and MemPtr owner leaks rejected.

## 検証

Run focused Resource IR owner regression for fs_normalize_range_push and std/fs/stat.nepl doctest after fixing. Current evidence file: tmp/agent1-fs-stat-after-raw-identity-rerun.json.
