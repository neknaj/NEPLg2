---
id: ISS-20260506T115339863Z-UTF-8-NEGATIVE-PATH-TESTS-LEAK-OWNED-2057C710
title: "UTF-8 negative-path tests leak owned str in Ok arms"
area: stdlib
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/tests/string.n.md, tests/stdlib/text_utf8.n.md"
---

# ISS-20260506T115339863Z-UTF-8-NEGATIVE-PATH-TESTS-LEAK-OWNED-2057C710: UTF-8 negative-path tests leak owned str in Ok arms

## 概要

stdlib/tests/string.n.md and tests/stdlib/text_utf8.n.md bind Result::Ok _text in invalid UTF-8 negative-path branches and then drop the owned str without moving or freeing it. After the Resource IR assignment/drop checks on remote main, these fixtures fail static checking with resource.raw.ownership_violation. stdlib/tests/string.n.md::doctest#5 also mixes TestAssertion and Result<(),str> match arm return types.

## 対象

- `stdlib/tests/string.n.md, tests/stdlib/text_utf8.n.md`

## 根拠

- 未記入

## 問題

stdlib/tests/string.n.md and tests/stdlib/text_utf8.n.md bind Result::Ok _text in invalid UTF-8 negative-path branches and then drop the owned str without moving or freeing it. After the Resource IR assignment/drop checks on remote main, these fixtures fail static checking with resource.raw.ownership_violation. stdlib/tests/string.n.md::doctest#5 also mixes TestAssertion and Result<(),str> match arm return types.

## 影響

The tests are intended to prove invalid UTF-8 is rejected, but unreachable-at-runtime Ok branches still must satisfy static owner obligations. Leaving the fixtures as-is hides real memory-safety requirements and blocks broad string/text regression runs.

## 修正方針

Rewrite negative-path Ok arms to bind the returned str as an owned value and move it into a failure message or otherwise consume it. Normalize stdlib/tests/string.n.md::doctest#5 so all match arms return Result<(),str>/TestAssertion consistently through std/test helpers.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/string.n.md -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/utf8-negative-path-owner-fixtures.json -j 1 and confirm all cases pass.

## 2026-05-06 対応

- `stdlib/tests/string.n.md` の `expect_find_some` を `assert_eq_i32` ではなく `check_eq_i32` へ変更し、関数 signature の `Result<(),str>` と match arm 型を一致させた。
- `string_utf8_mem_result` の invalid UTF-8 Ok arm は `_text` を捨てず、`text` を `Result::Err text` として `checks_push` へ移動する形にした。
- `tests/stdlib/text_utf8.n.md` の invalid UTF-8 negative-path Ok arm も同様に、返ってきた `str` owner を failure result へ移動する形にした。

検証:

- `node nodesrc/tests.js -i stdlib/tests/string.n.md -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/utf8-negative-path-owner-fixtures.json -j 1`: total=18, passed=18
