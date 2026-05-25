---
id: ISS-20260525T214844057Z-SHA256-STILL-READS-STALE-VECPUSHERRO-77799ACC
title: "sha256 still reads stale VecPushError vec field after rejected payload split"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-25
updated: 2026-05-26
target: "stdlib/alloc/hash/sha256/{schedule,digest,api}.nepl, stdlib/alloc/collections/vec/mutation/push.nepl"
---

# ISS-20260525T214844057Z-SHA256-STILL-READS-STALE-VECPUSHERRO-77799ACC: sha256 still reads stale VecPushError vec field after rejected payload split

## 概要

SHA256 push failure branches still read VecPushError<T>.vec directly, but VecPushError<T> now carries rejected: VecPushRejected<T> plus error: StdErrorKind.

## 対象

- `stdlib/alloc/hash/sha256/{schedule,digest,api}.nepl, stdlib/alloc/collections/vec/mutation/push.nepl`

## 根拠

- `VecPushError<T>` は 2026-05-22 の rejected payload split 後、`rejected: VecPushRejected<T>` と `error: StdErrorKind` だけを持つ。`vec` field は `VecPushRejected<T>` 側へ移動している。
- `stdlib/alloc/hash/sha256/schedule.nepl`、`digest.nepl`、`api.nepl` は `push<i32>` の `Result::Err e` branch で `field::get e "vec"` を読んでいたため、現行型定義では `type.field.invalid_access` になる。
- `field::get e "error"` は field としては残っているが、public recovery surface としては `vec_push_error_kind<T>(&e)` が用意されている。owner を動かさず diagnostic kind を読む目的には accessor を使うほうが、push failure contract を呼び出し側に漏らさない。
- SHA256 の payload は `i32` なので Copy payload 専用の `vec_push_error_vec<T: Copy>(e)` を使える。non-Copy payload の場合は `vec_push_error_rejected<T>` と `vec_push_rejected_with<T,R>` で rejected item owner も同時に回収する必要がある。

## 問題

SHA256 push failure branches still read VecPushError<T>.vec directly, but VecPushError<T> now carries rejected: VecPushRejected<T> plus error: StdErrorKind.

## 影響

hash doctest compilation fails with type.field.invalid_access, blocking NEPLg2.1 corpus migration verification for the SHA256 path.

## 修正方針

Use vec_push_error_kind<T>(&e) for the diagnostic kind and vec_push_error_vec<T>(e) for Copy payload Vec owner recovery instead of direct field access.

## 検証

2026-05-26:

- `node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js`: passed.
- `rg -n 'field::get e "vec"|field::get e "error"' stdlib/alloc/hash/sha256 -g "*.nepl"`: no matches.
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md --no-tree -o tmp/neplg21-hash-vecpusherror-field.json -j 1 --dist web/dist --assert-io`: `type.field.invalid_access` は出ず、compile timeout after 60000ms。
- `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/tests/hash.n.md --no-tree -o tmp/neplg21-hash-vecpusherror-field-240s.json -j 1 --dist web/dist --assert-io`: compile timeout after 240000ms。
- `target\debug\nepl-cli.exe --check -i tmp\neplg21_sha256_vecpusherror_smoke.neplg2 --target core`: memory allocation failure。これは field access mismatch ではなく、[ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5](./ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5.md) 側の compile-time / memory budget 問題として扱う。
