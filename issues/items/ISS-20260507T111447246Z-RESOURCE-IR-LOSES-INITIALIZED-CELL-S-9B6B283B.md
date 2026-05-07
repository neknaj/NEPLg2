---
id: ISS-20260507T111447246Z-RESOURCE-IR-LOSES-INITIALIZED-CELL-S-9B6B283B
title: "Resource IR loses initialized cell state after MemPtr retagged fill"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T111447246Z-RESOURCE-IR-LOSES-INITIALIZED-CELL-S-9B6B283B: Resource IR loses initialized cell state after MemPtr retagged fill

## 概要

`MemPtr<u8>` から `mem_ptr_addr` / `mem_ptr_wrap` で `MemPtr<i32>` を作り、同じ region に `fill_u8` の後 `fill_i32` を行うと、直後の `load_i32` が `resource.cell.uninit` で拒否される。

raw fill helper の initialized-cell transition が retagged typed pointer view に安定して伝播していない。

## 対象

- `nepl-core/src/resource, tests/stdlib/memory_safety.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-after-resource-tuple-field.json -j 1 --dist web/dist` で `doctest#8` が `resource.cell.uninit` になった。
- 失敗箇所は `alloc_region<u8>` から `region_ptr &token` で得た `MemPtr<u8>` を `mem_ptr_wrap mem_ptr_addr p_u8` で `MemPtr<i32>` に retag し、`fill_u8 p_u8 16 0` と `fill_i32 p_i32 4 7` の後に `load_i32 p_i32` する経路である。
- `fill_i32` の直後の `load_i32` は同じ region の同じ typed cell view を読むため、true load-before-store ではない。
- `ISS-20260429T233515324Z-RESOURCE-IR-DOES-NOT-SUMMARIZE-RAW-F-48450939` で raw fill helper 自体の initialized-cell summary は導入済みだが、別 typed `MemPtr` view をまたぐ raw cell state には不足が残っている。

## 問題

`MemPtr<u8>` view と `MemPtr<i32>` view が同じ compiler-owned region storage を指すにもかかわらず、Resource IR の initialized cell 判定が typed pointer 変数または projection へ寄りすぎている。

そのため byte fill 後に typed fill を行っても、後続の typed load が同じ storage 上の initialized `i32` cell として照合されず、`Uninit` と誤判定される。

## 影響

正しい raw memory 初期化 sequence が Resource IR の false positive で拒否され、memory_safety doctest#8 と self-host の typed buffer 初期化経路を阻害する。

## 修正方針

Raw memory fill/load の initialized state を pointer variable 名ではなく compiler-owned storage/provenance と typed cell projection に基づいて照合し、異なる typed MemPtr view 間でも同一 storage の初期化済み state を安全側に伝播させる。

## 検証

Resource IR 回帰テストで MemPtr retag + fill_u8 + fill_i32 + load_i32 を通し、tests/stdlib/memory_safety.n.md doctest#8 と関連 memory_safety suite を確認する。

## 関連

- 親 issue: `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- 関連済み issue: `ISS-20260429T233515324Z-RESOURCE-IR-DOES-NOT-SUMMARIZE-RAW-F-48450939`
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource check

## 対応結果

Resource IR の raw cell initialization summary に、direct / variant-gated の param raw byte range を追加した。`fill_i32` のように `Result::Ok` でだけ raw memory range を初期化する helper は、成功分岐の pending summary として address param、count param、range unit、element type を保持する。

`load_i32` などの variant-gated load requirement は、直接 `CellState::Initialized` だけでなく、同じ raw address を cover する initialized byte/element range でも満たすようにした。これにより `MemPtr<u8>` から retag した `MemPtr<i32>` でも、同一 compiler-owned storage 上の `fill_i32` 成功後の `load_i32` を正しく accepted と判定できる。

この修正は `RawMemoryLoadCell` gate を弱めていない。load-before-store は引き続き `CellState` または initialized range が存在しない場合に拒否される。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_accepts_retagged_mem_ptr_after_byte_and_word_fill -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_borrowed_region_ptr_retag_then_region_dealloc -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_fill_helpers_initialize_copy_cells -- --nocapture`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt -p nepl-core --check`: passed
- `trunk build --release`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 8 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-retag-fill-initialized-cell.json -j 1 --dist web/dist`: 14 passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-retag-fill-initialized-cell.json -j 1 --dist web/dist`: 110 passed
