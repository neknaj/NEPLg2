---
id: ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF
title: "MemPtr and RegionToken lack compiler owned provenance model"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-05-12
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

## 2026-04-29 Resource IR RegionToken ptr.raw 部分対応

`RegionToken<T>` の value move state と `token.ptr.raw` が指す pointee cell state が混ざり、`get token "ptr"` helper 経由の `MemPtr<T>` load が `RawMemoryLoadCell` `Moved` / `Uninit` になり得る問題を `ISS-20260428T223917440Z-RESOURCE-CELLSTATE-LETS-MOVED-REGION-D9FDA87D` として修正した。

今回の対応では、Resource IR lowering の `region_new` / `get token "ptr"` / typecheck 後の `load token` を `token.ptr.raw` alias として扱い、`CellState` は `Deref` / `StorageOffset` の先へ aggregate value move state を流さないようにした。これは `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の initialized / moved state 分離の一部であり、compiler-issued owner token への大規模移行は引き続きこの親 issue の残件である。

## 2026-04-30 memory_safety 残件更新

`ISS-20260430T083411167Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-D35A0DAD` の修正で、raw `alloc` の `Result::Ok` payload 条件と checked `dealloc` の Err 到達不能性は owner checker へ伝播できるようになった。

その結果、`node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-owner-variant-value-conditions.json -j 1 --dist web/dist` は 12 total / 9 passed / 3 failed になった。残る失敗は次の通りで、この親 issue の owner token / non-owning pointer 分離が未完であることを示す。

- `doctest#6`: `RegionToken` の `token.ptr.raw` owner が `dealloc_region` / finish boundary によって最終所有者へ移ったことを Resource IR owner checker が表現しきれず、`token.ptr.raw` leak として残る。
- `doctest#7`: 同じく `RegionToken` storage owner が value wrapper 内の raw owner として残り、RegionToken 自体が compiler-issued free obligation owner でない問題が露出している。
- `doctest#8`: `MemPtr<i32>` projection `p_i32.raw` が non-owning pointer と storage owner の両方に見えるため、owner transfer/dealloc の責務境界が不明確なまま leak として残る。

この 3 件は stdlib に追加 cleanup を足して隠す問題ではなく、`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の分離を完了するまで追跡する。

## 修正方針

`MemPtr<T>` は borrowed/non-owning pointer、`OwnedRegion<T>` または `Storage<T>` は free 責務を持つ owner、`InitializedCell<T>` は initialized state を持つ place、のように役割を分ける。compiler Resource IR では allocator が発行した resource token と pointer projection を扱い、raw address expression ではなく resource id / offset / initialized state / borrow state を共有する。stdlib の `RegionToken<T>` はこの compiler-owned model の safe wrapper として再設計する。

## 検証

owner token の duplicate / copy / forged token を compile_fail にする。`MemPtr<T>` の copy は non-owning pointer として許可しつつ、free は owner token だけに許可する。raw load/store の move semantics は Resource IR dump snapshot と compile_fail/normal regression の両方で確認する。既存 collection は element owner を drop する path と storage only dealloc path を分けて検証する。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [NEPLg2 compiler diagnostic redesign plan](../../doc/neplg2/compiler_diagnostics_redesign_plan.md)

## 2026-04-29 diagnostic 再設計追記

`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` へ分ける方針では、diagnostic も pointer provenance、owner obligation、cell state を分離して表す必要がある。

現行の `D3100` bucket だけでは、MemPtr の pointer alias 問題、owner token の leak/double-free、initialized cell の moved/uninit/drop state を区別しにくい。今後は [compiler diagnostic redesign plan](../../doc/neplg2/compiler_diagnostics_redesign_plan.md) に沿って、Resource IR diagnostic kind と stable string code を対応させる。

## 2026-04-28 issue 整理

この issue は `MemPtr` / `RegionToken` を安全化しながら拡張し続けるための issue ではなく、役割分割を追跡する設計 issue とする。今後の完了条件は、`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の分離を compiler と stdlib の両方で確認できることである。

raw place alias tracking による既存回帰の防壁は維持するが、追加の container / callback / payload summary は Resource IR 移行前の暫定措置として扱う。owner token と initialized cell の責務は `ISS-20260425T000000Z-RV-CORE-009-58589A3F` と同期し、stdlib public API の移行は `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` へ分ける。

## 2026-04-28 memory safety 方針レビュー追記

