---
id: ISS-20260522T211933733Z-VEC-REPLACE-MUST-PROVE-SLOT-REPLACEM-3686CB53
title: "Vec replace must prove slot replacement through Resource IR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/mutation/replace.nepl, stdlib/alloc/collections/vec/types.nepl, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js"
---

# ISS-20260522T211933733Z-VEC-REPLACE-MUST-PROVE-SLOT-REPLACEM-3686CB53: Vec replace must prove slot replacement through Resource IR

## 概要

`Vec.replace<T: Copy>` は initialized slot を public body から raw `store` で直接上書きしており、Resource IR の collection slot replacement lifecycle proof に接続されていなかった。また、borrowed no-op API のまま `.T: Copy` を外すと、範囲外失敗時の new item owner と、成功時の old slot cleanup discipline が API 型から消える。

## 対象

- `stdlib/alloc/collections/vec/mutation/replace.nepl, stdlib/alloc/collections/vec/types.nepl, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js`

## 根拠

- `Vec.push` / `Vec.drop_last` / `Vec.pop` は private helper 内で raw payload operation と collection slot lifecycle marker を同じ proof boundary に閉じる設計へ移行済みだったが、`replace` だけは public body に raw store が残っていた。
- Resource IR の `CollectionSlotLifecycleEvent::ReplaceInitialized` は generic proof として存在するため、stdlib module allowlist ではなく、raw load/store/drop evidence を compiler が検査できる source boundary へ接続する必要があった。
- non-Copy payload の失敗時 recovery は、`Vec<T>` owner と rejected `item: T` owner を同時に返さない限り、所有権の行方を型で追跡できない。

## 問題

- Copy payload の `replace<T: Copy>` は、initialized slot を raw store しても slot state を `ReplaceInitialized` として証明していなかった。
- Drop payload の置換を borrowed no-op API に拡張すると、old payload の actual `Drop::drop` proof と new payload store proof を同じ boundary で要求できない。
- 範囲外や invariant failure で `item` owner を返さない API では、non-Copy payload support が型安全にならない。
- direct public `field::get` による owner-backed rejected aggregate の分解は、caller 側で片方の owner だけを取り出す設計に傾きやすく、回収漏れを検査しにくい。

## 影響

Non-Copy collection replacement would require stdlib-specific exceptions or would lose old/new owner obligations. That blocks self-host collection payload support and weakens the generic Resource IR proof model.

## 修正方針

- Copy raw replacement は private `vec_replace_copy_initialized_slot<T: Copy>` へ移し、raw load witness、raw store、`collection_slot_replace_drop_old` marker を同じ proof boundary に置く。
- Drop payload は owner-consuming `replace_drop_old<T: Drop>(Vec<T>, i32, T) -> Result<Vec<T>, VecReplaceError<T>>` として追加し、old payload の raw load、actual `Drop::drop`、new store、replacement marker を private helper に閉じる。
- 失敗時は `VecReplaceRejected<T>` / `VecReplaceError<T>` に `Vec<T>` owner と rejected `item: T` owner をまとめて返す。
- `VecReplaceRejected<T>` の public 分解は direct field projection ではなく、`vec_replace_rejected_with<T, R>` が callback へ両 owner を同時に渡す eliminator とする。

## 対応結果

- `stdlib/alloc/collections/vec/types.nepl` に `VecReplaceRejected<T>` と `VecReplaceError<T>` を追加した。
- `stdlib/alloc/collections/vec/mutation/replace.nepl` の `replace<T: Copy>` は `VecStorageInvariant` を確認し、raw pointer operation / marker を public body から排除した。
- `replace_drop_old<T: Drop>` を追加し、成功時は old slot を drop して new item を store した `Vec<T>` を返し、失敗時は `VecReplaceError<T>` で `Vec<T>` と `item` を回収できるようにした。
- `vec_replace_error_rejected<T>`、`vec_replace_rejected_with<T, R>`、Copy payload 用の `vec_replace_error_vec<T: Copy>` を追加し、non-Copy recovery と legacy Copy convenience accessor を型で分けた。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_collection_cleanup_contract.js` を、public surface が raw/marker を持たず、private proof helper が load/store/drop/marker evidence を持つことを検査する形へ更新した。
- Resource IR regression を追加し、Copy replace、Drop replace success、Drop replace failure recovery の 3 経路を実 stdlib source から確認した。

## 関連文書

- [static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [Non-Copy collection payload support umbrella](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)

## 検証

- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_copy_replace_emits_replace_lifecycle -- --test-threads=1 --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_replace_drop_old_closes_old_slot -- --test-threads=1 --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_replace_drop_old_failure_recovers_owners -- --test-threads=1 --exact --nocapture`: passed
- `trunk build`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=180000 node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/replace.nepl --no-tree -o tmp/vec-replace-lifecycle-doctest.json -j 1 --dist web/dist`: 2 passed
