---
id: ISS-20260521T000647693Z-COLLECTION-SLOT-LIFECYCLE-LACKS-STDL-4001F199
title: "Collection slot lifecycle lacks stdlib source integration regression"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/initialized_collection_slot.rs, nepl-core/src/resource/collection_slot_summary_build.rs, nepl-core/src/resource/collection_slot_summary_apply.rs, nepl-core/tests/resource_ir.rs
---

# ISS-20260521T000647693Z-COLLECTION-SLOT-LIFECYCLE-LACKS-STDL-4001F199: Collection slot lifecycle lacks stdlib source integration regression

## 概要

Collection slot lifecycle checks had strong manual ResourceOp coverage, but no focused regression proving that compiler-owned stdlib source intrinsics using canonical MemPtr/RegionToken lower into Resource IR and produce the same typed refutations.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- stdlib source から `#intrinsic "collection_slot_*"` を下げると、同じ `&RegionToken<T>` を複数回読むたびに `tmp1`, `tmp4`, `tmp7` のような別 temporary root を持つ `Place` が生成されていた。
- raw-address alias table はこれらを同一 storage 由来として把握していたが、collection slot lifecycle checker は alias 正規化前の `Place` を直接 `CollectionSlotStateTable` に渡していた。
- その結果、source-level の `initialize_empty -> move_out -> move_out` が「Moved への二重 move」ではなく、各 temporary に対する独立した「Uninitialized からの move」として扱われた。

## 問題

Collection slot lifecycle checks had strong manual ResourceOp coverage, but no focused regression proving that compiler-owned stdlib source intrinsics using canonical MemPtr/RegionToken lower into Resource IR and produce the same typed refutations.

## 影響

A future lowering, source capability, or canonical compiler-memory change could keep manual Resource IR tests green while breaking the actual stdlib source path needed for non-Copy collection payload support.

## 修正方針

- `CollectionSlotLifecycle` / `CollectionStorageRelocate` を適用する境界で `RawCellAddressAliases::canonicalize` を必ず通し、同一 storage 由来の temporary root を canonical place に統合してから `CollectionSlotStateTable` に渡す。
- 関数 summary の構築時も raw-address alias を用いて collection slot target を正規化し、callee 内で temporary 経由になった slot 操作を parameter-relative summary として保存する。
- 関数 summary の replay 時も caller 側の raw-address alias で正規化し、callee summary が caller の canonical storage state に適用されるようにする。
- stdlib source path から canonical `core/mem/types` を import して Resource IR まで下げる回帰テストを追加し、直接 intrinsic と summary 経由の両方で typed refutation を確認する。

## 修正内容

- `nepl-core/src/resource/initialized_collection_slot.rs`
  - collection slot lifecycle / storage relocate を raw-address alias 正規化付きで適用する入口を追加した。
- `nepl-core/src/resource/initialized.rs`
  - 直接の `ResourceOp::CollectionSlotLifecycle` / `CollectionStorageRelocate` に alias 正規化付き入口を使うようにした。
- `nepl-core/src/resource/collection_slot_summary_build.rs`
  - summary 生成時に target / storage を alias 正規化してから parameter-relative suffix に変換するようにした。
- `nepl-core/src/resource/collection_slot_summary_apply.rs`
  - summary replay 時に caller 側 raw-address alias で target / storage を正規化するようにした。
- `nepl-core/tests/resource_ir.rs`
  - source-level intrinsic の二重 move 回帰テストを追加した。
  - callee summary 経由の二重 move 回帰テストを追加した。

## 検証

- `cargo test -p nepl-core resource_ir_collection_slot_source_ --test resource_ir -- --test-threads=1`
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
