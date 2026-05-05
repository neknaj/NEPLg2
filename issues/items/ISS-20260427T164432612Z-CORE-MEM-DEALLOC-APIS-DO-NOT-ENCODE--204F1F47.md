---
id: ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47
title: "core mem dealloc APIs do not encode drop obligations for initialized storage"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-05-05
target: "stdlib/core/mem.nepl, nepl-core/src/passes/drop_insertion.rs, nepl-core/src/passes/move_check.rs, stdlib/alloc/collections/**"
---

# ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47: core mem dealloc APIs do not encode drop obligations for initialized storage

## 概要

`dealloc_raw` / `dealloc_ptr` / `dealloc_region` は address と size で storage を解放するだけで、その region 内の non-Copy value が initialized か、drop/consume 済みかを型にも compiler state にも表現しない。

## 対象

- `stdlib/core/mem.nepl, nepl-core/src/passes/drop_insertion.rs, nepl-core/src/passes/move_check.rs, stdlib/alloc/collections/**`

## 根拠

- `stdlib/core/mem.nepl:167` の `dealloc_region<T>` は `RegionToken<T>` から `ptr` と `size` を取り出して `dealloc_ptr<T>` に渡す。
- `stdlib/core/mem.nepl:386` の `dealloc_raw` は raw address と size だけで free list へ戻す。
- `stdlib/core/mem.nepl:1039` の `dealloc_ptr<T>` は `MemPtr<T>` と byte size を raw `dealloc` に渡すだけで、`T` の initialized/drop state を扱わない。
- `stdlib/core/mem.nepl:116` の `region_new<T>` により `RegionToken<T>` は stdlib code から再構成でき、compiler-issued capability ではない。
- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` と `ISS-20260427T132414663Z-SELFHOSTOUTCOME-FREE-DROPS-ONLY-STOR-CFD7EA86` は collection / single-cell owner で同じ問題が表面化している。
- `nepl-core/src/passes/drop_insertion.rs` は scope 上の typed HIR value に対して drop を挿入するが、raw region 内の initialized cells を compiler resource として保持していない。

## 問題

storage-only free と owner-cell destruction の責務が同じ API に混ざっている。`core/mem.nepl` は byte allocator と pointer wrapper を提供しているが、compiler は「この region には initialized non-Copy value が n 個あり、まだ drop obligation が残っている」という resource を持っていない。そのため stdlib 側が `dealloc_*` を呼ぶだけで payload leak を作れる。

## 影響

generic storage owner は payload を drop せず bytes だけ解放できる。逆に stdlib 側が場当たり的に drop loop を増やすと、将来 Resource IR による auto-drop と衝突して double drop になる。self-host の AST、diagnostic、buffer、`Result` cell の失敗経路で leak が見えにくくなる。

## 2026-04-28 raw dealloc move_check 部分対応

`move_check` が raw `load` / `store` だけを ownership event として扱い、`dealloc_raw` / `dealloc_ptr` は live initialized state を確認していなかった問題を `ISS-20260427T184214411Z-MOVE-CHECK-ALLOWS-RAW-DEALLOC-WITH-L-6543A0A2` として分離し、修正した。

この対応で、non-Copy value を `store<T>` した raw place を `load<T>` などで consume せずに `dealloc_raw` / `dealloc_ptr` へ渡す経路は `D3100` になる。raw place は i32 address と `MemPtr<T>` 由来 address の両方を正規化して検査する。

ただし、この親 issue はまだ閉じない。`dealloc_region` の compiler-issued capability 化、owner token と storage-only dealloc の型分離、stdlib collection の要素 drop contract、Resource IR での initialized cell / drop obligation 表現は残件である。

## 2026-04-28 RegionToken dealloc 部分対応

`RegionToken<T>` が `MemPtr<T>` と同じ raw storage provenance を持つにもかかわらず、`move_check` が token alias と `dealloc_region` を raw place state に接続していなかった問題を `ISS-20260427T185057228Z-MOVE-CHECK-DOES-NOT-CONNECT-REGIONTO-665927E2` として分離し、修正した。

この対応で、`region_new` / `RegionToken` construct / token copy から raw place alias を引き継ぎ、`dealloc_region<T> token` も live non-Copy payload が残っていれば D3100 になる。`RegionToken` が stdlib code から forgeable である問題と、storage owner token の型分離はまだこの親 issue の残件である。

## 修正方針

`Storage<T>` / `OwnedRegion<T>` / `InitializedCell<T>` の責務を分ける。uninitialized storage の dealloc は storage owner token のみで許可し、initialized cell を含む region は要素を consume/drop してから storage-only state へ戻す。compiler Resource IR は initialized state と drop obligation を tracked resource として保持し、`dealloc_*` が残 obligation を捨てる経路を拒否する。

## 検証

owning payload を `store<T>` した region を `dealloc_region` / `dealloc_ptr` だけで解放する code を compile_fail にする。payload を `load<T>` で consume した後、または `drop` した後の storage-only dealloc は通す。Copy buffer / uninitialized allocation の dealloc 正常系も維持する。collection / `SelfhostOutcome` の cleanup tests はこの issue の設計に合わせて更新する。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-04-28 issue 整理

この issue は Stage 4/6 の initialized cell / drop obligation / storage-only dealloc の境界を追跡する。`dealloc_*` の caller を一つずつ patch するだけでは解決にならない。完了条件は、Resource IR または同等の compiler-owned state が「initialized payload を含む storage」と「payload consume/drop 後の storage-only region」を区別し、後者だけを free できることである。

collection 固有の element cleanup API は `ISS-20260425T000000Z-RV-STDLIB-004-91534828`、owner token と `MemPtr` の型分離は `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` に分ける。

## 2026-05-05 対応結果

- 最新 main 同期後、`dealloc_raw` / `dealloc_ptr` / `dealloc_region` が initialized non-Copy payload を storage-only free で捨てる経路は compiler / Resource IR 側の D3100 gate で拒否されることを確認した。
- `tests/compiler/move_effect.n.md` には、raw dealloc、`dealloc_ptr`、`dealloc_region` が live non-Copy payload を捨てられない compile_fail と、payload を `load` で consume した後なら dealloc できる正常系が含まれており、110 件すべて通過している。
- `tests/stdlib/memory_safety.n.md` は 12 件すべて通過し、RegionToken / MemPtr の基本操作と invalid argument handling が現在の Resource IR owner/cell gate と整合している。
- collection 固有の element cleanup は `ISS-20260425T000000Z-RV-STDLIB-004-91534828`、`MemPtr` / `RegionToken` の owner token 型分離は `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`、safe public raw API の縮小は `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D` で継続する。

## 2026-05-05 検証

- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-current-agent1.json -j 1 --dist web/dist`: total=12, passed=12
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-current-agent1.json -j 1 --dist web/dist`: total=110, passed=110

## 2026-04-28 Stage 4 owner token / free obligation 追記

`doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 4 commit 単位 2 として、Resource IR 上で allocation の free obligation を `OwnerState` として追跡する検査を追加した。

