---
id: ISS-20260507T100459865Z-EXAMPLES-RELY-ON-QUALIFIED-ALLOC-STR-B6D68CEA
title: "examples rely on qualified alloc/string facade re-exports"
area: examples
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "examples/rpn.nepl, examples/rpn_legacy.nepl, examples/bf.nepl, nodesrc/test_examples_string_direct_imports.js"
---

# ISS-20260507T100459865Z-EXAMPLES-RELY-ON-QUALIFIED-ALLOC-STR-B6D68CEA: examples rely on qualified alloc/string facade re-exports

## 概要

RPN and Brainfuck examples import alloc/string as a broad facade and call s::len, s::byte_at, s::str_trim, s::str_slice_result, s::to_i32, and s::str_eq through that facade. alloc/string is now a re-export facade, and qualified calls through broad facades are not stable, so focused example doctests fail with resolve.identifier.undefined before Stack observer changes can be validated.

## 対象

- `examples/rpn.nepl, examples/rpn_legacy.nepl, examples/bf.nepl, nodesrc/test_examples_string_direct_imports.js`

## 根拠

- `examples/rpn.nepl` / `examples/rpn_legacy.nepl` / `examples/bf.nepl` は `#import "alloc/string" as s` で broad facade を qualified namespace として使っていた。
- 現在の `alloc/string.nepl` は `access` / `search` / `slice` / `integer` / `concat` などを re-export する facade であり、qualified 呼び出しは実体 module を直接 import する方針に反していた。
- focused doctest は `s::len` / `s::byte_at` / `s::to_i32` などを `resolve.identifier.undefined` として報告していた。

## 問題

RPN and Brainfuck examples import alloc/string as a broad facade and call s::len, s::byte_at, s::str_trim, s::str_slice_result, s::to_i32, and s::str_eq through that facade. alloc/string is now a re-export facade, and qualified calls through broad facades are not stable, so focused example doctests fail with resolve.identifier.undefined before Stack observer changes can be validated.

## 影響

Example doctests cannot be used as regression coverage for stdlib collection changes, and tutorial/example code keeps demonstrating an import style that the stdlib facade boundary already rejects elsewhere.

## 修正方針

Import the concrete alloc/string submodules needed by each example and call those qualified modules directly. Add a source-policy regression so examples do not reintroduce qualified calls through alloc/string as a broad facade.

## 検証

- `trunk build`: passed
- `node nodesrc/test_examples_string_direct_imports.js`: passed
- `node nodesrc/tests.js -i examples/rpn.nepl -i examples/rpn_legacy.nepl -i examples/bf.nepl --no-tree -o tmp/examples-string-direct-imports.json -j 1 --dist web/dist`: total=5, passed=5

## 対応結果

- RPN examples は `alloc/string/access` / `search` / `slice` / `integer/parse` / `integer/format` / `concat` を用途ごとに直接 import するようにした。
- Brainfuck example は `alloc/string/access` を直接 import し、`len` / `byte_at` を実体 module 経由にした。
- `nodesrc/test_examples_string_direct_imports.js` を追加し、examples が broad `alloc/string` facade を qualified namespace として再導入しないことを固定した。
- `nodesrc/run_source_policy_regressions.js` に regression を追加した。
