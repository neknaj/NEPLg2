---
id: ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543
title: "Non-Copy collection payload support needs compiler-issued owner and drop traversal"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
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
- `nodesrc/test_stdlib_collection_cleanup_contract.js` は現行の Copy-only mitigation を横断 policy として固定しているが、non-Copy payload lifecycle の compile-pass coverage はまだ存在しない。

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
