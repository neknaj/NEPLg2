---
id: ISS-20260523T003359949Z-VEC-NON-COPY-TRANSFORMS-NEED-SCOPED--CEE50B61
title: "Vec non-Copy transforms need scoped borrowed slot observer"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-23
updated: 2026-05-23
target: "stdlib/alloc/collections/vec/access/borrow.nepl, stdlib/alloc/collections/vec/access.nepl, nepl-core/src/resource/**, nepl-core/src/typecheck/**, nepl-core/src/source_capability/**, nepl-core/src/codegen_*"
---

# ISS-20260523T003359949Z-VEC-NON-COPY-TRANSFORMS-NEED-SCOPED--CEE50B61: Vec non-Copy transforms need scoped borrowed slot observer

## 概要

`filter` / `partition` / `take_while` / `drop_while` 系の transform は現状 `(.T)->bool` の predicate を取り、要素取得は `get<T: Copy>` に依存している。ここから単に `.T: Copy` を外すと、predicate 呼び出しが non-Copy payload の owner を値渡しで消費し、要素を出力 Vec へ移すか drop する前に slot lifecycle が破綻する。

Resource IR には `CollectionSlotLifecycleEvent::BorrowRead` があり、initialized slot を維持したまま読み取り借用の precondition を検査できる。しかし stdlib/compiler には、証明済み Vec storage slot から `&T` を安全に materialize し、raw `MemPtr<T>` を返さず、borrow を callback scope の外へ逃がさない observer 境界がまだない。

## 対象

- `stdlib/alloc/collections/vec/access/data.nepl, stdlib/alloc/collections/vec/query/get.nepl, nepl-core/src/intrinsic_kinds.rs, nepl-core/src/resource`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy collection を raw `mem_move` や stdlib module allowlist ではなく、`InitializeEmpty` / `BorrowRead` / `MoveOut` / `ReplaceInitialized` / `DropInitialized` / `StorageDealloc` の generic Resource IR proof boundary へ載せる方針を定めている。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、静的検査の正確性を維持するため、数値・文字列・個別関数名ではなく enum / match / typed proof boundary で検査を構成することを Stage 6 の完了条件にしている。
- [ISS-20260521T064245727Z-COLLECTION-SLOT-BORROWREAD-LACKS-SOU-1999DF36](./ISS-20260521T064245727Z-COLLECTION-SLOT-BORROWREAD-LACKS-SOU-1999DF36.md) で、source-level `BorrowRead` は initialized slot を保持し、MoveOut 後の BorrowRead を typed refutation として拒否することまで固定済みである。未解決なのは、その汎用 proof を実 `Vec<T>` の public borrowed observer API に安全に接続する設計である。
- `data_mem_view<T: Copy>(&Vec<T>) -> VecDataView<T>` は Copy raw access proof を返す境界であり、non-Copy observer のために `.T: Copy` を外すと raw pointer escape と payload lifecycle proof の混同を再導入する。

## 問題

`Vec<T>` の non-Copy transform を実装するには、判定だけを行う段階では payload owner を動かさず、`&T` を借用して predicate を呼ぶ必要がある。現行の `(.T)->bool` predicate はこの責務に合っておらず、`get<T: Copy>` は値コピー observer なので non-Copy payload へ拡張できない。

また、`VecDataView<T>` や `MemPtr<T>` を public observer として開くと、caller が slot borrow の寿命や initialized state を compiler の検査外で扱えるため、Resource IR の `BorrowRead` を通した意味がなくなる。必要なのは、`VecStorageInvariant` で storage/extent を証明し、slot offset の initialized state を `BorrowRead` で検査し、その結果得た `&T` を callback scope 内だけで使わせる単一の汎用境界である。

## 影響

この境界がないままでは、non-Copy `filter` / `partition` / `take_while` / `drop_while` を sound に実装できない。by-value predicate のまま進めると、shallow copy、隠れた move-out、stdlib 固有 allowlist のいずれかへ寄り、静的検査の設計方針に反する。self-host 側も owning payload collection を避けるために不自然な ID / arena / manual cleanup を増やすことになる。

## 修正方針

non-Copy predicate transform より先に、Vec slot borrowed observer boundary を設計・実装する。

