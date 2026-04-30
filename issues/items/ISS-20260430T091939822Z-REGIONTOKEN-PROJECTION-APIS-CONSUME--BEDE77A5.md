---
id: ISS-20260430T091939822Z-REGIONTOKEN-PROJECTION-APIS-CONSUME--BEDE77A5
title: "RegionToken projection APIs consume owner token before cleanup"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260430T091939822Z-REGIONTOKEN-PROJECTION-APIS-CONSUME--BEDE77A5: RegionToken projection APIs consume owner token before cleanup

## 概要

region_ptr and region_ptr_at are read-only projection APIs, but they take RegionToken by value. Safe code that projects a MemPtr cannot keep the owner token available for dealloc_region, and memory_safety doctests therefore either leak the allocation or pressure tests toward raw cleanup.

## 対象

- `stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `region_ptr` / `region_ptr_at` は `RegionToken<T>` の読み取り専用投影であるにもかかわらず by-value 引数を取っていたため、投影後に `dealloc_region token` へ owner token を渡せなかった。
- Resource IR lowering は `&token` から `get_ref token "ptr"` へ進む borrow projection を raw-address alias として保持できず、`RegionToken` 内の owned storage と `MemPtr` view の対応を失っていた。
- `mem_ptr_wrap (mem_ptr_addr view)` のような再ラップは non-owning view であるべきだが、construct field の owner transfer が raw alias を実 owner とみなして owner を移動していた。
- `NoFreeObligation` は「この値自体に free 義務がない」ことを表す marker であり、path merge 時に raw owner canonical へ畳み込むと実 owner state を覆い隠す。
- raw-address alias group の canonical 代表が同順位 local projection の挿入順に依存しており、`p.raw` と `q.raw` が同じ address でも store/load の cell state が別名側へ割れる場合があった。
- `dealloc_ptr` / `dealloc` のように破壊的 raw memory operation を関数内に隠す wrapper call では、callee 側の `dealloc_raw` が caller 側の live non-Copy raw cell と照合されず、unknown-offset view からの dealloc が通る false negative があった。

## 問題

region_ptr and region_ptr_at are read-only projection APIs, but they take RegionToken by value. Safe code that projects a MemPtr cannot keep the owner token available for dealloc_region, and memory_safety doctests therefore either leak the allocation or pressure tests toward raw cleanup.

## 影響

Valid checked-memory code cannot express projection followed by exactly-once region cleanup. This blocks the MemPtr=non-owning pointer / RegionToken=owner-token separation required by the static check complexity reduction plan.

## 修正方針

Make RegionToken projection helpers borrow the token, keep MemPtr results as non-owning views, and update focused regression tests to deallocate the original token after projection use.

## 修正内容

- `stdlib/core/mem.nepl` の `region_ptr` / `region_ptr_at` / `region_size` / `region_in_bounds` を `&RegionToken<T>` ベースに揃え、読み取り投影で owner token を消費しない API にした。
- Resource IR の borrow lowering / raw-alias propagation / owner check で、`&T` の raw-address alias を参照値本体ではなく deref 先へ保持するようにした。
- `DeclareLocal` / `Assign` / `Branch` / `Match` / `Construct` で、raw-address alias から作られた `MemPtr` を non-owning view として扱い、owner transfer ではなく alias/marker copy にする経路を追加した。
- owner path merge では `NoFreeObligation` marker を raw owner canonical へ畳み込まず、実 ownership state と non-owning marker を分離した。
- raw-address alias canonical は root/projection/offset の構造順で代表を決めるようにし、挿入順によって moved/initialized/uninit cell state が割れないようにした。
- raw cell initialization summary に破壊的 raw memory requirement を追加し、`dealloc_ptr -> dealloc -> dealloc_raw` のような wrapper call でも caller 側で live non-Copy cell conflict を検出するようにした。
- 回帰テストでは、`Result::Err` 分岐も含めてすべての fallible path で元の `RegionToken` を閉じる形にし、静的検査が正しく leak を検出できる前提を崩さないようにした。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_region_ptr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_alloc_ptr_raw_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-region-token-cleanup.json -j 1 --dist web/dist`: 110 passed / 0 failed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-region-token-cleanup.json -j 1 --dist web/dist`: 12 passed / 0 failed
