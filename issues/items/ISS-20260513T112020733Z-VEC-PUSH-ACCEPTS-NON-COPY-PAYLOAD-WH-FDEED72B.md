---
id: ISS-20260513T112020733Z-VEC-PUSH-ACCEPTS-NON-COPY-PAYLOAD-WH-FDEED72B
title: "Vec push accepts non-Copy payload while failure paths cannot preserve owners"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: stdlib/alloc/collections/vec/mutation/push.nepl
---

# ISS-20260513T112020733Z-VEC-PUSH-ACCEPTS-NON-COPY-PAYLOAD-WH-FDEED72B: Vec push accepts non-Copy payload while failure paths cannot preserve owners

## 概要

Vec.push<T> returns Result<Vec<T>, StdErrorKind>. For non-Copy payloads, the Err paths cannot return the input Vec owner and item owner, and reallocation failure currently deallocates storage without element drop traversal. This makes owner safety depend on caller discipline instead of type/API proof.

## 対象

- `stdlib/alloc/collections/vec/mutation/push.nepl`

## 根拠

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の Stage D は、non-Copy payload の `push` を `OwnedBuffer<T>` と owner-preserving `PushResult` で扱う方針にしている。
- 現行 `Vec.push<T>` は `Result<Vec<T>, StdErrorKind>` を返すため、`Err` に入力 `Vec` owner と `item` owner を含められない。
- grow 失敗 branch は旧 storage を storage-only dealloc して `Err` を返す。これは Copy payload では安全な縮退だが、non-Copy payload では initialized element traversal と item owner recovery が必要になる。

## 問題

Vec.push<T> returns Result<Vec<T>, StdErrorKind>. For non-Copy payloads, the Err paths cannot return the input Vec owner and item owner, and reallocation failure currently deallocates storage without element drop traversal. This makes owner safety depend on caller discipline instead of type/API proof.

## 影響

Non-Copy payload Vec can leak or lose ownership on allocation failure, and the API contradicts the Stage D OwnedBuffer/PushResult plan.

## 修正方針

Until OwnedBuffer<T> and owner-preserving PushResult are implemented, restrict Vec.push to T: Copy, document the boundary, and add compile-fail regression for Vec<str>. Track full non-Copy push in the parent collection redesign issue.

## 検証

Run focused Vec doctests, a compile-fail regression for non-Copy push, issue check, and relevant source policy checks.

## 修正内容

- `Vec.push` を `.T: Copy` に限定した。
- `Vec.push` の doc comment に、`OwnedBuffer<T>` / owner-preserving `PushResult` が入るまで non-Copy payload を受け入れない理由を明記した。
- `Vec<NonCopyPayload>` の `push` が `type.trait_bound.unsatisfied` で compile-fail になる doctest を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` の source policy を、`Vec.push` が Copy-only であることを監視する形へ更新した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree -o tmp/agent1-vec-push-copy-bound-push.json -j 1 --dist web/dist`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-push-copy-bound-vec.json -j 4 --dist web/dist`: total=33, passed=33

## 親issueとの関係

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` の Stage D 残件のうち、owner-preserving `PushResult` 未導入下で non-Copy payload を受け入れていた入口を閉じた。
- full non-Copy collection support はこの issue では完了扱いにしない。`OwnedBuffer<T>`、initialized prefix、drop traversal、owner-preserving update result は親 issue で継続する。
