---
id: ISS-20260507T050057362Z-RESOURCE-IR-REALLOC-SUCCESS-LOSES-IN-36BCA745
title: "Resource IR realloc success loses initialized raw range facts"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-21
target: "nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260507T050057362Z-RESOURCE-IR-REALLOC-SUCCESS-LOSES-IN-36BCA745: Resource IR realloc success loses initialized raw range facts

## 概要

After fill_u8/fill_i32 records an initialized range for a raw allocation, realloc_raw success moves ownership and fixed raw cell facts to the new address but leaves the typed range attached to the old address. A guarded symbolic load from the grown pointer is therefore rejected even though realloc preserves the initialized prefix.

## 対象

- `nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/tests/resource_ir.rs`

## 根拠

realloc success path copies initialized Copy raw cells but does not rekey InitializedRawByteRange facts created by fill_u8/fill_i32.

## 問題

After fill_u8/fill_i32 records an initialized range for a raw allocation, realloc_raw success moves ownership and fixed raw cell facts to the new address but leaves the typed range attached to the old address. A guarded symbolic load from the grown pointer is therefore rejected even though realloc preserves the initialized prefix.

## 影響

Scanner and buffer growth code cannot prove that bytes written before realloc remain initialized after a successful grow. This blocks the returned-header/fd_read/capacity model without weakening RawMemoryLoadCell.

## 修正方針

Transfer only initialized raw range facts whose address is under the realloc source to the success result address, preserving count/unit/type and keeping failure path facts on the original address.

## 検証

Add Resource IR regressions for guarded fill_u8 and fill_i32 loads after realloc success, plus existing realloc and returned-header range tests.

## 2026-05-07 修正

`RawMemoryOp::Realloc` の success path で、旧 raw address 配下の `InitializedRawByteRange` を新しい realloc result address へ転送するようにした。

修正内容:

- `CellTable` に `copy_initialized_raw_byte_ranges_under` を追加し、address が realloc source 配下にある range だけを result address へ再投影する。
- count / unit / ty はそのまま保持するため、`fill_u8` の byte range と `fill_i32` の element-size scaled range の両方が preserved range として扱われる。
- 既存の `clear_raw_cells_under(source)` は source 側の raw cell / range を消すので、成功分岐では旧 pointer ではなく new pointer にだけ range fact が残る。
- failure 分岐は既存どおり result 側 alias を消すだけで、source 側の initialized range を保持する。

追加した回帰:

- `resource_ir_cell_check_realloc_transfers_initialized_byte_ranges`
- `resource_ir_cell_check_realloc_transfers_initialized_element_ranges`

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized_byte_ranges -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized_element_ranges -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_copy_raw_cells -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_guarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_rejects_unguarded_byte_range -- --nocapture`: passed

## 2026-05-20 再オープン

Resource raw cell lifecycle 境界の監査中に、`resource_ir_cell_check_realloc_transfers_initialized_element_ranges` が current main baseline でも `RawMemoryLoadCell` / `Uninit` で失敗することを確認した。これは 2026-05-07 の修正後に別変更で再発した regression と扱う。

再修正では、`RawCellLifecycleEvent::ReallocSuccessTransfer` が単に range entry を再投影するだけでなく、element stride、count、success/failure 分岐、raw alias の postcondition を typed transition proof として保持する必要がある。timeout 延長、テスト緩和、stdlib helper 名の allowlist ではなく、Resource IR state から initialized range preservation を証明すること。

## 2026-05-21 再修正

再発原因は `realloc` の range 転送自体ではなく、`off = mul i 4` の scalar scale fact が `i` の既知値に引きずられて誤った source を選んでいたことだった。`i` が `id 2` で既知値を持つと、旧実装は `off = 2 * 4` として literal 側を scale source にし、`i < len` の branch condition fact と element range proof を接続できなかった。

修正内容:

- `record_direct_call_i32_facts` の `mul` scale fact で、単に「先に見つかった既知正数」を scale にするのではなく、literal / temporary constant 由来の operand を scale として優先し、local / return / storage など意味を持つ scalar origin を source として保持する。
- これにより、`i` が既知値を持つ場合でも `off = i * 4` は `source = i, scale = 4` として Resource IR の generic range proof へ渡される。
- stdlib helper 名や個別モジュールの allowlist は追加していない。`RawCellAddressAliases` の scalar fact と branch condition fact を用いた汎用証明の修正である。

追加した回帰:

- `records_i32_scale_result_preferring_literal_scale_over_known_index`

検証:

- `cargo test -p nepl-core i32_call_facts -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized_element_ranges -- --test-threads=1 --exact`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized_byte_ranges -- --test-threads=1 --exact`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_word_fill_accepts_scaled_symbolic_load_with_range_guard -- --test-threads=1 --exact`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_word_fill_requires_guard_for_scaled_symbolic_load -- --test-threads=1 --exact`: passed