`MemPtr<T>` / `RegionToken<T>` の現方針は、既存 stdlib を壊さないための過渡期としては許容できるが、NEPLG2 の安全なメモリモデルとしては複雑すぎる。特に `MemPtr<T>` が non-owning pointer と owning storage handle の両方に使われ、`RegionToken<T>` が stdlib code から再構成できるため、compiler-issued capability としての意味を持てない。

今後の基本方針は、`MemPtr<T>` を安全化して全用途を抱え込ませることではなく、役割を分割することに置く。

- `MemPtr<T>`: copy 可能な non-owning pointer / projection。
- `OwnedRegion<T>` または `Storage<T>`: free obligation を持つ owner token。
- `InitializedCell<T>` または Resource IR 上の cell state: 値が入っているか、move 済みか、drop obligation が残っているかを表す。
- `RegionToken<T>`: stdlib が勝手に forge できない compiler-issued wrapper へ移行する。

この分割が入らないまま self-host の AST / token stream / diagnostic buffer を増やすと、stdlib 側に raw pointer discipline が広がり、memory safety を compiler で証明するという方針から外れる。

## 2026-05-06 Resource IR borrowed region / raw cell 部分対応

`RegionToken<T>` / `MemPtr<T>` の値 move state と、その内部 raw address が指す initialized cell state が Resource IR 上で混線する経路を追加で修正した。

- `MemPtr` / `RegionToken` は値として `Deref` されても raw address alias を保持する型として扱い、関数サマリ内で `Deref` によって provenance を落とさないようにした。
- `region_ptr_at` / `region_ptr` / `region_size` / `region_in_bounds` を borrowed `RegionToken` ベースに整理し、pointer projection が token 自体の move と混ざらないようにした。
- `CellTable` の availability 判定で、raw cell の `Moved` / `Dropped` / `Uninit` / `MaybeMoved` が known offset / unknown offset / alias 経由の query へ保守的に伝播するようにした。
- `dealloc_ptr` や helper 内 raw dealloc のような destructive raw-memory effect は、関数サマリに call-side release requirement として記録し、caller 側で live non-Copy raw cell を検査するようにした。
- 外部参照由来 aggregate storage は、raw-memory cell の `Deref` と混同せず、external storage root の照合だけで参照先 field / storage offset を untracked external として扱うように分離した。

今回の対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource check 移行に含まれる。`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の最終分離は引き続きこの issue の残件である。

検証:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 155 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-borrowed-region-owner.json -j 1`: 12 passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-borrowed-region-owner.json -j 1`: 110 passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

## 2026-05-07 Resource IR str_addr non-owning view 部分対応

`str_addr` を user helper で包むと、戻り値の raw `i32` が owner alias と同じ経路へ落ち、`dealloc_raw addr len s` のような経路で `str` の backing storage を非所有 view から解放できる問題を確認した。

根本原因は、Resource IR lowering が zero-offset raw address を一律に alias として扱い、owner checker の `dealloc` / `realloc` 入口も raw view を確認する前に alias を辿って元 owner へ到達していたことだった。これにより、`MemPtr` の owned raw address transfer と、`str_addr` / `region_ptr` の non-owning pointer projection が同じ表現に混ざっていた。

対応として `RawAddressViewKind::Offset` / `RawAddressViewKind::NonOwningProjection` を追加し、`str_addr` と borrowed `region_ptr` は仕様上 non-owning projection として lowering する。owner checker は `NonOwningProjection` を source 未解決でも non-owning view として保持し、`dealloc` / `realloc` では raw view を owner として扱わず `OwnerUnavailable` を出す。一方で `mem_ptr_addr` と `str_from_addr_unchecked` は既存どおり owner transfer 可能な raw owner 経路として残した。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の最終分離は引き続きこの issue の残件である。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_dealloc_through_str_addr_helper_view -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_str_owner_through_str_addr_helper -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_alloc_ptr_raw_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_raw_owner_through_str_from_addr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_str_addr_helper_parameter_raw_load -- --nocapture`: passed

## 2026-05-12 Result payload owner summary 部分対応

`ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3` として、Resource IR owner summary が `Result` payload owner を unconditional projection return として扱い、runtime 上同時に存在しない `Ok` / `Err` payload owner を caller 側で materialize し得る問題を修正した。

