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

## 2026-04-28 MemPtr raw alias 部分対応

`move_check` の raw place tracking が i32 address alias に偏っており、`mem_ptr_addr` で同じ `MemPtr<T>` から raw address を複数回取り出すと別 place として扱われる問題を `ISS-20260427T183234007Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-CE6E5F55` として分離し、修正した。

この対応で `mem_ptr_wrap` / `mem_ptr_addr` / `MemPtr` 変数コピーは同じ raw place key に畳み込まれる。これにより、同じ `MemPtr` 由来 address から non-Copy value を二重に `load<T>` する経路は D3100 で拒否される。ただし、`RegionToken` の forging、owner/free 責務、initialized cell tracking はまだこの親 issue の残件である。

## 2026-04-28 MemPtr dealloc 部分対応

`dealloc_ptr<T>` が `MemPtr<T>` 由来の raw place に initialized non-Copy payload が残っているか確認しない問題を `ISS-20260427T184214411Z-MOVE-CHECK-ALLOWS-RAW-DEALLOC-WITH-L-6543A0A2` として修正した。これにより、`MemPtr<T>` から `store<T>` 済み cell を consume せずに `dealloc_ptr<T>` する経路は D3100 で拒否される。

この対応は `MemPtr` の raw place 正規化を使った局所検査であり、`MemPtr` が non-owning pointer と storage owner の両方に使われる設計そのものはまだ未解決である。

## 2026-04-28 RegionToken raw alias 部分対応

`RegionToken<T>` の underlying raw place を `move_check` の alias state に接続していなかった問題を `ISS-20260427T185057228Z-MOVE-CHECK-DOES-NOT-CONNECT-REGIONTO-665927E2` として修正した。`region_new` / token copy / `region_ptr` projection / `dealloc_region` は同じ raw place key に畳み込まれ、live non-Copy payload を残した region dealloc は D3100 になる。

この対応は既存 stdlib token を compiler 検査へ接続する暫定的な安全側強化であり、`RegionToken` を compiler-issued owner capability にする設計変更は未解決である。

## 2026-04-28 MemPtr realloc 部分対応

`realloc_ptr<T>` が `MemPtr<T>` の old range に live non-Copy payload が残るか確認しない問題を `ISS-20260427T185656579Z-MOVE-CHECK-ALLOWS-REALLOCATING-RAW-S-45B12E2B` として修正した。これにより、payload ownership を consume しないまま `MemPtr<T>` storage を byte-level realloc する経路は D3100 になる。

この対応も局所的な raw place alias 検査であり、`MemPtr<T>` の owner/non-owner 分離は未解決である。

## 2026-04-28 mem_ptr_add alias 部分対応

`mem_ptr_add<T>` が `move_check` の raw place 正規化に含まれておらず、`mem_ptr_add p 0` で同じ storage を別 raw place として扱える問題を `ISS-20260427T191722304Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-FEAEF49B` として修正した。literal offset の `mem_ptr_add` は base `MemPtr` の canonical raw place に offset を加えた key へ畳み込まれ、same-place の二重 non-Copy load や live-payload dealloc は D3100 になる。

この対応は pointer arithmetic の literal offset に対する局所検査であり、未知 offset や owner/non-owner 分離は compiler-owned Resource IR の残件である。

## 2026-04-28 mem_ptr_add unknown offset 部分対応

`mem_ptr_add<T>` の offset が non-literal の場合に raw place key が `None` になり、base `MemPtr` の provenance を失う問題を `ISS-20260427T192528620Z-MOVE-CHECK-LOSES-PROVENANCE-FOR-MEM--A1AE98CC` として修正した。あわせて raw `i32` address `add base off` も同じ root cause で provenance を失っていたため、non-literal offset は `base+?` の unknown-offset raw place として扱い、同じ base の known raw place と保守的に overlap させる。

この対応で dynamic pointer arithmetic 経由の same-base non-Copy 二重 load / live payload overwrite / dealloc は D3100 になる。ただし、これは既存 stdlib `MemPtr` を raw place tracking に接続する安全側の補強であり、owner token と non-owning pointer を型・Resource IR で分離する根本設計はまだ未解決である。

## 2026-04-28 MemPtr byte write / bulk copy 部分対応

raw address 由来の byte write 検査だけでは、`MemPtr<i32>` overload の `store_i32` や typed `mem_copy<T>` / `mem_move<T>` が caller 側の raw place state に接続されない問題を `ISS-20260427T212724800Z-MOVE-CHECK-ALLOWS-MEMPTR-BYTE-WRITES-9D19BC9D` として修正した。`MemPtr<T>` の destination/source も raw place key に正規化し、live non-Copy payload との重なりを D3100 で拒否する。