今回追加した `check_resource_owner_obligations` は、`RawMemoryOp::Alloc` の output に `OwnerState::Live { storage }` を与え、`DeclareLocal` / `Read` / `Assign` / `Move` / `Return` で owner state を transfer する。`RawMemoryOp::Dealloc` は live owner を `Freed` にし、同じ owner を再度 read/dealloc した場合は `ResourceOwnerDiagnostic::OwnerUnavailable` を返す。関数終了時に `Live` owner が残っていれば `OwnerLeaked` として報告する。

この実装は `MemPtr` を owner として拡張するものではなく、free obligation を Resource IR の owner table として分離するための最初の足場である。現時点では compiler pipeline へ接続せず、function summary、aggregate owner、branch / loop merge、RegionToken capability 化は後続 Stage に残す。既存の D3100 防壁は維持しつつ、Resource IR 側で storage owner と initialized cell を分ける方向へ移行する。

## 2026-04-28 Resource IR raw storage op 追記

`ISS-20260428T141745924Z-RESOURCE-CELLSTATE-CHECKER-IGNORES-D-40CECA56` として、Resource IR CellState checker でも storage-only raw operation が initialized non-Copy cell を消せないようにした。`Dealloc` / `Realloc` は `address.*` 配下の live non-Copy / maybe-moved cell を拒否し、non-Copy `Load` で payload を consume 済みの storage-only dealloc は許可する。

これはこの issue の完了条件である `Storage<T>` / `OwnedRegion<T>` / `InitializedCell<T>` の API 分離そのものではないが、Resource IR 側で「payload obligation が残る region」と「storage-only region」を区別するための Stage 4 実装である。stdlib public API と owner token 化は引き続きこの issue の残件として扱う。

回帰テストでは、`alloc_raw` 後に `dealloc_raw` する正常系、未解放 allocation の leak、二重 dealloc の owner moved 診断を `nepl-core/tests/resource_ir.rs` に固定した。