今回の対応では、複数 variant payload が混在する enum payload projection return だけを `OwnerVariantProjectionReturn` へ正規化し、`unwrap_ok` / `unwrap_box` のような resolved variant summary と組み合わせて、選択された variant の owner だけを materialize する。あわせて raw owner summary alias walk は `DeclareLocal` / match bind / raw store value consumption でも projection suffix を保持し、`Result::Ok.field0` や raw node field に移した raw i32 owner seed を失わないようにした。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の owner token / free obligation summary 精度向上に含まれる。`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の最終分離は引き続きこの親 issue の残件である。

## 2026-05-07 Resource IR aggregate payload non-owning view 部分対応

`str_addr` の non-owning raw view を `Result::Ok` payload に包み、caller 側で match bind した後に `dealloc_raw` へ渡すと、direct view では拒否できる free bypass が再発する問題を `ISS-20260507T085434323Z-RESOURCE-OWNER-CHECKER-LOSES-NON-OWN-344F2372` として修正した。

根本原因は、Resource IR owner summary が payload projection marker を生成しているにもかかわらず、construct / branch / match / call return summary / read の value-preserving flow が non-owning raw view fact を統一的にコピーしていなかったことだった。対応では `RawAddressViewTable` で通常 raw address view と non-owning raw address view を分け、aggregate / branch / match / call return summary では non-owning fact だけを伝播する。`OwnerState::NoFreeObligation` は汎用 owner marker のまま維持し、pointer authority には流用しない。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。direct `str_addr` と aggregate payload 経由の `str_addr` はどちらも owner ではないため `dealloc_raw` / `realloc_raw` では `OwnerUnavailable` になり、`mem_ptr_addr` / `str_from_addr_unchecked` の raw owner transfer 経路とは分離したまま維持する。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_dealloc_through_result_wrapped_str_addr_view -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir str_addr -- --nocapture`: passed

## 2026-05-07 Resource IR helper parameter consumption non-owning view 部分対応

`str_addr` 由来の non-owning raw view を `mem_ptr_wrap` で `MemPtr<u8>` に包み、さらに `region_new` で `RegionToken<u8>` へ詰め直してから `dealloc_region` に渡すと、`dealloc_region` の caller-side owner summary consumption が何も診断しない問題を `ISS-20260507T134613401Z-RESOURCE-OWNER-SUMMARY-IGNORES-NON-O-9A39F228` として修正した。

根本原因は、callee summary が `RegionToken` parameter の `token.ptr.raw` を consumed owner source として示していても、caller actual projection が non-owning raw view の場合に `consume_call_argument_owner` が no-op になっていたことだった。これは direct `dealloc_raw` 入口では拒否できる non-owning view を、helper parameter consumption 経由で free obligation owner のように扱わせる抜け道だった。

対応では owner summary consumption を `owner_consumption.rs` に分離し、actual projection が non-owning raw address view なら `OwnerState::NoFreeObligation` として `resource.owner.no_free_obligation` を出すようにした。owned storage origin が残るのに transferable owner がない場合も同じく拒否し、provenance のない unmanaged fixed address は従来どおり owner summary consumption だけでは拒否しない。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。`RegionToken` を compiler-issued owner token にする最終設計はまだ残るが、少なくとも `region_new` を使って non-owning pointer projection を helper-call 境界で free obligation owner に偽装する経路は塞いだ。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_str_addr_view -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-region-forge.json -j 1 --dist web/dist`: 15 passed

## 2026-05-07 Resource IR region_ptr_at Ok payload non-owning view 部分対応

`region_ptr_at<T,U>` の `Result::Ok(MemPtr<U>)` payload を `region_new` へ詰め直すと、borrowed `RegionToken` projection が free obligation owner に偽装される問題を `ISS-20260507T143247279Z-RESOURCE-IR-OWNER-CHECKER-LOSES-NON--66D5734F` として修正した。

根本原因は、direct `region_ptr` は non-owning projection として下げていた一方で、`region_ptr_at` の実装が `region_token_ptr_ref` / `mem_ptr_addr` / `mem_ptr_wrap` / `Result::Ok` を経由するため、Ok payload の raw view fact が owner summary に残らなかったことだった。対応では `region_ptr_at` の Ok payload raw field を Resource IR lowering で明示的な `NonOwningProjection` にし、borrowed `region_token_ptr_ref` の raw field も non-owning view として扱う。

この対応により、bounds-checked projection は読み書き用の `MemPtr` としては使えるが、`region_new` で owner token に昇格して `dealloc_region` する経路は `resource.owner.no_free_obligation` で拒否される。`region_ptr_at` で取得した pointer を使って元の `RegionToken` を解放する正常系は維持する。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_region_ptr_at_ok_payload -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_borrowed_region_ptr_at_then_region_dealloc -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-region-ptr-at-forge.json -j 1 --dist web/dist`: 17 passed

## 2026-05-08 RegionToken direct constructor boundary 部分対応

