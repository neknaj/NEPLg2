---
id: ISS-20260513T115656872Z-VEC-DATA-OBSERVERS-EXPOSE-RAW-POINTE-674F1AFF
title: "Vec data observers expose raw pointer views for non-Copy payloads"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: stdlib/alloc/collections/vec/access/data.nepl
---

# ISS-20260513T115656872Z-VEC-DATA-OBSERVERS-EXPOSE-RAW-POINTE-674F1AFF: Vec data observers expose raw pointer views for non-Copy payloads

## 概要

Vec.data_ptr<T>, data_mem_ptr<T>, data_len<T>, and vec_storage_mem_ptr<T> return raw address or MemPtr<T> views without requiring T: Copy. After raw element and constructor APIs are Copy-gated, these observers can still expose non-Copy payload storage identity to callers and raw-memory-backed stdlib code.

## 対象

- `stdlib/alloc/collections/vec/access/data.nepl`

## 根拠

- `data_ptr<T>` は `MemPtr<T>` を raw `i32` address へ変換して返す。
- `data_mem_ptr<T>` / `data_len<T>` / `vec_storage_mem_ptr<T>` は `MemPtr<T>` を caller へ返すため、storage identity が safe-looking observer API から外へ出る。
- raw element helper、allocation constructor、push/pop/sort/cleanup は Copy-only に閉じたが、raw data observer が unconstrained のままだと non-Copy payload の storage identity を別経路で取り出せる。
- `len` / `cap` / `is_empty` は storage identity や element pointer を返さないため、今回の Copy-only 対象にはしない。

## 問題

Vec.data_ptr<T>, data_mem_ptr<T>, data_len<T>, and vec_storage_mem_ptr<T> return raw address or MemPtr<T> views without requiring T: Copy. After raw element and constructor APIs are Copy-gated, these observers can still expose non-Copy payload storage identity to callers and raw-memory-backed stdlib code.

## 影響

Unsupported non-Copy Vec payloads can leak raw storage identity and reconstruct shallow load/store paths outside the typed public API boundary. This keeps Resource IR initialized-cell and owner/drop obligations dependent on caller discipline.

## 修正方針

Require T: Copy on raw data observer helpers until OwnedBuffer<T> and borrowed element access APIs exist. Keep len/cap/is_empty generic because they do not expose storage identity. Add compile-fail doctests and source policy coverage.

## 検証

Run Vec source policy checks, focused access/storage doctests, Vec doctests, issue check, and diff check.

## 修正内容

- `vec_storage_mem_ptr` / `data_ptr` / `data_mem_ptr` / `data_len` を `.T: Copy` に限定した。
- `access/data.nepl` と `storage/view.nepl` の doc comment に、non-Copy payload の raw storage identity を `OwnedBuffer<T>` / borrowed element API 導入まで公開しない方針を明記した。
- `Vec<NonCopyPayload>` の `data_ptr` / `data_mem_ptr` / `data_len` が `type.trait_bound.unsatisfied` で compile-fail になる doctest を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_vec_borrowed_observers.js` の source policy を、raw data observer が borrow-based かつ Copy-only であることを監視する形へ更新した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/alloc/collections/vec/storage/view.nepl --no-tree -o tmp/agent1-vec-data-observer-copy-bound-access.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-data-observer-copy-bound-vec.json -j 4 --dist web/dist`: total=42, passed=42
- `git diff --check`: passed

## 親issueとの関係

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` と `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` の Stage D/Stage 6 残件のうち、raw storage identity observer が non-Copy payload に開いていた入口を閉じた。
- full non-Copy borrowed access はこの issue では完了扱いにしない。`OwnedBuffer<T>`、borrow projection、initialized cell state、lifetime/borrow checking と接続した dedicated API が必要である。

## 2026-05-15 follow-up

`ISS-20260514T160255919Z-VEC-DATA-PTR-EXPOSES-RAW-I32-STORAGE-546EA2EB` で `data_ptr<T>` 自体を削除し、`ISS-20260514T161819706Z-VEC-STORAGE-MEMPTR-HELPER-EXPOSES-LO-A9C5BC02` で `vec_storage_mem_ptr<T>` も削除した。したがって、この issue の解決時点で行った `data_ptr<T: Copy>` / `vec_storage_mem_ptr<T: Copy>` 制約は歴史的な中間状態であり、現在の public observer は `data_mem_ptr<T: Copy>` だけである。
