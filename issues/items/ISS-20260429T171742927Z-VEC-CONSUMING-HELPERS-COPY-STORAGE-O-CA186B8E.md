---
id: ISS-20260429T171742927Z-VEC-CONSUMING-HELPERS-COPY-STORAGE-O-CA186B8E
title: "Vec consuming helpers copy storage owner instead of transferring it"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/collections/vec.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, tests/compiler/list_dot_map.n.md, tests/compiler/overload.n.md, tests/compiler/overload_nested_generic_push.n.md"
---

# ISS-20260429T171742927Z-VEC-CONSUMING-HELPERS-COPY-STORAGE-O-CA186B8E: Vec consuming helpers copy storage owner instead of transferring it

## 概要

Vec consuming helpers such as push and map read the owning data field through field::get_ref and then construct a returned Vec from that copy. Resource IR can no longer prove that the caller's input Vec owner or the temporary output Vec owner was consumed exactly once.

## 対象

- `stdlib/alloc/collections/vec.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, tests/compiler/list_dot_map.n.md`

## 根拠

- `tests/compiler/list_dot_map.n.md::doctest#4` は修正前、`push` 後の `xs0` / `xs1` / `xs2` と `map` 内の `alloc_r.Ok.data` / `out0.data` を owner obligation leak として報告した。
- `push` は `Vec<.T>` を consuming API として受け取るが、`data` を `field::get_ref &v "data"` で copy 的に読み、戻り値の `Vec` をその copy から構築していた。
- `map` / `filter` / `partition` / `take_while` / `drop_while` も、入力 storage と確保済み出力 storage の owner を借用読みの alias のまま扱い、成功/失敗 path で入力 storage を閉じていなかった。
- 既存 source policy は `len` / `cap` の誤 move 防止と `data` owner transfer を区別しておらず、`data` の正しい owner transfer まで禁止していた。

## 問題

Vec consuming helpers such as push and map read the owning data field through field::get_ref and then construct a returned Vec from that copy. Resource IR can no longer prove that the caller's input Vec owner or the temporary output Vec owner was consumed exactly once.

## 影響

Strict owner checking reports leaks for xs0/xs1/xs2 and map out0 in compiler fixtures. Keeping the copy-alias pattern would require weakening Resource IR or accepting ambiguous Vec storage ownership, which blocks self-host collection use.

## 修正方針

Keep len/cap as borrowed Copy header reads, but make helpers that return or deallocate the storage consume the owning data field at the point ownership leaves the input/output Vec. Update the Vec source policy so it forbids accidental len/cap moves while allowing intentional data owner transfer with tests.

## 検証

Run the Vec source policy, focused list_dot_map and overload fixtures, Vec doctests, compiler suite, and issues check.

## 修正内容

- `push` / `pop` / `clear` / `free` は `len` / `cap` を `field::get_ref` で Copy read し、`data` owner は `field::get` で明示的に取り出して戻り値または dealloc/realloc へ渡す形にした。
- `map` / `filter` / `partition` / `take_while` / `drop_while` は入力 Vec の `cap` と `data` owner を取得し、出力確保失敗 path と成功 path の両方で入力 storage を `dealloc_raw` するようにした。
- transform 系の `with_capacity` 結果は不要な `alloc_r` / `alloc_left` / `alloc_right` 中間 owner local を作らず、直接 `match with_capacity` する形へ戻した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は `len` / `cap` の direct `field::get` を禁止しつつ、`push` / `pop` / `clear` / `free` / `map` が `data` owner を明示的に移すことを確認する policy に更新した。
- compiler fixtures の Vec 使用例は、検査目的に関係しない by-value observation leak を避けるため、`len_ref` / `get_ref` と `free` を使う現在の所有権契約へ更新した。

## 検証結果

- `trunk build`: pass
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/compiler/list_dot_map.n.md --no-tree -o tmp/vec-owner-transfer-list-dot-map-after-fixtures.json -j 1 --dist web/dist`: total=4, passed=3, failed=1。残りは `list::map` の既存 owner issue (`ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB`)。
- `node nodesrc/tests.js -i tests/compiler/overload.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-tree -o tmp/vec-owner-transfer-overload-after-fixtures.json -j 1 --dist web/dist`: total=47, passed=43, failed=4。Vec push/len/nested generic push 由来の failure は解消。残りは tuple field owner obligation と Stack owner issue。
- `node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-vec-owner-transfer.json -j 4 --dist web/dist`: total=649, passed=643, failed=6。修正前の total=649, passed=637, failed=12 から Vec owner transfer 起因の 6 failure が解消。
- `stdlib/alloc/collections/vec.nepl` doctest は、今回の修正で隠れていた by-value observation API / fixture の owner leak が表面化する。これは既存 `ISS-20260425T000000Z-RV-STDLIB-004-91534828` の collection API / element cleanup 設計の対象として扱い、この issue では Vec transform owner transfer を完了範囲とする。