- public API は callback scope 型にする。概念上は `Vec<T>` owner または `&Vec<T>`、index、`on_some(&T)`、`on_none` を受け取り、`&T` が戻り値や owner aggregate field として外へ逃げない形にする。
- implementation boundary は `VecStorageInvariant` と `CollectionSlotLifecycleEvent::BorrowRead` を使う。`data_mem_view<T: Copy>` の Copy 制約は緩めない。
- `BorrowRead` から `&T` を materialize する compiler-owned source boundary は、raw `load<T>` や raw `MemPtr<T>` public exposure と別概念にする。
- `filter` / `partition` / prefix transform ごとの proof engine は作らない。全 transform は同じ borrowed observer と generic Resource IR borrow/lifetime/slot-state proof を使う。
- non-Copy transform の順序は、まず borrowed observer、次に move-out / output slot initialization / rollback / drop traversal を伴う `map` / prefix、最後に左右 2 本の output owner を扱う `partition` とする。

## 検証

focused regression は次を満たす。

- non-Copy payload を持つ `Vec<T>` の borrowed observer が compile-pass する。
- MoveOut 後の slot borrow が generic Resource IR checker により拒否される。
- source policy が non-Copy observer の raw `MemPtr<T>` / `VecDataView<T>` 露出を拒否する。
- 既存の Copy `get` / `filter` / `partition` doctest が引き続き有効である。
- predicate-style transform の `.T: Copy` 制約を外す前に、by-value `(.T)->bool` ではなく borrowed observer boundary を通ることを検査する。

## 解決内容

- `collection_slot_borrow_ref<T>(&RegionToken<T>, i32) -> &T` を `CollectionSlotBorrowPrimitive::BorrowRef` として typed enum に追加し、文字列だけの ad hoc intrinsic 判定にしない形で primitive / source capability / typecheck / Resource IR / codegen に接続した。
- `collection_slot_borrow_ref` は compiler-owned stdlib source evidence を持つ private implementation boundary だけで使える。public stdlib callable surface や user source からは拒否され、raw `MemPtr<T>` anchor も受け付けない。
- Resource IR lowering は同じ typed slot に対して `CollectionSlotLifecycleEvent::BorrowRead` と `ResourceOp::Borrow { kind: Shared }` を発行する。`BorrowRead` は initialized slot state を保持し、MoveOut 後の borrow は generic slot-state checker が拒否する。
- slot 由来の `&T` は `StorageOffset` projection を持つ scoped observer borrow として扱い、function return escape を borrow checker が拒否する。
- stdlib 側に `alloc/collections/vec/access/borrow.nepl` を追加し、public API は `borrow_at_predicate_or<T>(&Vec<T>, i32, (&T)->bool, bool)->bool` に絞った。任意 `R` 戻り値 observer は、borrow escape / owner summary の性能と soundness を別途固めるまで公開しない。
- `VecDataView<T>` / `MemPtr<T>` を non-Copy observer の public surface へ出さず、`VecStorageInvariant` と typed `BorrowRead` proof から callback scope 内の `&T` だけを materialize する設計にした。

## 検証結果

- `cargo test -q -p nepl-core collection_slot_borrow -- --nocapture`: pass
- `cargo test -q -p nepl-core resource_ir_collection_slot_borrow_ref -- --nocapture`: pass
- `cargo test -q -p nepl-core collection_slot_borrow_intrinsic_lowers_state_proof_and_shared_borrow -- --nocapture`: pass
- `cargo test -q -p nepl-core resource_ir_vec_borrow_at_predicate_or -- --nocapture`: pass（約 83.74s。既存 `Vec<DropPayload>.push -> free` regression と同程度の Resource IR summary cost）
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib\alloc\collections\vec\access\borrow.nepl --no-tree -o tmp\agent1-vec-borrow-doctest.json -j 1 --dist web\dist --assert-io`: pass

## 関連する新規 issue

- [ISS-20260523T014105503Z-VEC-DROPPAYLOAD-RESOURCE-IR-SUMMARY--873A5BCD](./ISS-20260523T014105503Z-VEC-DROPPAYLOAD-RESOURCE-IR-SUMMARY--873A5BCD.md): 実 stdlib `Vec<DropPayload>` の focused Resource IR regression が 80 秒級になる performance 残件。`collection_slot_borrow_ref` 個別の失敗ではなく、既存 `push -> free` と同じ `resource_initialized_i32_scalar_summaries` / `resource_initialized_collection_slot_summaries` の計算量問題として分離した。
