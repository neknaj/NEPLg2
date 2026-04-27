---
id: ISS-20260427T132911011Z-ALLOC-STRING-BYTE-SLICING-CAN-CONSTR-FA25D390
title: "alloc/string byte slicing can construct invalid UTF-8 str"
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

# ISS-20260427T132911011Z-ALLOC-STRING-BYTE-SLICING-CAN-CONSTR-FA25D390: alloc/string byte slicing can construct invalid UTF-8 str

## 概要

Public str_slice_result copies an arbitrary byte range with string_from_mem_unchecked_result. A caller can slice the middle of a multi-byte UTF-8 sequence and still receive Result::Ok<str>, which breaks the str UTF-8 invariant fixed at external input boundaries.

## 対象

- `stdlib/alloc/string.nepl; stdlib/tests/string.n.md`

## 根拠

- `str_slice_result` clamps byte indices but calls `string_from_mem_unchecked_result` without UTF-8 boundary validation.
- `str_split_result` builds parts through `str_slice_result`, so the same contract affects split-derived strings.

## 問題

Public str_slice_result copies an arbitrary byte range with string_from_mem_unchecked_result. A caller can slice the middle of a multi-byte UTF-8 sequence and still receive Result::Ok<str>, which breaks the str UTF-8 invariant fixed at external input boundaries.

## 影響

Self-host lexer/parser/diagnostics code relies on str being valid UTF-8 after construction. Invalid strings produced by stdlib slicing can later corrupt text output, JSON/markdown generation, or byte-based scanners that assume string invariants are preserved inside alloc/string.

## 修正方針

Define and enforce the boundary contract for string slicing. Prefer rejecting non-UTF-8-boundary start/end in str_slice_result and keeping byte-level operations in explicitly raw/byte APIs. Add regressions for multibyte partial slices and split behavior.

## 修正内容

- `str_utf8_is_boundary` を追加し、0、`len(s)`、continuation byte 以外の位置だけを UTF-8 文字境界として扱うようにした。
- `str_slice_result` が clamp 後の start/end を `str_utf8_is_boundary` で確認し、非境界なら `Result::Err "string.slice invalid utf8 boundary"` を返すようにした。
- `str_slice` は互換 facade として、非境界 slice では従来どおり空文字列 fallback を返す。
- `stdlib/tests/string.n.md` に multibyte 文字の valid slice / invalid partial slice regression を追加した。
- source policy を拡張し、`str_slice_result` から境界検査が外れないようにした。

## 検証

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_string_doc_no_boilerplate.js`: pass
- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/string-utf8-slice-boundary.json -j 1`: `total=9`, `passed=9`
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string_numeric_overflow.n.md -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/string-utf8-slice-focused.json -j 1`: `total=29`, `passed=29`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-string-utf8-slice-boundary.json -j 4`: `total=420`, `passed=420`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass（CRLF 変換 warning のみ）
