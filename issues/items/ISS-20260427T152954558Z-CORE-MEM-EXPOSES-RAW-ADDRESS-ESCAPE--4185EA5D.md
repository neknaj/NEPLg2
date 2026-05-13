---
id: ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D
title: "core mem exposes raw address escape hatches as safe API"
area: stdlib
status: open
resolved: false
priority: P1
type: security
created: 2026-04-27
updated: 2026-05-13
target: "stdlib/core/mem.nepl, stdlib/core/traits/copy.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D: core mem exposes raw address escape hatches as safe API

## 概要

`stdlib/core/mem.nepl` は `MemPtr<T>` を導入しているが、同時に raw `i32` address への unwrap / wrap と raw load/store を safe public API として公開している。結果として compiler が pointer provenance、bounds、ownership、effect を管理する前に、stdlib と利用者 code が raw address へ降りられる。

## 対象

- `stdlib/core/mem.nepl, stdlib/core/traits/copy.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- `stdlib/core/mem.nepl:97` で `MemPtr<T>`、`100` で `RegionToken<T>` を定義している。
- `stdlib/core/mem.nepl:104` の `mem_ptr_wrap` と `107` の `mem_ptr_addr` が raw `i32` と `MemPtr<T>` を双方向変換できる。
- `stdlib/core/mem.nepl:278` / `386` / `450` の `alloc_raw` / `dealloc_raw` / `realloc_raw` は raw `i32` address を公開する。
- `stdlib/core/mem.nepl:558` / `591` の raw `load_i32(i32)` / `store_i32(i32,i32)` と、`1101` / `1117` の generic raw `load<T>(i32)` / `store<T>(i32,T)` が public に見える。
- `stdlib/core/traits/copy.nepl:151` / `155` で `MemPtr<T>` は `Clone` / `Copy` になっており、コメント上も non-owning address とされている。
- `doc/compare/memory_model.md:47` は Phase 1 で `mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` / `realloc_raw` を公開面から除く計画を明記している。

## 問題

`MemPtr<T>` を safe pointer wrapper として扱うには、raw address への変換、任意型 load/store、allocator primitive は unsafe または compiler-owned boundary に閉じる必要がある。現状では safe source code から raw `i32` を作り、pointer arithmetic 後に任意型として読み書きできるため、`MemPtr<T>` の型引数は provenance や ownership の証明になっていない。

## 影響

型安全上は `MemPtr<T>` から別 `U` の pointer を作る、所有値を raw memory から浅く複製する、dealloc 済み address を再利用する、といった経路を compiler が根本的に遮断できない。メモリ安全上は double free / use-after-free / uninitialized read を safe API から構成できる。self-host stdlib の collection と diagnostic storage が増えるほど被害範囲が広がる。

## 2026-04-28 追加レビュー追記

今回の責務分割レビューでは、raw address escape は `mem_ptr_wrap` / `mem_ptr_addr` だけでなく、typed に見える `MemPtr` overload へも残っていると判断した。`mem_copy<T>` / `mem_move<T>` は `MemPtr<T>` を受け取るため利用者からは typed API に見えるが、実体は raw byte copy で、`T` の ownership 制約を持たない。この点は `ISS-20260427T164412420Z-CORE-MEM-TYPED-MEM-COPY-AND-MEM-MOVE-621A41C7` に分離した。

また、`dealloc_region<T>` は `RegionToken<T>` を受け取るが、token 自体が forgeable であり、region 内の initialized value を drop 済みにする契約もない。この点は `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47` として追跡する。

## 2026-04-28 raw bulk copy 部分対応

raw `mem_copy` / `mem_move` が public raw address を受け取り live non-Copy payload を byte copy/overwrite できる問題を `ISS-20260427T190303188Z-MOVE-CHECK-ALLOWS-RAW-MEM-COPY-AND-M-AA0F96F9` として compiler 側で塞いだ。

この対応は exposed raw address API を閉じるものではなく、現行の公開面に残る危険な操作を `move_check` で安全側に拒否する暫定措置である。raw address escape hatch の unsafe/internal API 化はこの issue の残件として維持する。

## 2026-04-28 raw byte write 部分対応

`store_i32` / `store_u8` / `memset_u8` / `fill_i32` などの raw byte write helper が public raw address を受け取り live non-Copy payload を byte overwrite できる問題を `ISS-20260427T190852368Z-MOVE-CHECK-ALLOWS-RAW-BYTE-WRITES-TO-B56A7B43` として compiler 側で塞いだ。

この対応も exposed raw address API を閉じるものではなく、現行公開面に対する安全側の防壁である。safe public API と unsafe/internal raw API の分離はこの issue の残件である。

## 修正方針

public `core/mem` は checked allocation、typed pointer arithmetic、copy-only load/store、owned move in/out のような safe operation に限定する。raw `i32` address 変換と generic raw load/store は non-public または明示 unsafe module へ分離し、compiler 側の Resource IR / effect model と同期して移行する。`MemPtr<T>` は non-owning pointer と明示し、owner token / storage handle は別型へ分ける。

## 検証

safe import だけでは `mem_ptr_addr` / `mem_ptr_wrap` / raw `load<T>` / raw `store<T>` / raw allocator primitive を呼べない compile_fail を追加する。safe wrapper は bounds error を `Result` / `Option` で返す正常系を維持する。raw escape が必要な既存 stdlib は unsafe/internal module へ寄せ、使用箇所を source policy で追跡する。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-04-28 issue 整理

この issue は Stage 5/6 の stdlib public API 境界を追跡する。compiler 側の raw memory effect / ownership 境界は `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`、`MemPtr` の owner/non-owner 分離は `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` に分ける。

完了条件は、safe import から raw address escape を構成できないこと、raw identity が public pure API へ漏れないこと、raw operation が必要な stdlib 実装は internal/unsafe boundary 内へ閉じられていることである。

## 2026-05-08 MemPtr direct constructor boundary 部分対応

`MemPtr<T>` の通常 struct constructor を user source から直接呼び、`MemPtr raw` の形で raw pointer wrapper を作れる問題を `ISS-20260507T171425909Z-MEMPTR-STRUCT-CONSTRUCTOR-IS-FORGEAB-7EC211C1` として修正した。

対応では `StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::RawPointer)` を追加し、core memory boundary 内で定義された `MemPtr` direct constructor だけを raw-memory-boundary capability に制限した。これにより direct aggregate construction は safe source から閉じたが、`mem_ptr_wrap` / `mem_ptr_addr` の public API 移行はこの親 issue の残件として維持する。

検証:

- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_struct_constructor_outside_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_mem_ptr -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memptr-constructor-boundary.json -j 1 --dist web/dist`: 19 passed

