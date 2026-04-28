---
id: ISS-20260428T224138753Z-STRING-CONSTRUCTORS-REUSE-REGIONTOKE-91ED01B9
title: "String constructors reuse RegionToken storage after RawMemoryLoadCell move"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, nodesrc/test_stdlib_string_no_unsafe_unwraps.js"
---

# ISS-20260428T224138753Z-STRING-CONSTRUCTORS-REUSE-REGIONTOKE-91ED01B9: String constructors reuse RegionToken storage after RawMemoryLoadCell move

## 概要

After the external raw root fix, string parameter reads are improved, but owned string construction still fails: concat_result and from_u128_radix read RegionToken-derived output storage after prior pointer extraction, and RawMemoryLoadCell reports out_region as Moved or scratch_raw as MaybeMoved.

## 対象

- `stdlib/alloc/string.nepl, nepl-core/src/resource`

## 根拠

- `trunk build` 後の `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\vec-header-ref-reads-after-trunk-type-arena.json -j 1` は `total=5, failed=5` で、全 doctest の top issue が `concat_result__str_str__Result_T_E_str_str__pure` の D3100 だった。
- 具体的には `/stdlib/alloc/string.nepl:552` の `let out_base <MemPtr<u8>> get out_region "ptr"` が `RawMemoryLoadCell ... Local("out_region") ... found Moved` になる。
- `stdlib\alloc\collections\vec.nepl` focused doctest でも、string-heavy helper 経由で同じ `concat_result` D3100 が先に出る。
- 同じログには `/stdlib/alloc/string.nepl:2427` の `from_u128_radix` `out_region` Moved と、`/stdlib/alloc/string.nepl:2434` の `scratch_raw` MaybeMoved も含まれる。
- `ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2` は `str_addr` parameter backing storage の false Uninit を直したが、owned `RegionToken` / scratch allocation の move state はまだ別問題として残っている。

## 問題

After the external raw root fix, string parameter reads are improved, but owned string construction still fails: concat_result and from_u128_radix read RegionToken-derived output storage after prior pointer extraction, and RawMemoryLoadCell reports out_region as Moved or scratch_raw as MaybeMoved.

## 影響

Self-host TypeArena fixture still fails 5/5 at concat_result, and Vec doctests that import std/test/string-heavy helpers still see string construction D3100 before exercising their own code. This keeps string-heavy parser/resolver/diagnostic work from becoming a reliable regression gate.

## 修正方針

String 側では `RegionToken` の `ptr` projection を owned `get` ではなく `get_ref` で読む。`from_u128_radix` は scratch raw buffer を使わず、桁数を数えてから出力領域へ末尾から直接書き込む。RawMemoryLoadCell の判定は弱めない。

## 修正内容

- `string_region_len_ptr` / `string_region_data_ptr` を `&RegionToken<u8>` から投影する helper に変更し、`RegionToken` owner を pointer projection だけで move しない形にした。
- `string_finish` / `string_from_mem_unchecked_result` / `concat_result` / `sb_build_result` / `from_u128_radix` の output region pointer reads を `get_ref` に統一した。
- `byte_at` は範囲確認後に `string_byte_at_unchecked` を使い、`MemPtr` temporary deref の RawMemoryLoadCell `Uninit` を作らないようにした。
- `from_u128_radix` から `scratch_raw` buffer を削除し、`digit_count` を先に数えて `string_alloc_region digit_count` 後に出力領域へ後ろから直接 digit を書く方式に変更した。
- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` に、`RegionToken` ptr の direct `get` 再導入、by-value `string_region_data_ptr` 呼び出し、`from_u128_radix` の scratch raw storage 再導入を防ぐ regression を追加した。

## 検証

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib\alloc\string.nepl --no-tree -o tmp\string-regiontoken-ref-string-2.json -j 1`: total=6, passed=6
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\string-regiontoken-ref-type-arena.json -j 1`: total=5, failed=5。`concat_result` / `out_region` / `scratch_raw` の失敗は解消し、現在の top issue は既知の `ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B`。
- `node nodesrc/tests.js -i stdlib\alloc\collections\vec.nepl --no-tree -o tmp\string-regiontoken-ref-vec.json -j 1`: total=39, passed=29, failed=10。残件は既知の Vec element load provenance。
- `node nodesrc/tests.js -i stdlib\neplg2\core\ty\ty.nepl -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\string-regiontoken-ref-ty-prelude.json -j 1`: total=2, passed=2
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
