---
id: ISS-20260522T231831254Z-VEC-PUSH-FAILURE-RECOVERY-NEEDS-OWNE-5765BBFD
title: "Vec push failure recovery needs owner-preserving rejected eliminator"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/mutation/push.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js, nepl-core/tests/resource_ir.rs"
---

# ISS-20260522T231831254Z-VEC-PUSH-FAILURE-RECOVERY-NEEDS-OWNE-5765BBFD: Vec push failure recovery needs owner-preserving rejected eliminator

## 概要

VecPushError<T> carries VecPushRejected<T> with both the consumed Vec<T> and rejected item T, but unlike replace_drop_old there is no vec_push_rejected_with<T,R> eliminator. Drop payload callers can obtain the aggregate, yet the public API does not provide a single callback boundary that exposes both owners together.

## 対象

- `stdlib/alloc/collections/vec/mutation/push.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

VecPushError<T> carries VecPushRejected<T> with both the consumed Vec<T> and rejected item T, but unlike replace_drop_old there is no vec_push_rejected_with<T,R> eliminator. Drop payload callers can obtain the aggregate, yet the public API does not provide a single callback boundary that exposes both owners together.

## 影響

Drop payload push failure recovery remains incomplete and encourages direct field projection or Vec-only accessors that discard the rejected item owner. This weakens the owner-preserving API discipline needed for non-Copy collection support and self-host data structures.

## 修正方針

Add vec_push_rejected_with<T,R> beside vec_push_error_rejected, document it as the Drop-safe recovery path, keep vec_push_error_vec<T: Copy> Copy-only, and add source policy plus Resource IR owner-obligation regression for DropPayload push Err branch recovery through the callback.

## 検証

Run focused Resource IR owner test, Vec source policy tests, issues check/index, cargo fmt, and git diff --check.

## 2026-05-22 Agent 1 修正

`VecPushRejected<T>` に対して `vec_push_rejected_with<T, R>` を追加し、`push` 失敗時の `Vec<T>` owner と rejected `T` owner を同じ callback へ渡せるようにした。これにより、Drop payload の失敗時 recovery が direct field projection や `vec_push_error_vec<T: Copy>` に寄らず、API 型上で両 owner を同じ control-flow に載せられる。

`vec_push_error_vec<T: Copy>` の説明は、item owner を返さないため Copy-only に留める境界として更新した。`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_collection_cleanup_contract.js` には、`vec_push_rejected_with<T, R>` の callback signature と owner-preserving recovery 分類を監視する回帰を追加した。

Resource IR には `Vec<DropPayload>` の `push` Err branch で `vec_push_error_rejected` から `vec_push_rejected_with` へ渡し、callback 内で rejected item を `Drop::drop` し、返却された Vec を `free` する回帰を追加した。実行 path は成功でも、静的検査は Err branch の owner obligation を検査するため、Drop payload の failure recovery 境界を固定できる。

検証:

- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_vec_push_drop_error_rejected_with_recovers_owners -- --test-threads=1 --exact --nocapture`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/mutation/push.nepl -n 4 --dist web/dist`
