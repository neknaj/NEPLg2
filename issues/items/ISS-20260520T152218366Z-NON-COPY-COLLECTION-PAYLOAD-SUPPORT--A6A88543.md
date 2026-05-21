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
