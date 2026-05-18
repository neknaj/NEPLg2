---
id: ISS-20260518T114636396Z-CLIARG-C-STRING-CONVERSION-LACKS-ARG-21786F27
title: "cliarg C string conversion lacks argv buffer bound proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/std/env/cliarg/cstr.nepl, stdlib/std/env/cliarg/raw.nepl"
---

# ISS-20260518T114636396Z-CLIARG-C-STRING-CONVERSION-LACKS-ARG-21786F27: cliarg C string conversion lacks argv buffer bound proof

## 概要

std/env/cliarg/cstr exposes public C-string readers that scan a MemPtr until NUL without carrying the argv byte-buffer extent. cliarg_get_checked knows argv_buf and buf_size, but the proof is dropped before cstr conversion, so the implementation relies on an implicit NUL contract instead of a bounded source proof.

## 対象

- `stdlib/std/env/cliarg/cstr.nepl, stdlib/std/env/cliarg/raw.nepl`

## 根拠

- `stdlib/std/env/cliarg/cstr.nepl` は `cstr_len_result(MemPtr<u8>)` / `cstr_len(MemPtr<u8>)` / `cstr_to_str(MemPtr<u8>)` を public にし、NUL が現れるまで `mem_ptr_add p i` / `load_u8` で走査していた。
- `stdlib/std/env/cliarg/raw.nepl` の `cliarg_get_checked` は `argv_buf_raw` と `buf_size` を保持していたが、`arg_ptr` だけを `cstr_to_str` に渡していたため、C string が argv byte buffer の残り範囲内で終端している証明を落としていた。
- 旧 `cstr_to_str` は外部 argv byte 列を `string_from_mem_unchecked_result` へ渡し、UTF-8 検証なしに `str` を構築できた。

## 問題

std/env/cliarg/cstr exposes public C-string readers that scan a MemPtr until NUL without carrying the argv byte-buffer extent. cliarg_get_checked knows argv_buf and buf_size, but the proof is dropped before cstr conversion, so the implementation relies on an implicit NUL contract instead of a bounded source proof.

## 影響

A raw argv pointer can be converted to str without proving that the NUL terminator lies inside the owned argv buffer, and external bytes can bypass UTF-8 validation before becoming str. This weakens Stage 6 raw-memory-backed API boundaries and hides buffer extent errors from Resource IR/source policy review.

## 修正方針

Replace unbounded cstr_len/cstr_to_str public APIs with bounded Result-returning helpers that take a maximum byte extent, require the NUL terminator within that extent, validate UTF-8 before constructing str, and make cliarg_get_checked pass the remaining argv_buf extent derived from argv_buf_raw/buf_size.

## 検証

Run cliarg source policy, cstr/cliarg doctests, memory-safety compile_fail regressions, issue index check, and diff check.

## 対応内容

`cstr_len_result` / `cstr_len` / `cstr_to_str` の unbounded public API を削除し、`cstr_len_bounded_result(MemPtr<u8>, i32)` と `cstr_to_str_bounded_result(MemPtr<u8>, i32)` に置き換えた。

`cstr_len_bounded_result` は `max_len <= 0` を拒否し、`while and and eq done 0 eq ok 1 lt i max_len` の上限付き走査で NUL を探す。`Option::None` は invalid pointer、`max_len` 内に NUL が無い場合は missing terminator として `Err` を返すため、raw pointer 計算は呼び出し元が渡した byte extent を越えない。

`cstr_to_str_bounded_result` は length proof の成功後に `string_from_utf8_mem_result` を通す。外部 argv byte 列を `string_from_mem_unchecked_result` で直接 `str` にする経路は削除した。

`cliarg_get_checked` は `arg_ptr - argv_buf_raw` から `arg_offset` を求め、`0 <= arg_offset < buf_size` を確認した後、`buf_size - arg_offset` を `cstr_to_str_bounded_result` に渡す。これにより `args_get` が返した pointer と argv byte buffer owner extent が C string conversion まで保持される。

`nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` と `tests/stdlib/memory_safety.n.md` は、旧 unbounded reader 名、UTF-8 未検証 construction、argv 残り長を渡さない conversion の再導入を拒否する。

## 対応後検証

- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib\std\env\cliarg\cstr.nepl -i stdlib\tests\cliarg.n.md --no-tree -o tmp\agent1-cliarg-cstr-bounded-cliarg.json -j 1 --dist web\dist --assert-io`: total=9, passed=9
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-cliarg-cstr-bounded-memory-safety.json -j 1 --dist web\dist --assert-io`: total=59, passed=59
- `node nodesrc/tests.js -i stdlib\std\env\cliarg.nepl -i stdlib\std\env\cliarg\raw.nepl -i stdlib\tests\cliarg.n.md --no-tree -o tmp\agent1-cliarg-cstr-bounded-focused.json -j 1 --dist web\dist --assert-io`: total=10, passed=10
- `node nodesrc/issues.js check --dir issues`
