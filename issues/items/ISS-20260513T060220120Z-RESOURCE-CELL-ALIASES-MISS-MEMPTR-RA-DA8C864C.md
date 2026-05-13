---
id: ISS-20260513T060220120Z-RESOURCE-CELL-ALIASES-MISS-MEMPTR-RA-DA8C864C
title: "Resource cell aliases miss MemPtr raw identity through aggregates and returns"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/src/resource
---

# ISS-20260513T060220120Z-RESOURCE-CELL-ALIASES-MISS-MEMPTR-RA-DA8C864C: Resource cell aliases miss MemPtr raw identity through aggregates and returns

## 概要

tests/compiler/move_effect.n.md の MemPtr alias / aggregate field / Result payload / function return を経由するケースで、non-Copy raw load の二重所有値生成が `resource.cell.moved` として検出されずコンパイル成功していた。原因は alias 伝播の欠落ではなく、zero-initialized runtime cell 判定が alias group 全体の tracked storage を確認していなかったことだった。

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `tests/compiler/move_effect.n.md` の doctest#22,#23,#44-#55 が `expected compile_fail, but compiled successfully` になっていた。
- `RawMemory::Load` の zero-initialized runtime cell 判定が、alias group 内の一部 alias だけを見て「既知 non-negative raw address かつ tracked storage なし」と扱っていた。
- 同じ raw address alias group 内の別 alias に moved/tracked raw cell がある場合でも、未追跡 alias 側だけを見ると runtime zero-initialized load と誤判定できた。

## 問題

同じ raw address alias group のどこかに CellTable の raw cell entry / owned storage / external storage がある場合、その address group は既に Resource IR の cell state 管理対象である。従来実装は alias group 内の任意 alias が未追跡なら runtime zero-initialized cell とみなせたため、最初の non-Copy load で raw cell が moved になった後でも、別 alias 経由の二回目の load が moved state を迂回できた。

## 影響

Resource IR の memory safety 検査が、同じ raw cell から non-Copy value を複数回 move するケースを一部見逃していた。これは型安全・メモリ安全の必達条件に直接関わる。

## 修正方針

MemPtr raw field、aggregate field、enum payload、function return summary、branch/match merge の raw address alias 伝播を監査し、cell state が同一 raw cell を同一 place として扱えるように根本修正する。特定テストを列挙して通すのではなく、RawCellAddressAliases と CellTable の責務境界を保った設計にする。

## 修正内容

- `raw_memory_load_reads_zero_initialized_runtime_cell` を、alias group 内の一部 alias ではなく、alias group 全体で tracked storage が存在しない場合だけ true にした。
- 既知 non-negative raw address の存在は runtime linear memory の zero-initialized load を許可する条件として維持したが、同じ alias group のどこかに tracked storage がある場合は必ず CellTable の state を参照させる。
- `resource_ir_cell_check_rejects_double_non_copy_load_through_mem_ptr_alias` を追加し、同じ `MemPtr` から取り出した複数 raw address alias を通じた non-Copy raw load 二重 move が `resource.cell.moved` で拒否されることを固定した。

## 検証

- `cargo test -p nepl-core resource_ir_cell_check_rejects_double_non_copy_load_through_mem_ptr_alias -- --nocapture`
- `cargo test -p nepl-core resource_ir_cell_check_allows_zero_initialized_runtime_literal_raw_load -- --nocapture`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-resource-cell-aliases-move-effect.json -j 1 --dist web/dist`

`tests/compiler/move_effect.n.md` は 113/113 pass となり、doctest#22,#23,#44-#55 は期待通り `resource.cell.moved` に到達した。
