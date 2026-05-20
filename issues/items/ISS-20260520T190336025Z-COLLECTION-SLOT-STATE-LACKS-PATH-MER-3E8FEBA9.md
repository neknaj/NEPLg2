---
id: ISS-20260520T190336025Z-COLLECTION-SLOT-STATE-LACKS-PATH-MER-3E8FEBA9
title: "Collection slot state lacks path-merge uncertainty"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/collection_slot_state_table.rs"
---

# ISS-20260520T190336025Z-COLLECTION-SLOT-STATE-LACKS-PATH-MER-3E8FEBA9: Collection slot state lacks path-merge uncertainty

## 概要

Collection slot state は definite な initialized / moved / dropped / released state だけを表せたが、片方の control-flow path では initialized、別 path では vacant または released という不確実性を表せなかった。これを Resource IR の if / match / loop merge へ接続すると、partial move 後の move/drop/dealloc を unsafe に許可するか、typed proof boundary の外側で ad hoc に拒否する実装になる。

## 対象

- `nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/collection_slot_state_table.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、Resource IR の initialized / moved / dropped state を enum / match に載せ、検査の正確性を型で維持することを Stage 6 の完了条件としている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、collection slot lifecycle を compiler-core の generic proof boundary で扱う方針を定めている。
- [ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C](./ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C.md) で `CollectionSlotStateTable` を追加したが、path merge の不確実性は未実装だった。

## 問題

`CollectionSlotStateTable` が control-flow merge 後の不確実性を表せないため、次のような状態を安全に扱えない。

- 片方の分岐だけで non-Copy slot を move-out した。
- 片方の分岐だけで non-Copy slot を drop した。
- 片方の分岐だけで backing storage を release した。
- 片方の分岐では initialized slot が残り、別分岐では slot が vacant になった。

この状態を definite な `Moved` / `Dropped` / `Released` へ潰すと後続の reinit / move / drop / storage dealloc を誤って許可する。逆に各 caller で個別拒否すると、stdlib module ごとの proof rule が増え、generic proof solver へ載せる方針から外れる。

## 影響

Non-Copy collection payload support を if / match / loop を含む Resource IR へ sound に接続できない。特に self-host compiler の AST / HIR / diagnostic collection では owning payload を多用するため、このままでは collection を Copy-only に閉じ込めるか、unsafe な shallow owner operation を再導入することになる。

## 修正方針

次を実装した。

- `CollectionSlotState::MaybeInitialized(Option<TypeId>)` を追加し、path merge 後に live slot が残る可能性を型で保持する。
- `CollectionSlotState::MaybeReleased` を追加し、backing storage release が path-dependent になった状態を型で保持する。
- `MaybeLiveSlotOverwrite` / `MaybeLiveSlotDuringStorageDealloc` refutation を追加し、reinit / storage release が partial live slot を潰さないようにした。
- `CollectionSlotStateTable::merge_paths` を追加し、branch / match / loop merge が slot state と storage release state を generic table として合流できるようにした。
- lifecycle / state table / merge tests を分割し、責務分割の policy test に新 module を登録した。

## 検証

- `cargo test -p nepl-core collection_slot -- --test-threads=1`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