`RegionToken<T>` の通常 struct constructor を user source から直接呼び、`RegionToken p size` の形で owner-token-shaped value を作れる問題を `ISS-20260507T170021735Z-REGIONTOKEN-STRUCT-CONSTRUCTOR-IS-FO-0CC2D37A` として修正した。

根本原因は、`RegionToken` が free obligation owner を表す token であるにもかかわらず、型検査上は他の struct constructor と同じ pure callable として扱われていたことだった。後段の Resource IR owner checker は forged token の使用時に `resource.owner.no_free_obligation` を出せるが、typed HIR に forge 済み token を残す設計は compiler-issued capability として弱い。

対応では `TypeDiagnosticCode::OwnerTokenConstructorRestricted` / `type.owner_token.constructor_restricted` を追加し、struct 定義時に `StructConstructorPolicy` enum で constructor policy を分類するようにした。core memory boundary 内で定義された `RegionToken` の direct constructor だけを raw-memory-boundary capability を持つ source に限定し、同名の user-defined struct は通常の public constructor のまま扱う。`stdlib/core/mem.nepl` の `region_new` は compiler-owned memory boundary 内の wrapper として維持し、user source は allocation API と Resource IR summary 経由でしか free obligation owner を得られない。

検証:

- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_struct_constructor_outside_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_region_token -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_str_addr_view -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-region-token-constructor-boundary.json -j 1 --dist web/dist`: 18 passed

## 2026-05-08 MemPtr direct constructor boundary 部分対応

`MemPtr<T>` の通常 struct constructor を user source から直接呼び、`MemPtr raw` の形で raw pointer wrapper を作れる問題を `ISS-20260507T171425909Z-MEMPTR-STRUCT-CONSTRUCTOR-IS-FORGEAB-7EC211C1` として修正した。

根本原因は、`MemPtr` が non-owning pointer wrapper であり raw address provenance の入口であるにもかかわらず、型検査上は他の struct constructor と同じ pure callable として扱われていたことだった。`mem_ptr_wrap` は compiler-known helper として Resource IR が扱えるが、direct aggregate constructor は raw pointer construction を通常 struct construction に混ぜる。

対応では `StructConstructorPolicy::RawMemoryBoundaryOnly` に `RestrictedStructConstructor::{OwnerToken,RawPointer}` を持たせ、core memory boundary 内で定義された `MemPtr` direct constructor を `RawPointer` として raw-memory-boundary capability に限定した。診断は `type.raw_pointer.constructor_restricted` として owner token 制限と分けた。`mem_ptr_wrap` の public API 移行は raw address escape 親 issue 側で継続する。

検証:

- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_struct_constructor_outside_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_mem_ptr -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memptr-constructor-boundary.json -j 1 --dist web/dist`: 19 passed

## 2026-05-08 Resource IR unknown callback non-owning view 部分対応

unknown callback 境界で same-type non-owning raw address view argument が返り得るにもかかわらず、別の same-type owner argument を definite owner return として先に選ぶ問題を `ISS-20260507T183017038Z-RESOURCE-OWNER-CHECKER-TREATS-NON-OW-95BB68AF` として修正した。

