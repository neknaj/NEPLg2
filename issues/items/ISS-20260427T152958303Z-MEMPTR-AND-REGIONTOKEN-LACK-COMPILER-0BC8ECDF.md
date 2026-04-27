---
id: ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF
title: "MemPtr and RegionToken lack compiler owned provenance model"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/core/mem.nepl, stdlib/core/traits/copy.nepl, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, doc/compare/memory_model.md"
---

# ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF: MemPtr and RegionToken lack compiler owned provenance model

## 概要

`MemPtr<T>` と `RegionToken<T>` は stdlib 上の struct として表現されており、compiler が provenance、initialized state、ownership、borrow/lifetime を所有する resource として扱っていない。`MemPtr<T>` は Copy な non-owning address だが、collection storage や self-host outcome では owning storage handle としても使われている。

## 対象

- `stdlib/core/mem.nepl, stdlib/core/traits/copy.nepl, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, doc/compare/memory_model.md`

## 根拠

- `stdlib/core/mem.nepl:100` の `RegionToken<T>` は `ptr` と `size` を持つ stdlib struct で、compiler builtin resource ではない。
- `stdlib/core/mem.nepl:110` / `113` / `116` は `RegionToken<T>` から `MemPtr<T>` と size を取り出し、任意 `MemPtr` から token を作れる。
- `stdlib/core/traits/copy.nepl:151` / `155` により `MemPtr<T>` は `Clone` / `Copy` で、所有権を持たない型付き address と説明されている。
- `stdlib/alloc/collections/vec.nepl:127` は `Vec<T>` の storage を `data <MemPtr<T>>` として保持する。
- `stdlib/alloc/collections/stack.nepl:116` や `binary_heap.nepl:41` は header pointer を owning storage として保持し、その中に element pointer を raw address として保存している。
- `stdlib/neplg2/core/infra/outcome.nepl:47` は `SelfhostOutcome<T,E>` の `Result<T,E>` cell を `MemPtr<Result<T,E>>` として保持する。
- `nepl-core/src/passes/move_check.rs:61` 以降の raw memory tracking は address expression と call name をもとに local place state を推測している。
- `nepl-core/src/passes/drop_insertion.rs` も drop elaboration 側で raw load/store や field address を別途 special-case しており、move/drop/borrow で共有される provenance model がない。

## 問題

同じ `MemPtr<T>` が non-owning borrowed address と owning storage handle の両方に使われているため、型だけでは「誰が free するか」「cell が初期化済みか」「load は copy か move か」「borrow がどの region に紐づくか」を表現できない。`RegionToken<T>` も stdlib code から再構成できるため、compiler が発行した capability としての強度を持たない。

## 影響

collection / self-host outcome / temporary buffer が増えるほど、所有者と raw address alias の対応を compiler が追跡できず、move check と drop insertion の special-case が増える。これは memory safety の false negative だけでなく、正しい code を誤って拒否する false positive と compile-time complexity の増加にもつながる。

## 2026-04-28 追加レビュー追記

責務分割レビューでは、`MemPtr<T>` / `RegionToken<T>` の設計問題が 3 種類に分かれることを確認した。

- `MemPtr<T>` は `stdlib/core/traits/copy.nepl` で non-owning Copy address と説明されるが、collection storage や `SelfhostOutcome` の cell owner としても使われている。
- `RegionToken<T>` は `stdlib/core/mem.nepl:116` の `region_new` で stdlib code から再構成できるため、compiler-issued owner/provenance capability ではない。
- typed `mem_copy<T>` / `mem_move<T>` は `MemPtr<T>` を受け取るが `T: Copy` 制約も source invalidation もないため、`ISS-20260427T164412420Z-CORE-MEM-TYPED-MEM-COPY-AND-MEM-MOVE-621A41C7` として分離した。
- `dealloc_raw` / `dealloc_ptr` / `dealloc_region` は initialized non-Copy payload の drop obligation を表現しないため、`ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47` として分離した。

したがって修正は「`MemPtr` を少し厳しくする」だけでは足りない。non-owning pointer、storage owner、initialized cell、borrow projection、drop obligation を compiler-owned Resource IR で分け、stdlib 側はその wrapper として設計する必要がある。

## 修正方針

`MemPtr<T>` は borrowed/non-owning pointer、`OwnedRegion<T>` または `Storage<T>` は free 責務を持つ owner、`InitializedCell<T>` は initialized state を持つ place、のように役割を分ける。compiler Resource IR では allocator が発行した resource token と pointer projection を扱い、raw address expression ではなく resource id / offset / initialized state / borrow state を共有する。stdlib の `RegionToken<T>` はこの compiler-owned model の safe wrapper として再設計する。

## 検証

owner token の duplicate / copy / forged token を compile_fail にする。`MemPtr<T>` の copy は non-owning pointer として許可しつつ、free は owner token だけに許可する。raw load/store の move semantics は Resource IR dump snapshot と compile_fail/normal regression の両方で確認する。既存 collection は element owner を drop する path と storage only dealloc path を分けて検証する。