## 2026-05-13 MemPtr raw address view lowering 部分対応

`MemPtr<T>` を non-owning pointer として扱う方針に対して、compiler Resource IR lowering が `mem_ptr_addr` を単なる `RawAddressAlias` として表現していた。これでは `MemPtr.raw` を raw `i32` として取り出した値が owner transfer ではなく non-owning projection であることを、後段の owner / raw view policy が enum-first に判別しにくい。

対応として `mem_ptr_addr` の lowering を `RawAddressViewKind::NonOwningProjection` の `RawAddressView` に変更した。これにより `mem_ptr_addr` は free obligation を移動させる alias ではなく、既存 storage への non-owning raw projection として Resource IR 上に明示される。

この対応は raw address public API を完全に閉じる最終対応ではない。`mem_ptr_wrap` / `mem_ptr_addr` の safe surface migration、stdlib raw-memory-boundary の縮小、`OwnedRegion` / `Storage` 型への移行は引き続きこの親 issue の残件として維持する。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_marks_mem_ptr_addr_as_non_owning_projection -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_bytebuf_owner_after_raw_address_view -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memptr-view-memory-safety.json -j 1 --dist web/dist`: 23 passed
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --no-tree -o tmp/agent1-memptr-view-reference-codegen.json -j 1 --dist web/dist`: 3 passed
- `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-tree -o tmp/agent1-memptr-view-prelude-copy.json -j 1 --dist web/dist`: 4 passed
- `node nodesrc/issues.js check --dir issues`: passed

## 2026-05-13 import visibility blocker 追記

`core/mem` の raw address escape を public surface から閉じるには、stdlib file 分割だけでなく compiler の import visibility enforcement が先に必要であることを確認した。

現状の parser / module graph には `pub` / private の概念があるが、typecheck の `Binding` は item visibility を保持していない。そのため flat loader representation では、imported file の private item も `as *` / qualified lookup から選択され得る。`mem_ptr_addr` / `mem_ptr_wrap` / raw allocator / raw load-store を internal module へ移しても、compiler が `Visibility::Pub` を binding authority にしなければ safe source から隠せない。

この blocker は `ISS-20260512T235355207Z-IMPORT-VISIBILITY-DOES-NOT-ENFORCE-P-30FB5573` として分離した。この issue の完了条件である「safe import から raw address escape を構成できないこと」は、同 blocker の解決後に `core/mem` public facade / internal raw-memory-boundary module 分割として進める。

## 2026-05-13 core/mem facade raw boundary 縮小

import visibility blocker は解決済みだったため、次の前提整理として `ISS-20260513T023254911Z-CORE-MEM-FACADE-STILL-CARRIED-RAW-ME-FEEF633F` を追加し、修正した。

`stdlib/core/mem.nepl` は public facade でありながら raw-memory-boundary capability を持ち、allocator / raw load-store / pointer wrapper / type definition が同居していた。これを `types` / `raw` / `allocator` / `pointer` submodule へ分割し、loader の exact raw-memory-boundary table から root `core/mem.nepl` を外して実装 submodule だけに capability を付与した。

これにより「public facade 自体が raw boundary privilege を持つ」状態は解消した。ただし、既存互換のため `alloc_raw` / `dealloc_raw` / `mem_ptr_addr` / generic `load` / `store` はまだ facade から re-export されている。safe import から raw address escape を完全に構成できなくする作業は、Stage 6 の public safe API / internal raw API migration としてこの issue を open のまま継続する。
