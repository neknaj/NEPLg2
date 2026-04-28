---
id: ISS-20260428T223917440Z-RESOURCE-CELLSTATE-LETS-MOVED-REGION-D9FDA87D
title: "Resource CellState lets moved RegionToken poison dereferenced raw cells"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/resource/type_pattern.rs, nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260428T223917440Z-RESOURCE-CELLSTATE-LETS-MOVED-REGION-D9FDA87D: Resource CellState lets moved RegionToken poison dereferenced raw cells

## 概要

`RegionToken<T>` から取り出した `MemPtr<T>` 経由の raw cell が、token value の move state と混ざって `Moved` / `Uninit` と判定される。`CellState` が aggregate owner の move を `Deref` 境界の先まで伝播し、Resource IR lowering が `RegionToken.ptr.raw` を raw address alias として復元できていなかった。

## 対象

- `nepl-core/src/resource/cell_state.rs`
- `nepl-core/src/resource/lower.rs`
- `nepl-core/src/resource/coverage.rs`
- `nepl-core/src/resource/type_pattern.rs`
- `nepl-core/tests/resource_ir.rs`
- [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 4

## 根拠

- `tests/tutorials` の残存 D3100 に `concat_result` / `out_region` / `scratch_raw` 系の `RawMemoryLoadCell` false positive が残り、外部 raw root 修正後も `RegionToken` 由来 pointer の alias が途切れていた。
- `resource_ir_cell_check_preserves_region_token_ptr_helper_alias_after_token_move` の再現では、`fn token_ptr(token): get token "ptr"` が `load token` に下がり、caller 側の `q.raw` が `p.raw` / `token.ptr.raw` と同一 raw address として扱われなかった。
- `region_new` の Resource IR alias が `ptr.raw -> token whole` になり得ており、token value と pointee cell state の責務分離を崩していた。

## 問題

`RegionToken` は `ptr` と `size` を持つ aggregate value だが、raw memory cell は `token.ptr.raw.Deref` の先にある storage state であり、token value 自体の move / drop state と同じ階層として扱ってはいけない。さらに、typecheck が `get_field` を offset付き `load` に正規化した後、lowering 側が generic field type `MemPtr<.T>` と実体 `MemPtr<LocalToken>` を対応付けられず、field projection を復元できなかった。

## 影響

Valid stdlib string allocation helpers such as concat_result derive MemPtr values from a RegionToken, then use the pointer after the token value has been consumed by helper calls. Resource IR rejects those loads before the intended owner/free model can be checked.

## 修正方針

Stop non-initialized aggregate owner state from flowing through Deref or StorageOffset into raw memory cells, and teach raw address summaries that RegionToken.ptr exposes the underlying MemPtr raw address.

## 対応

- `CellTable::availability_state` で non-initialized ancestor state を伝播する際、`Deref` / `StorageOffset` を越える descendant には value move/drop state を流さないようにした。
- `region_new` の Resource IR alias を `ptr.raw -> token.ptr.raw` に変更し、token value 全体を raw address alias target にしないようにした。
- `raw_address_alias_target` と helper return の raw address source 推定に `RegionToken.ptr.raw` を追加し、`get token "ptr"` と typecheck 後の `load token` の両方から `MemPtr` raw alias を復元できるようにした。
- Generic field pattern `MemPtr<.T>` と実際の result type `MemPtr<LocalToken>` を lower 側で照合できるようにし、typecheck 済み field load の projection 復元を壊さないようにした。
- Resource IR coverage gate でも同じ generic field pattern 照合を使い、`get_field` 由来の `load token` を raw memory loss ではなく typed field `Read` として数えるようにした。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_region_token_ptr_helper_alias_after_token_move -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check -- --nocapture`: 25 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/region-token-raw-cell-move-effect-rebased-final.json -j 1`: 110 passed
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md --no-tree -o tmp/region-token-raw-cell-hello-world-rebased-final.json -j 1`: 1 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass

## 検証

Add Resource IR regression for a RegionToken-derived MemPtr load after helper extraction; run nepl-core resource_ir tests, focused CLI move tests, issue check, and diff check.