今回の対応では、unknown indirect-call return handling が non-owning view candidate を owner transfer より先に評価し、出力を non-owning raw view として伝播する。unknown callback argument consumption でも non-owning view 引数を free obligation owner として消費しない。これにより、borrowed `RegionToken` から作った `MemPtr` を callback parameter 経由で返す正常系は維持しつつ、callback result が non-owning candidate を返し得る場合の `dealloc returned` は `OwnerState::NoFreeObligation` で拒否される。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。`MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の最終分離は引き続きこの親 issue の残件である。

## 2026-05-08 Resource IR fixed raw MemPtr region_new 部分対応

固定 raw address を `mem_ptr_wrap` で `MemPtr<u8>` にし、`region_new` で `RegionToken<u8>` へ昇格してから `dealloc_region` へ渡すと、free obligation がないにもかかわらず owner checker が拒否しない問題を `ISS-20260507T185234792Z-REGION-NEW-ACCEPTS-FIXED-RAW-MEMPTR--A0FDF3E7` として修正した。

根本原因は、`region_new` の lowering が raw address alias だけを表し、`RegionToken.ptr.raw` が owned storage provenance を要求することを Resource IR に残していなかったことだった。さらに `StorageOriginTable` が value 内部の origin を prefix 付きで移動・コピーできず、whole `RegionToken` consumption から配下の owned origin を検出できていなかった。

対応では `ResourceOp::StorageOrigin` を追加し、`region_new` の出力 `RegionToken.ptr.raw` へ `StorageOrigin::Owned` を明示的に付与する。`StorageOriginTable` は value move / read / assign で配下 origin を移動・コピーし、owner checker は whole value 配下の owned origin も free obligation 要求として扱う。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。`RegionToken` を compiler-issued owner token にする最終設計は残るが、少なくとも固定 raw address から owner-token-shaped value を作って dealloc する経路は `resource.owner.no_free_obligation` で拒否される。

## 2026-05-12 Resource IR returned RegionToken storage origin 部分対応

helper が `mem_ptr_wrap 16` 由来の `RegionToken` を返し、caller 側で `dealloc_region` すると、callee 内には `RegionToken.ptr.raw` の `StorageOrigin::Owned` が存在するにもかかわらず、return summary 境界で storage origin が落ちる問題を `ISS-20260507T191618061Z-RESOURCE-OWNER-SUMMARY-DROPS-STORAGE-DAB6ECA2` として修正した。

根本原因は 2 つあった。第一に `OwnerReturnSummary` が returned value 配下の `StorageOrigin` を表現しておらず、`region_new` が要求する owned storage provenance を caller 側へ復元できなかった。第二に `variant_projection_returns` が parameter source だけを表す設計だったため、`alloc_ptr` / `alloc_region` の `Result::Ok` payload に入る fresh owner / maybe owner も caller 側へ網羅的に伝播できなかった。

対応では `OwnerReturnSummary` に `storage_origin_markers` を追加し、returned aggregate 配下の storage origin を call output の対応 projection へ復元するようにした。あわせて `OwnerVariantProjectionReturn` は `OwnerProjectionReturnOwner::{Parameter,Fresh,Maybe}` を持つ enum 設計に変更し、variant payload に入った owner source を exhaustive `match` で処理する。`RawAddressAlias` が raw owner を wrapper 内のより深い projection へ移す `mem_ptr_wrap` / `region_new` 境界では、transferable owner がある場合だけ owner state を移動し、`mem_ptr_addr` のような scalar view 取得とは分離した。

この対応は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。fixed raw 由来の returned `RegionToken` は helper return を跨いでも `resource.owner.no_free_obligation` で拒否され、`alloc_region` 由来の正当な returned `RegionToken` は caller 側で `dealloc_region` 可能なまま維持される。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_returned_allocated_region_token -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_returned_region_token_forged_from_fixed_mem_ptr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir region_token_forged -- --nocapture`: 6 passed
- `cargo test -p nepl-core --test resource_ir alloc_ptr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir variant_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: 8 passed
- `cargo fmt --check -p nepl-core`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-return-storage-origin-memory-safety.json -j 1 --dist web/dist`: 23 passed

## 2026-05-12 Resource IR returned owner provenance 追加補強

`ISS-20260507T191618061Z-RESOURCE-OWNER-SUMMARY-DROPS-STORAGE-DAB6ECA2` の調査中に、returned `RegionToken` の storage origin だけでなく、戻り値と元 local owner の対応、shared borrow からの read、variant condition の source 判定が同じ owner summary 経路に混在していることを確認した。

対応では `StorageOriginTable` に copy origin の source place を保持させ、`read local -> tmp -> return` のような by-value projection return で owner state を複製せず「戻り値が元 owner を保存している」ことを summary / `EndScope` で判定できるようにした。`EndScope` の自動 drop は戻り値配下の origin source と重なる local owner を drop しないため、aggregate identity helper や `TestReport` stdout path が returned owner を失わない。

また `BorrowKind::Shared` は owner / storage origin alias を作らず non-owning raw view だけを伝播するように分離した。共有参照から `str` field を読む処理が元 aggregate の string owner を消費扱いにしないため、borrow projection と free obligation owner の責務が混ざらない。

raw `i32` owner seed は raw owner を消費する関数、または aggregate 内 raw i32 leaf を返す関数に限定した。裸の `i32 -> i32` identity を owner transfer と誤認しない一方、`Boxed { ptr: i32 } -> Boxed` のような aggregate owner return は保持する。variant condition tracking はこの owner seed と分離し、`dealloc(ptr, size)` の `size < 0` のような通常 i32 parameter 条件も caller 側へ伝播する。

この追加補強も `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。ただし `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の最終分離はこの親 issue の残件である。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reinitializes_self_update -- --nocapture`: 5 passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: 8 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_dealloc_through_result_wrapped_str_addr_view -- --nocapture`: passed
