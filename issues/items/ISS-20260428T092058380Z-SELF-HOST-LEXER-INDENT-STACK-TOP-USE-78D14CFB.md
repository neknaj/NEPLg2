---
id: ISS-20260428T092058380Z-SELF-HOST-LEXER-INDENT-STACK-TOP-USE-78D14CFB
title: "self-host lexer indent stack top uses unsafe unwrap"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/lexer.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260428T092058380Z-SELF-HOST-LEXER-INDENT-STACK-TOP-USE-78D14CFB: self-host lexer indent stack top uses unsafe unwrap

## 概要

The offside lexer reads the indent stack top through unwrap(get_ref(...)), which violates the stdlib unsafe helper source policy and fails GitHub Actions Source policy regressions.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js, tests/stdlib/neplg2_lexer.n.md`

## 根拠

- GitHub Actions `Source policy regressions` が `nodesrc/test_stdlib_no_unsafe_helpers.js` で失敗した。
- 失敗箇所は `stdlib/neplg2/core/syntax/lexer.nepl:397` の `unwrap<i32> get_ref<i32> stack sub len_ref<i32> stack 1`。
- offside lexer の indent stack は初期値 0 を push する設計だが、破損時に trap する helper では lexer diagnostic として扱えない。

## 問題

The offside lexer reads the indent stack top through unwrap(get_ref(...)), which violates the stdlib unsafe helper source policy and fails GitHub Actions Source policy regressions.

## 影響

main stays red after the self-host lexer offside token change, and an empty/corrupt indent stack would trap instead of returning a lexer diagnostic.

## 修正方針

Replace the unsafe stack-top helper with a Result-returning helper that reports InvalidIndentation, and update callers to propagate diagnostics while freeing owned buffers.

## 検証

Run node nodesrc/test_stdlib_no_unsafe_helpers.js and focused neplg2 lexer doctests.

## 対応結果

- `lex_stack_top` を `lex_stack_top_result` に置き換え、空 stack / top 取得失敗を `LexErrorCode::InvalidIndentation` の `Result::Err` として返すようにした。
- `lex_dedent_to` と `lex_line_start` は stack-top error を受けたら token buffer と indent stack を解放して diagnostic を返す。
- `unwrap` / `unwrap_ok` / `unwrap_err` などの unsafe helper は self-host lexer implementation から消えた。

## 実行した検証

- `node nodesrc/test_stdlib_no_unsafe_helpers.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-safe-stack-focused.json -j 1`: total=9, passed=9
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-safe-stack-focused.json -j 1`: total=34, passed=34
- `node nodesrc/issues.js check`: pass