この対応も現行 `MemPtr<T>` を raw place tracking へ接続する補強であり、`MemPtr` の owner/non-owner 型分離と compiler-owned resource token 化は引き続きこの親 issue の残件である。

## 2026-04-28 helper function raw effect propagation 部分対応

`MemPtr<T>` の raw place 正規化を call site で行っても、`fn helper(p: MemPtr<i32>): store_i32 p 0` のように helper 関数へ隠すと caller の live non-Copy raw place を検査できない問題を `ISS-20260427T214055047Z-MOVE-CHECK-IGNORES-RAW-MEMORY-WRITES-417A7103` として修正した。

今回の対応で、関数サマリに raw memory 副作用を持たせ、`MemPtr<T>` 引数由来の byte write / bulk copy / dealloc / realloc も caller 引数の raw place に instantiate して検査する。これは現行 HIR 上の補強であり、`MemPtr` の owner/non-owner 分離と Resource IR 化は引き続き必要である。

## 2026-04-28 higher-order raw effect propagation 部分対応

`MemPtr<T>` を受け取る callback を higher-order helper へ渡すと、`CallIndirect` で raw memory effect が途切れる問題を `ISS-20260427T215657067Z-MOVE-CHECK-LOSES-RAW-MEMORY-EFFECTS--BDFF8DD5` として修正した。`@fn` と function-typed parameter の function value alias を追跡し、known callback の raw memory effect を caller の `MemPtr` raw place に instantiate する。

この対応も現行 HIR 上で `MemPtr<T>` provenance を補強するものであり、`MemPtr` の owner/non-owner 型分離と compiler-owned resource token 化は引き続きこの親 issue の残件である。

## 2026-04-28 region_ptr_at Ok binding 部分対応

`region_ptr_at<T,U> token off` の `Result::Ok` payload を match bind した `MemPtr<U>` が `RegionToken` の raw place provenance に接続されない問題を `ISS-20260427T194024586Z-MOVE-CHECK-LOSES-REGIONTOKEN-PROVENA-711BD515` として修正した。Ok payload bind は token raw place + offset に正規化し、non-literal offset は `base+?` として扱う。

この対応で bounds-checked projection API から取り出した `MemPtr` も、元 region と同じ raw ownership state に接続される。ただし、`Result<MemPtr<T>,E>` payload の provenance を compiler-owned Resource IR として一般化する設計変更はまだ未解決である。

## 2026-04-28 enum payload raw alias 部分対応

`Result::Ok p` や `region_ptr_at token off` の結果を enum 変数に保存した後で match すると、payload の `MemPtr` provenance が復元されない問題を `ISS-20260427T194927207Z-MOVE-CHECK-LOSES-RAW-ALIASES-STORED--5E0586DB` として修正した。`MoveCheckContext` に enum payload raw alias stack を追加し、let/set、snapshot/restore、branch merge、match bind に接続した。

この対応で enum wrapper 変数を挟んでも `MemPtr` payload は raw ownership state に戻る。ただし、これは現行 HIR 上の raw alias tracking の補強であり、enum payload を含む resource provenance を型・Resource IR で一貫表現する根本設計はまだ未解決である。

## 2026-04-28 enum payload callback raw effect 部分対応

`Option<(MemPtr<T>)->()>` のように `MemPtr` を操作する callback を enum payload に保存すると、match-bind 後の indirect call で callback の raw memory effect が caller の `MemPtr` raw place に戻らない問題を `ISS-20260427T221533970Z-MOVE-CHECK-LOSES-RAW-EFFECTS-THROUGH-308A8AC3` として修正した。

今回の対応では、enum payload function alias を `MoveCheckContext` と関数サマリに保持し、`match` payload bind で function value alias を復元する。これにより、`MemPtr` を渡す callback を `Option::Some` などに包んでも raw ownership state は D3100 検査へ伝播する。ただし、callback、enum payload、pointer provenance を型・Resource IR で一貫表現する根本設計は引き続きこの親 issue の残件である。

## 修正方針

`MemPtr<T>` は borrowed/non-owning pointer、`OwnedRegion<T>` または `Storage<T>` は free 責務を持つ owner、`InitializedCell<T>` は initialized state を持つ place、のように役割を分ける。compiler Resource IR では allocator が発行した resource token と pointer projection を扱い、raw address expression ではなく resource id / offset / initialized state / borrow state を共有する。stdlib の `RegionToken<T>` はこの compiler-owned model の safe wrapper として再設計する。

## 検証

owner token の duplicate / copy / forged token を compile_fail にする。`MemPtr<T>` の copy は non-owning pointer として許可しつつ、free は owner token だけに許可する。raw load/store の move semantics は Resource IR dump snapshot と compile_fail/normal regression の両方で確認する。既存 collection は element owner を drop する path と storage only dealloc path を分けて検証する。
