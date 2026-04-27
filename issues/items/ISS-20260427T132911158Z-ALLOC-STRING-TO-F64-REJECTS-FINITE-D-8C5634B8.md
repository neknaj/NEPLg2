---
id: ISS-20260427T132911158Z-ALLOC-STRING-TO-F64-REJECTS-FINITE-D-8C5634B8
title: "alloc/string to_f64 rejects finite decimal strings after digit scan"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/string.nepl; stdlib/tests/string.n.md"
source: ISS-20260427T132142587Z-ALLOC-STRING-DOC-COMMENTS-STILL-CONT-C037036C
---

# ISS-20260427T132911158Z-ALLOC-STRING-TO-F64-REJECTS-FINITE-D-8C5634B8: alloc/string to_f64 rejects finite decimal strings after digit scan

## 概要

to_f64 uses ok = 2 as the final success state, but inputs that end immediately after integer digits, fractional digits, or exponent digits can leave ok at 1. The final success check then returns Result::Err for ordinary finite decimal strings.

## 対象

- `stdlib/alloc/string.nepl; stdlib/tests/string.n.md`

## 根拠

- `to_f64` starts with `ok = 1`, sets `ok = 2` only when a non-digit transition is seen, and returns Ok only when `eq ok 2`.
- There is no focused stdlib test covering `to_f64 "123"`, `to_f64 "-1.5"`, or exponent input.

## 問題

to_f64 uses ok = 2 as the final success state, but inputs that end immediately after integer digits, fractional digits, or exponent digits can leave ok at 1. The final success check then returns Result::Err for ordinary finite decimal strings.

## 影響

Self-host config, diagnostics, and literal parsing need basic floating-point text parsing. Rejecting ordinary finite decimals makes the stdlib conversion unreliable and hides the missing coverage in current string tests.

## 修正方針

Normalize the parser state so a clean end-of-input after required digits is accepted, while malformed decimal/exponent forms still return Err. Add focused regressions for integer-only, fractional, exponent, signed, and invalid inputs.

## 修正内容

- `to_f64` の `ok = 2` に separator state と success state を兼任させる実装をやめた。
- 整数部、小数部、指数部を `done_int` / `done_frac` / `done_exp` でそれぞれ停止する state machine に整理した。
- clean end-of-input は `ok = 1` のまま成功として扱い、未消費 byte、digit 不足、指数部 digit 不足は `Result::Err 1` のまま維持した。
- `stdlib/tests/string.n.md` に、整数だけ、小数、先頭小数点、指数表記、invalid suffix / digit 不足の回帰テストを追加した。

## 検証

- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/string-to-f64-parser.json -j 1`: `total=8`, `passed=8`
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_string_doc_no_boilerplate.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string_numeric_overflow.n.md -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/string-to-f64-focused.json -j 1`: `total=28`, `passed=28`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-string-to-f64-parser.json -j 4`: `total=419`, `passed=419`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass（CRLF 変換 warning のみ）
