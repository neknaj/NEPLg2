---
id: ISS-20260430T044235959Z-VEC-MODULE-DOCTESTS-LEAK-OWNERS-UNDE-EB5A5659
title: "Vec module doctests leak owners under strict ResourceIR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/vec.nepl, stdlib/std/fs.nepl, stdlib/tests/vec.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260430T044235959Z-VEC-MODULE-DOCTESTS-LEAK-OWNERS-UNDE-EB5A5659: Vec module doctests leak owners under strict ResourceIR

## 概要

Most doctests embedded in stdlib/alloc/collections/vec.nepl still use old by-value examples such as len/get without preserving or freeing the Vec owner. Under strict ResourceIR, the module doctest run reports 35 owner leaks out of 39 doctests.

## 対象

- `stdlib/alloc/collections/vec.nepl, stdlib/std/fs.nepl, stdlib/tests/vec.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-doctest-review-20260430.json -j 1 --dist web/dist` で `total=39`, `passed=4`, `failed=35` だった。
- 失敗は、doctest が `len/get/data_mem_ptr` などの値渡し観測を行った後に `Vec` owner を閉じないこと、`count/fold/reduce/find/any/all` が読み取りなのに `Vec` を値で受け取り backing storage owner を閉じないこと、`pop/partition` が owner を `.Pair` で返して caller 側の field move/free を ResourceIR が証明できないことに集中していた。
- `Vec` は self-host と stdlib collection の基礎なので、doctest が owner leak を教える状態はメモリ安全方針に反する。

## 問題

Most doctests embedded in stdlib/alloc/collections/vec.nepl still use old by-value examples such as len/get without preserving or freeing the Vec owner. Under strict ResourceIR, the module doctest run reports 35 owner leaks out of 39 doctests.

## 影響

The stdlib documentation teaches unsafe owner discipline for the core collection used by self-host, and CI cannot use Vec module doctests as regression coverage while they fail strict memory-safety checking.

## 修正方針

Rewrite Vec doctests to use borrowed read APIs when the owner must remain live, explicitly free successful owners, and keep consuming terminal APIs only when they close the owner. Add a source policy check that Vec doctests contain free/borrowed-read coverage and run the module doctests to completion.

## 検証

Run Vec module doctests, Vec collection tests, Vec source policy, source policy regression runner, issues check, and diff check.

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-doctest-owner-discipline-after-named-results.json -j 1 --dist web/dist`: `total=39`, `passed=39`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/vec-doctest-owner-discipline-collections.json -j 1 --dist web/dist`: `total=3`, `passed=3`
- `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/fs-after-vec-pop-result-parsefix.json -j 1 --dist web/dist`: `total=7`, `passed=7`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed

既存残件:

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-stdlib-test-owner-discipline-after-fix.json -j 1 --dist web/dist`: doctest#1 は pass。doctest#2 は分割 probe では各まとまりが pass するが、既存の巨大 functional helper doctest が 60 秒 timeout に当たる。

## 修正内容

- `stdlib/alloc/collections/vec.nepl` の doctest を、borrowed read helper で観測し、戻り値 owner を明示的に `free` する形へ更新した。
- `count` / `fold` / `reduce` / `find` / `any` / `all` を `&Vec<.T>` 受け取りかつ `.T: Copy` に変更し、読み取り helper が `Vec` owner を消費しない契約にした。
- owner を含む匿名 `.Pair` 戻り値をやめ、`VecPop<.T>` と `VecPartition<.T>` を追加して `pop` / `partition` の戻り値 owner を名前付き field で追跡できる形にした。
- `stdlib/std/fs.nepl` の `v::pop` 利用箇所を `VecPop.vec` field へ追従させた。
- `stdlib/tests/vec.n.md` の古い owner-leaking 呼び方を、今回変更した owner-safe API に追従した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、borrowed functional helper と named owner result の source policy を追加した。
