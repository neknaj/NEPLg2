---
id: ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543
title: "Non-Copy collection payload support needs compiler-issued owner and drop traversal"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-21
target: "stdlib/alloc/collections/**, stdlib/core/mem/**, nepl-core/src/**"
---

# ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543: Non-Copy collection payload support needs compiler-issued owner and drop traversal

## 概要

現行 collection API は、constructor、update、observer、cleanup、owner recovery、storage view の境界を Copy-only に揃えることで、`free` が non-Copy payload の Drop を呼ばず storage-only dealloc へ進む旧バグ入口を閉じている。

これは最終設計ではない。self-host の AST / HIR / diagnostic collection では owning payload を大量に扱うため、non-Copy payload を安全に格納、移動、取り出し、破棄できる collection が必要になる。これを stdlib module ごとの個別証明や raw `MemPtr` helper の復活で扱うのは不適切であり、compiler-issued owner token、InitializedCell / Resource IR の initialized / moved / dropped state、generic proof boundary に接続した設計として実装する。

## 対象

- `stdlib/alloc/collections/**`
- `stdlib/core/mem/**`
- `nepl-core/src/**`
- `stdlib/neplg2/core/resource/**`
- `stdlib/neplg2/core/proof/**`

## 根拠

- Legacy bug issue [ISS-20260425T000000Z-RV-STDLIB-004-91534828](./ISS-20260425T000000Z-RV-STDLIB-004-91534828.md) は、現行 Copy-only 境界の監査により fixed とした。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、`OwnedBuffer<T>` の `initialized_len`、moved slot、drop traversal、compiler-issued owner token が残件であることを明記している。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、Resource IR、owner、initialized cell、borrow、effect を enum / match / proof boundary へ載せる方針を完了条件にしている。
- [ISS-20260520T153639309Z-REGIONTOKEN-CONSTRUCTION-SHARES-GENE-C5BF72D0](./ISS-20260520T153639309Z-REGIONTOKEN-CONSTRUCTION-SHARES-GENE-C5BF72D0.md) で、`region_new` の owner-token construction capability を `mem_ptr_wrap` の generic raw-address alias capability から分離した。これは compiler-issued owner token / initialized cell state へ進む前提整備であり、non-Copy collection support 全体は引き続き open である。
- [ISS-20260520T160659900Z-RAW-FILL-CAN-CREATE-INITIALIZED-RANG-641CBC9C](./ISS-20260520T160659900Z-RAW-FILL-CAN-CREATE-INITIALIZED-RANG-641CBC9C.md) で、Resource IR の raw fill range initialization proof を Copy payload に限定した。non-Copy slot lifecycle は shallow fill ではなく個別 move/drop proof へ接続する。
- [ISS-20260520T161920508Z-RESOURCE-IR-RAW-CELL-LIFECYCLE-TRANS-35AEA479](./ISS-20260520T161920508Z-RESOURCE-IR-RAW-CELL-LIFECYCLE-TRANS-35AEA479.md) で、raw load/store/fill/bulk/realloc/dealloc の initialized/moved/released transition を `RawCellLifecycleEvent` 境界へ集約し、non-Copy raw load 後に stale initialized evidence が残る経路を閉じた。
- [ISS-20260507T050057362Z-RESOURCE-IR-REALLOC-SUCCESS-LOSES-IN-36BCA745](./ISS-20260507T050057362Z-RESOURCE-IR-REALLOC-SUCCESS-LOSES-IN-36BCA745.md) は 2026-05-20 の監査で再オープンした。realloc success path の initialized element range transfer は current main baseline でまだ失敗している。
- [ISS-20260520T165035484Z-BULK-RAW-COPY-DOES-NOT-TRANSFER-INIT-C34AC2CD](./ISS-20260520T165035484Z-BULK-RAW-COPY-DOES-NOT-TRANSFER-INIT-C34AC2CD.md) で、bulk raw copy/move の initialized range transfer を別 issue として分離した。
- [ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D](./ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D.md) で、raw `mem_move` を ownership move に読み替えず、non-Copy payload の Initialize / BorrowRead / MoveOut / Replace / Drop / StorageDealloc を compiler-core の typed slot lifecycle proof boundary として切り出した。
- [ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C](./ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C.md) で、typed slot lifecycle を `Place` ごとの state table として保持し、storage release が live slot を拒否する compiler-core proof state を追加した。
- [ISS-20260520T190336025Z-COLLECTION-SLOT-STATE-LACKS-PATH-MER-3E8FEBA9](./ISS-20260520T190336025Z-COLLECTION-SLOT-STATE-LACKS-PATH-MER-3E8FEBA9.md) で、control-flow merge 後の partial move / partial drop / partial release を `MaybeInitialized` / `MaybeReleased` として typed state に残し、合流後の unsafe reinit / move / drop / dealloc を generic collection slot proof boundary で拒否するようにした。
- [ISS-20260520T192939566Z-RESOURCE-IR-DOES-NOT-CARRY-COLLECTIO-5585A1D7](./ISS-20260520T192939566Z-RESOURCE-IR-DOES-NOT-CARRY-COLLECTIO-5585A1D7.md) で、`ResourceOp::CollectionSlotLifecycle` を追加し、`CollectionSlotStateTable` を initialized checker の branch / loop / match state merge に接続した。これにより slot lifecycle proof は stdlib module allowlist ではなく Resource IR 上の generic typed diagnostic として発火する。
- [ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC](./ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC.md) で、callee 内の slot lifecycle effect を caller の `CollectionSlotStateTable` へ伝播する typed summary を追加した。direct call / function alias indirect call / branch path merge は Resource IR の generic summary program として処理する。
- [ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25](./ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25.md) で、compiler-owned collection slot lifecycle intrinsic から `ResourceOp::CollectionSlotLifecycle` を発行する production lowering / annotation path を追加した。これにより generic producer は存在するが、現行 public collection API はまだ Copy-only であり、non-Copy `Vec<T>` / `OwnedBuffer<T>` API がこの boundary を使う実装は本 issue の残件として扱う。
- [ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2](./ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2.md) で、storage grow / realloc / owner replacement 相当の transition として `ResourceOp::CollectionStorageRelocate` を追加し、old storage prefix 配下の slot lifecycle state を new storage prefix へ generic に rekey するようにした。これは Vec 固有 proof ではなく Resource IR summary でも replay される typed operation である。
- [ISS-20260521T105928164Z-COLLECTION-STORAGE-RELOCATE-LACKS-RA-BE2DCD3E](./ISS-20260521T105928164Z-COLLECTION-STORAGE-RELOCATE-LACKS-RA-BE2DCD3E.md) で、`CollectionStorageRelocate` を raw realloc success から発行された certified raw storage relocation proof に接続した。証明なしの relocate は `StorageRelocateRequiresRawMoveProof` として拒否し、call summary も proof 付き relocate だけを replay するため、slot state rekey が raw movement evidence なしに発火しない。
- [ISS-20260521T112740338Z-COLLECTION-STORAGE-DEALLOC-LACKS-RAW-B31FE6FE](./ISS-20260521T112740338Z-COLLECTION-STORAGE-DEALLOC-LACKS-RAW-B31FE6FE.md) で、`CollectionSlotLifecycleEvent::StorageDealloc` を `RawMemoryOp::Dealloc` 成功から発行された certified raw storage release proof に接続した。証明なしの storage release は `StorageDeallocRequiresRawReleaseProof` として拒否し、summary replay も proof 付き release だけを適用する。
- [ISS-20260520T223249968Z-COLLECTION-SLOT-STATE-DOES-NOT-FOLLO-A808C521](./ISS-20260520T223249968Z-COLLECTION-SLOT-STATE-DOES-NOT-FOLLO-A808C521.md) で、通常の Resource IR value transfer に collection slot state を追従させた。`Move`、aggregate `Construct`、branch/match output、call return summary は stale source prefix に slot state を残さず、移動先 storage dealloc で live slot を検出できる。
- [ISS-20260520T224333208Z-COMPILER-MEMORY-OWNER-TOKEN-AUTHORIT-7FCBF376](./ISS-20260520T224333208Z-COMPILER-MEMORY-OWNER-TOKEN-AUTHORIT-7FCBF376.md) として、`RegionToken` authority が canonical definition identity ではなく stdlib-root 内の名前/形状 evidence に依存している残件を分離した。
- [ISS-20260521T002920171Z-COLLECTION-SLOT-DROP-LIFECYCLE-CAN-E-DB699FC2](./ISS-20260521T002920171Z-COLLECTION-SLOT-DROP-LIFECYCLE-CAN-E-DB699FC2.md) で、`DropInitialized` / `ReplaceDropOld` が droppable payload を実際の drop elaboration なしに `Dropped` 扱いへ進める穴を閉じた。現時点では `ResourceDropRequirement::StateOnly` ではない payload の slot drop event を typed diagnostic として拒否し、state-only cleanup による storage dealloc 成功を防ぐ。完全な non-Copy collection support には、この拒否を compiler-owned slot-drop lowering へ置き換える後続作業が必要である。
- [ISS-20260521T004808159Z-COLLECTION-SLOT-SUMMARIES-DO-NOT-TRA-2AA84347](./ISS-20260521T004808159Z-COLLECTION-SLOT-SUMMARIES-DO-NOT-TRA-2AA84347.md) で、callee が既存の owner parameter をそのまま返す場合の caller slot state transfer を collection slot function summary に追加した。これにより owner-preserving helper / API を跨いでも、caller actual にあった initialized slot state が call output へ generic に追従し、返り値側 storage dealloc で live non-Copy payload を検出できる。
- [ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2](./ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2.md) で、`InitializeEmpty` / `MoveOut` / `ReplaceReturnOld` / `ReplaceDropOld` が payload value-flow evidence なしに non-Copy owner state を進める経路を閉じた。現時点では non-Copy owner-transfer lifecycle event を `OwnerTransferRequiresValueProof` として拒否し、positive support は本 issue の残件として payload consume / materialize proof と compiler-owned slot-drop lowering へ進める。また collection slot の place 正規化は raw value origin ではなく owner-cell canonicalization に限定し、value move 後の storage dealloc が移動先 live slot を見落とさないようにした。
- [ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A](./ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A.md) で、local raw `StoreValue` / `MoveOutLoadedCell` を `RawCellValueFlowFacts` として記録し、non-Copy collection slot owner-transfer event がその fact を消費できる場合だけ state transition を許可するようにした。これにより同一関数内の `raw store -> InitializeEmpty` と `raw load -> MoveOut` は generic Resource IR proof として通る。callee で証明済みの lifecycle を caller へ伝える certified summary proof と compiler-owned slot-drop lowering は引き続き本 issue の残件である。
- [ISS-20260521T025115696Z-COLLECTION-SLOT-SUMMARIES-NEED-CERTI-92379E7C](./ISS-20260521T025115696Z-COLLECTION-SLOT-SUMMARIES-NEED-CERTI-92379E7C.md) で、callee 内で raw value-flow proof から証明済みの collection slot owner-transfer を `CollectionSlotLifecycleSummaryEventProof::OwnerTransferValueFlow` として caller summary replay へ伝えるようにした。これにより同一関数内だけでなく direct call summary 経由でも non-Copy slot initialize / move-out が generic proof として通る。compiler-owned slot-drop lowering は引き続き本 issue の残件である。
- [ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC](./ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC.md) で、raw load された non-Copy slot payload の value origin と `ResourceOp::Drop` / auto-drop を結び、`DropInitialized` / `ReplaceDropOld` が `DropLoadedCell` proof を消費できるようにした。summary proof も owner-transfer と slot-drop を別フィールドで保持し、callee-certified drop proof を caller replay へ伝える。これにより droppable slot cleanup は state-only annotation ではなく actual loaded-value drop proof に接続された。stdlib collection API がこの positive support を使う段階は引き続き本 issue の残件である。
- [ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF](./ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF.md) で、source-level compiler-owned stdlib lowering から得られる raw store/load fact と collection slot lifecycle target が、raw address alias と explicit `[+0]` を跨いでも同じ raw cell として証明されるようにした。手書き Resource IR だけでなく production source path の `raw store -> InitializeEmpty` / `raw load -> MoveOut` が generic proof boundary に接続された。non-zero offset は同じ proof として扱わない。
- [ISS-20260521T054050076Z-SOURCE-LEVEL-DROPPABLE-COLLECTION-SL-7041CED9](./ISS-20260521T054050076Z-SOURCE-LEVEL-DROPPABLE-COLLECTION-SL-7041CED9.md) で、source-level compiler-owned stdlib lowering 経由の `DropInitialized` / `ReplaceDropOld` も raw load された payload の actual drop proof と new payload store proof を generic Resource IR proof boundary で消費することを固定した。raw load だけで droppable slot cleanup を証明する経路、または replacement new store proof なしで `ReplaceDropOld` を進める経路は typed refutation として残る。
- [ISS-20260521T055400560Z-SOURCE-LEVEL-COLLECTION-SLOT-PROOF-L-B5FB8CDA](./ISS-20260521T055400560Z-SOURCE-LEVEL-COLLECTION-SLOT-PROOF-L-B5FB8CDA.md) で、compiler-owned stdlib source path の indexed slot が symbolic offset を使う場合でも、raw store/load proof と collection slot lifecycle state が scalar origin へ正規化されるようにした。同じ `off` の複数 read は同一 slot proof として扱うが、`off + size_of<T>` のように別 offset へ進んだものは typed refutation として残る。
- [ISS-20260521T062752580Z-COLLECTION-SLOT-LIFECYCLE-LOWERING-I-653BBF1A](./ISS-20260521T062752580Z-COLLECTION-SLOT-LIFECYCLE-LOWERING-I-653BBF1A.md) で、collection slot lifecycle producer と storage relocate producer を Resource lowering coverage の typed count に追加した。これにより `ResourceOp::CollectionSlotLifecycle` / `ResourceOp::CollectionStorageRelocate` が lowering から欠落した場合、place coverage だけではなく `CountMismatch` として検出できる。
- [ISS-20260521T064245727Z-COLLECTION-SLOT-BORROWREAD-LACKS-SOU-1999DF36](./ISS-20260521T064245727Z-COLLECTION-SLOT-BORROWREAD-LACKS-SOU-1999DF36.md) で、source-level compiler-owned stdlib path の `BorrowRead` を固定した。initialized slot は BorrowRead 後も initialized のまま raw load / MoveOut proof を通せる一方、MoveOut 後の BorrowRead は `BorrowRead` / `Moved` の typed refutation になる。
- [ISS-20260521T065624831Z-COLLECTION-SLOT-STATE-RETURN-SUMMARY-4591B626](./ISS-20260521T065624831Z-COLLECTION-SLOT-STATE-RETURN-SUMMARY-4591B626.md) で、callee が `Result::Err(storage)` のように owner を enum payload へ包んで返す場合の collection slot return transfer を固定した。return summary は direct parameter return だけでなく、source Resource IR の enum / struct / tuple construct、branch / match value、local forwarding を辿り、caller 側の match bind 後も live slot state を generic proof boundary に残す。stdlib function allowlist や `Result` 固有処理は追加していない。
- [ISS-20260521T071641871Z-COLLECTION-SLOT-RETURN-SUMMARY-MISSE-6D5CDC78](./ISS-20260521T071641871Z-COLLECTION-SLOT-RETURN-SUMMARY-MISSE-6D5CDC78.md) で、wrapper が helper call output をさらに `Result::Err(...)` などの aggregate payload に包んで返す場合も collection slot return transfer を固定した。callee summary の `return_transfers` を wrapper call actual へ instantiate し、raw owner alias canonicalization 後に wrapper parameter-relative source へ戻して target suffix を合成するため、owner-preserving helper composition でも caller の match bind 後に live slot state を失わない。
- [ISS-20260521T072639351Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-40EDCECA](./ISS-20260521T072639351Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-40EDCECA.md) で、同じ nested return transfer を function-alias indirect call 経由でも固定した。`FunctionValue` / `IndirectCall` による higher-order helper composition が direct call と同じ generic summary composition を通ることを regression として保持する。
- [ISS-20260521T073615983Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-0A85EC1F](./ISS-20260521T073615983Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-0A85EC1F.md) で、nested return transfer の indirect callee 解決を block 終端の stale alias table ではなく callsite 直前の `FunctionAliasTable` 再生に切り替えた。indirect call 後に同じ function value place を別関数へ上書きしても、呼び出し時点で証明された callee summary から caller slot state を transfer する。
- [ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31](./ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31.md) で、direct return-transfer の parameter 判定も raw owner alias canonicalization に接続した。callee が parameter storage の owner-cell alias を返す場合でも、caller actual の live slot state が return value へ transfer される。
- [ISS-20260521T075313201Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-7DDCFFFD](./ISS-20260521T075313201Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-7DDCFFFD.md) で、nested return transfer の call argument canonicalization も callsite Resource IR state から解決するようにした。wrapper が raw owner alias 経由で owner-preserving helper を呼んだ後に alias を rebind しても、呼び出し時点の raw alias proof から caller slot state を transfer する。
- [ISS-20260521T080236863Z-COLLECTION-SLOT-RETURN-TRANSFER-IGNO-1F875A73](./ISS-20260521T080236863Z-COLLECTION-SLOT-RETURN-TRANSFER-IGNO-1F875A73.md) で、match arm の bind local を return-transfer 収集時の per-arm state に反映するようにした。callee が `Result::Err(storage)` のような enum payload を match して bound storage を返す場合でも、scrutinee payload から bind local へ raw owner alias と collection slot state が伝播し、caller 側の storage dealloc で live slot を検出できる。
- [ISS-20260521T081047776Z-COLLECTION-SLOT-RETURN-TRANSFER-PROD-1D005BEE](./ISS-20260521T081047776Z-COLLECTION-SLOT-RETURN-TRANSFER-PROD-1D005BEE.md) で、collection slot return-transfer の producer 逆追跡から wildcard arm を削除した。`ResourceOp` の全 variant を明示し、新しい value producer が増えた場合に `cargo check` が更新漏れを検出できるようにした。
- [ISS-20260521T081809182Z-COLLECTION-SLOT-RETURN-TRANSFER-MATC-F51ED8E2](./ISS-20260521T081809182Z-COLLECTION-SLOT-RETURN-TRANSFER-MATC-F51ED8E2.md) で、return-transfer 収集時の match arm entry state を本体 checker の match semantics に近づけた。unreachable arm を除外し、payload bind から raw/cell origin/function/pending realloc/variant state を伝播してから variant refinement を適用する。
- [ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846](./ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846.md) で、branch / match join 後の indirect call summary replay が callee alias と raw/slot state の path correlation を失う設計問題を修正した。`ResourceCheckState` alternatives を Resource IR checker に保持し、linear op / branch value / match value を feasible path ごとに進めてから merge することで、実行不能な callee/state cross product を作らない。
- [ISS-20260521T091611468Z-COLLECTION-SLOT-LIFECYCLE-SUMMARY-CO-1DED6918](./ISS-20260521T091611468Z-COLLECTION-SLOT-LIFECYCLE-SUMMARY-CO-1DED6918.md) で、callee summary event 収集時の `match` arm entry state を return-transfer 収集と本体 checker と共通化した。owned enum payload bind local へ raw alias / slot state / pending state を伝播してから lifecycle event を収集するため、`Result::Err(storage)` などを match した arm 内の storage dealloc も caller summary replay へ generic proof として届く。
- [ISS-20260521T094024431Z-COLLECTION-SLOT-RETURN-SUMMARY-LOSES-5E121C4F](./ISS-20260521T094024431Z-COLLECTION-SLOT-RETURN-SUMMARY-LOSES-5E121C4F.md) で、collection slot return summary の return-transfer / return-slot も path-sensitive summary として保持するようにした。callee alias、raw alias、collection slot state、return transfer は同じ feasible path から導出され、caller replay は call output の slot state だけを pre-call state から return path ごとに評価して merge するため、branch then 側の side effect と else 側の identity return が cross join されない。
- [ISS-20260521T103746552Z-COLLECTION-SLOT-LIFECYCLE-PROOF-CHEC-E31C8D02](./ISS-20260521T103746552Z-COLLECTION-SLOT-LIFECYCLE-PROOF-CHEC-E31C8D02.md) で、collection slot lifecycle event の drop proof / owner-transfer proof 消費を atomic transaction にした。`ReplaceInitialized(DropOldOwner)` のような複合 obligation で片方の proof だけを消費してから rejected event になる経路を閉じ、rejected lifecycle event が proof state を部分変異させないようにした。
- [ISS-20260521T104926514Z-COLLECTION-SLOT-RETURN-VALUE-PRODUCE-51DC87E9](./ISS-20260521T104926514Z-COLLECTION-SLOT-RETURN-VALUE-PRODUCE-51DC87E9.md) で、collection slot return value producer tracing も Branch / Match の `Never` value を除外するようにした。path-state selection と producer tracing が同じ feasible path model を使うため、実行不能 arm の parameter-shaped value から return transfer を作らない。
- [ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B](./ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B.md) として、single-slot `DropInitialized` proof だけでは collection-wide cleanup を positive proof として表せない残件を分離した。dynamic len / symbolic offset / loop traversal は stdlib allowlist ではなく generic Resource IR traversal proof として設計する。
- [ISS-20260521T131208972Z-COLLECTION-SLOT-DROP-TRAVERSAL-DOES--96E42080](./ISS-20260521T131208972Z-COLLECTION-SLOT-DROP-TRAVERSAL-DOES--96E42080.md) で、`collection_slot_drop_traversal<T>` の type arg と `RegionToken<T>` anchor の element type を source typecheck 境界で照合するようにした。これにより traversal proof の expected type が storage owner token と無関係な型として Resource IR へ入る経路を閉じた。
- [ISS-20260521T131741789Z-COLLECTION-SLOT-DROP-TRAVERSAL-LOWER-691CB7BE](./ISS-20260521T131741789Z-COLLECTION-SLOT-DROP-TRAVERSAL-LOWER-691CB7BE.md) で、source-level stdlib intrinsic から `ResourceOp::CollectionSlotDropTraversal` が生成され、その producer が欠落した場合に lowering coverage の `CountMismatch` で検出されることを固定した。
- [ISS-20260521T132527293Z-SOURCE-LEVEL-COLLECTION-DROP-TRAVERS-24EF497F](./ISS-20260521T132527293Z-SOURCE-LEVEL-COLLECTION-DROP-TRAVERS-24EF497F.md) で、compiler-owned stdlib source から raw store / raw load / actual drop / `collection_slot_drop_traversal` / raw dealloc / `collection_slot_storage_dealloc` が一続きの generic proof path として通ることを固定した。これは `Vec` や specific helper 名の allowlist ではなく、Resource IR の typed op と alias-aware proof boundary による end-to-end regression である。
- [ISS-20260521T133925178Z-COLLECTION-SLOT-OWNER-TOKEN-INTRINSI-79B2D12D](./ISS-20260521T133925178Z-COLLECTION-SLOT-OWNER-TOKEN-INTRINSI-79B2D12D.md) で、collection slot lifecycle marker の owner-token anchor を `OwnerTokenAnchorAccess::{Borrowed, ByValue}` で区別し、`RegionToken<T>` by-value anchor を typecheck 境界で拒否した。proof marker が storage owner を move する曖昧な境界を閉じ、`&RegionToken<T>` anchor だけを compiler-owned lifecycle proof に使う。
- [ISS-20260521T135336921Z-COLLECTION-SLOT-LIFECYCLE-TYPE-CHECK-126AADF5](./ISS-20260521T135336921Z-COLLECTION-SLOT-LIFECYCLE-TYPE-CHECK-126AADF5.md) で、collection slot lifecycle state transition の payload type check を exact `TypeId` 比較から `TypeCtx` / `type_pattern_matches` に接続した。また `CollectionSlotStateTable` の slot identity を payload `Place.ty` から分離し、root + projections で同じ storage + offset の slot を扱うようにした。source-level `ReplaceReturnOld` も raw load / raw store proof による positive regression として固定した。
- [ISS-20260521T145809160Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-3D619183](./ISS-20260521T145809160Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-3D619183.md) で、symbolic / unknown offset の collection slot を explicit finite slot として drop traversal / storage dealloc に使う経路を閉じた。dynamic `initialized_len` 全体の cleanup は typed range proof が別途必要であり、この修正は証明不足の範囲を証明済みとして扱わない安全側の境界である。
- [ISS-20260521T151802959Z-COLLECTION-SLOT-DROP-TRAVERSAL-LACKS-B557D89A](./ISS-20260521T151802959Z-COLLECTION-SLOT-DROP-TRAVERSAL-LACKS-B557D89A.md) で、`collection_slot_drop_traversal<T>` と `ResourceOp::CollectionSlotDropTraversal` に typed `initialized_count` operand を追加した。known slot は source-derived i32 relation facts で count 範囲内であることを検査し、summary build / replay も count を捨てない。symbolic slot の positive forall proof は引き続き本 issue の残件である。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` は現行の Copy-only mitigation を横断 policy として固定しているが、non-Copy payload lifecycle の compile-pass coverage はまだ存在しない。
- [ISS-20260521T041844157Z-COLLECTION-SLOT-LIFECYCLE-CAPABILITY-1DE754DB](./ISS-20260521T041844157Z-COLLECTION-SLOT-LIFECYCLE-CAPABILITY-1DE754DB.md) で、collection slot lifecycle intrinsic の source capability を intrinsic expression 全体ではなく intrinsic name literal の exact span に結び付け、同一 stdlib file 内の unrelated span や configured stdlib 外の同一 source text へ権限が広がらないことを回帰テスト化した。これにより generic Resource IR proof boundary の入口が file-wide / user-source authority へ退行しない。
- [ISS-20260521T043444492Z-COLLECTION-SLOT-LIFECYCLE-INTRINSIC--58368836](./ISS-20260521T043444492Z-COLLECTION-SLOT-LIFECYCLE-INTRINSIC--58368836.md) で、collection slot lifecycle intrinsic を包む stdlib public function / public alias target には source capability を発行しないようにした。これにより compiler-owned internal lowering helper は維持しつつ、lifecycle intrinsic の入口が public wrapper / re-export に退行しない。

## 問題

現状の安全性は「non-Copy payload collection を許可しない」ことで成立している。これは旧バグの再発防止としては正しいが、self-host compiler の中核では長期的に不足する。

不足しているもの:

- storage owner を user source から forge できない compiler-issued owner token。
- initialized / moved / dropped slot を `len` や null pointer ではなく typed state として保持する `InitializedCell` / Resource IR boundary。
- Copy read、borrow read、move-out、replace、container drop、storage-only dealloc の API 分離。
- fallible update の失敗時に collection owner と item owner を型で返す owner-preserving result。
- collection module ごとの個別 proof ではなく、generic proof solver に fact / obligation / evidence / refutation として載る static check。

## 影響

self-host の parser 後半、HIR、typecheck、Resource IR、diagnostic aggregation は owning payload を collection に置く必要がある。現行 Copy-only subset のまま進めると、次のどちらかになる。

- owning payload を避けるために不自然な ID / arena / manual cleanup を増やし、静的検査の設計を複雑化させる。
- Copy-only 境界を緩めて non-Copy payload を入れ、shallow copy、leak、double drop、storage-only dealloc を再導入する。

どちらも開発方針に反する。したがって non-Copy collection support は、stdlib の便利機能ではなく self-host 前の memory safety / type safety 基盤として扱う。

## 修正方針

次の順で実装する。

1. `MemPtr<T>` を non-owning view に限定したまま、storage owner は compiler-issued token またはそれに準じる forge 不能な wrapper へ移す。
2. `OwnedBuffer<T>` に live length、capacity、storage owner、initialized slot state を接続し、`len == initialized_len` 前提の Copy-only invariant から脱却する。
3. Resource IR lowering が collection operation を cell event / owner event / borrow event として generic proof boundary へ渡す。
4. `Vec<T>` から、borrowed observer、owned move-out、replace、clear/drop/free、fallible push/grow を owner discipline ごとに分ける。
5. derived collection は `Vec<Option<T>>` 依存を見直し、slot state と collection owner を同じ proof model に接続する。
6. Copy-only source policy は段階的に「unsupported non-Copy を拒否する policy」から「non-Copy support が Resource IR proof 経由であることを要求する policy」へ更新する。

禁止事項:

- raw `MemPtr` を owner field や public mutation authority として復活させない。
- stdlib module 名や関数名の allowlist で non-Copy payload を個別許可しない。
- collection family ごとに個別 proof engine を増やさない。
- `bool` / 文字列 / 数値 sentinel で initialized / moved / dropped state を表さない。

## 検証

実装時に次を追加する。

- non-Copy owner payload の `Vec` lifecycle compile-pass: construct、push、borrow observe、move-out、replace、drop/free。
- `List` / map / set 系で owner-preserving update と error recovery が item owner を失わない compile-pass。
- shallow raw copy、double move、double drop、drop after move、storage-only dealloc of initialized non-Copy cells の compile_fail。
- source policy: non-Copy collection support が generic Resource IR owner/cell proof boundary を迂回していないこと。
- performance regression: initialized slot tracking が collection traversal で組み合わせ爆発しないこと。
