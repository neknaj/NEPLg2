---
id: ISS-20260427T041629340Z-VEC-PUSH-ERR-LOSES-OWNER-7D2E9F43
title: "Vec push が grow 失敗時に旧 buffer owner を失う"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/vec.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, tests/stdlib/vec_collections.n.md"
source: "ISS-20260427T041147635Z-FS-VEC-PUSH-UNWRAP-TRAPS-4A80E8C1"
---

# ISS-20260427T041629340Z-VEC-PUSH-ERR-LOSES-OWNER-7D2E9F43: Vec push が grow 失敗時に旧 buffer owner を失う

## 概要

`Vec.push` は `Vec<.T>` を値で受け取る consuming API だが、容量不足で `realloc_ptr` が失敗した場合に `Result::Err` だけを返し、入力 `Vec` の `data` buffer を返さず解放もしない。

## 対象

- `stdlib/alloc/collections/vec.nepl`
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `tests/stdlib/vec_collections.n.md`

## 根拠

- `push` は `Vec<.T>` を所有値として受け取り、成功時は次の `Vec` owner を返す。
- grow 失敗時は `Result::Err StdErrorKind::OutOfMemory` だけを返すため、caller は旧 `Vec` owner を回収できない。
- `realloc_ptr` は grow 失敗時に旧 buffer を保持する設計なので、`push` が明示的に解放しないと storage が失われる。

## 問題

allocation failure が発生したとき、collection API の所有権境界が `Err` だけになり、旧 buffer が leak する。
`std/fs` のように `v::push` 失敗を `Result` として扱う修正でも、旧 `Vec` owner が失われる構造が残る。

## 影響

self-host parser / loader / diagnostic collection が大きい入力で allocation pressure に遭遇した場合、エラーを返しながら内部 buffer を失い、後続処理でメモリ不足を悪化させる。

## 修正方針

`push` の grow 失敗時に、consumed owner である旧 `data` buffer を `dealloc_raw` で解放してから `Err(OutOfMemory)` を返す。
source policy regression で、`push` の `realloc_ptr` failure branch が旧 buffer を解放することを固定する。

## 解決内容

- `Vec.push` の `realloc_ptr` failure branch で `dealloc_raw mem_ptr_addr v_data old_bytes` を呼び、consumed owner の旧 buffer を解放してから `Err(OutOfMemory)` を返すようにした。
- `push` の注意書きに、容量拡張失敗時も入力 `Vec` owner は関数が消費し、旧 buffer を解放することを明記した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、`push` の grow failure branch が旧 buffer を解放することを確認する source policy regression を追加した。

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-push-owner-docs.json -j 1`: 39/39 passed
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-push-owner-focused.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-vec-push-owner.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-vec-push-owner.json -j 4`: 418/418 passed
