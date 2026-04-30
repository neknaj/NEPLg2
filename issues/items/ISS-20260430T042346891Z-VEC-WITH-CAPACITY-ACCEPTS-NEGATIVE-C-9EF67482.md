---
id: ISS-20260430T042346891Z-VEC-WITH-CAPACITY-ACCEPTS-NEGATIVE-C-9EF67482
title: "Vec.with_capacity accepts negative capacity and can pass negative allocation size"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/vec.nepl, tests/stdlib/vec_collections.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260430T042346891Z-VEC-WITH-CAPACITY-ACCEPTS-NEGATIVE-C-9EF67482: Vec.with_capacity accepts negative capacity and can pass negative allocation size

## 概要

Vec.with_capacity only special-cases cap == 0. Negative cap values fall through to alloc_ptr with cap * size_of<T>, so an invalid public capacity can become a negative allocation size instead of a typed stdlib error.

## 対象

- `stdlib/alloc/collections/vec.nepl, tests/stdlib/vec_collections.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/vec.nepl` の `with_capacity` は修正前、`eq cap 0` だけを検査し、それ以外を `alloc_ptr<.T> mul cap size_of<.T>` に渡していた。
- `cap < 0` の public 入力が `StdErrorKind` の typed error にならず、allocator 境界へ負 byte size として到達し得た。
- `tests/stdlib/vec_collections.n.md` には負 capacity の回帰テストがなく、`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` も negative capacity guard を固定していなかった。

## 問題

Vec.with_capacity only special-cases cap == 0. Negative cap values fall through to alloc_ptr with cap * size_of<T>, so an invalid public capacity can become a negative allocation size instead of a typed stdlib error.

## 影響

Self-host code and stdlib callers cannot rely on Vec capacity APIs to enforce non-negative storage bounds. Resource and memory safety reasoning should not depend on allocator behavior for invalid negative sizes.

## 修正方針

Reject cap < 0 in Vec.with_capacity before allocation, document the contract, add .n.md regression tests, and extend the Vec source policy so the negative-capacity guard cannot regress.

## 検証

Run Vec collection tests, Vec source policy, issues check, and diff check. `stdlib/tests/vec.n.md` 全体は既存 ResourceIR owner/timeout 残件を拾うため、この issue の回帰確認は passing collection test へ置く。

確認済み:

- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/vec-negative-capacity-after-disjoint-merge.json -j 1 --dist web/dist`: `total=3`, `passed=3`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed (`files=439`)
- `git diff --check`: passed

既存残件:

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-negative-capacity-stdlib.json -j 1 --dist web/dist`: existing ResourceIR owner leak / timeout failures remain outside this issue.
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-negative-capacity-doctests.json -j 1 --dist web/dist`: existing Vec doctest ResourceIR owner failures remain outside this issue.

## 修正内容

- `Vec.with_capacity` に `cap < 0` guard を追加し、allocation 前に `Result::Err StdErrorKind::InvalidOperation` を返すようにした。
- `cap = 0` の empty Vec sentinel と `cap > 0` の allocation path は既存 semantics を維持した。
- `tests/stdlib/vec_collections.n.md` に `vec_negative_capacity_rejected` を追加し、負 capacity が `InvalidOperation` になることを確認した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に negative capacity guard の source policy を追加した。
