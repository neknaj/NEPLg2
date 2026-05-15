---
id: ISS-20260515T135819737Z-STDLIB-SOURCE-POLICY-LINE-LIMITS-C-4D318A9B
title: "stdlib source policy line limits count documentation comments"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-15
target: "nodesrc/source_policy/stdlib_builder_owner.js, nodesrc/test_stdlib_*boundary.js, nodesrc/test_stdlib_*no_unsafe_unwraps.js, doc/stdlib_doc_comment_policy.md"
---

# ISS-20260515T135819737Z-STDLIB-SOURCE-POLICY-LINE-LIMITS-C-4D318A9B: stdlib source policy line limits count documentation comments

## 概要

Multiple stdlib source-policy checks still enforce responsibility line limits using physical file lines, so adding required Japanese documentation comments can fail boundary policies even when implementation size does not grow.

## 対象

- `nodesrc/source_policy/stdlib_builder_owner.js`
- `nodesrc/test_stdlib_core_mem_boundary.js`
- `nodesrc/test_stdlib_math_module_split.js`
- `nodesrc/test_stdlib_text_boundary.js`
- `nodesrc/test_stdlib_string_*_boundary.js`
- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js`
- `nodesrc/test_stdlib_json_doc_no_boilerplate.js`
- `nodesrc/test_stdlib_nm_parser_document_boundary.js`
- `nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`
- `nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`
- `nodesrc/test_stdlib_stdio_debug_boundary.js`
- `nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`
- `nodesrc/test_stdlib_streamio_writer_boundary.js`
- `nodesrc/test_stdlib_vec_sort_module_split.js`
- `doc/stdlib_doc_comment_policy.md`

## 根拠

- `nodesrc/test_stdlib_core_mem_boundary.js` は `stdlib/core/mem/*.nepl` の line limit を `text.split("\n").length` で判定しており、doc-comment 増加を実装肥大化として扱っていた。
- `nodesrc/test_stdlib_math_module_split.js`、`test_stdlib_string_*_boundary.js`、`test_stdlib_text_boundary.js`、`test_stdlib_json_doc_no_boilerplate.js`、`test_stdlib_std_test_no_unsafe_unwraps.js` などにも、`.nepl` ファイルの物理行数を responsibility boundary として扱う同型の判定が残っていた。
- NEPLg2 の stdlib 方針では各関数・module・enum・struct に丁寧な日本語 doc-comment と doctest を整備するため、line limit が doc-comment を罰すると、source policy がドキュメント削減を誘導する。

## 問題

Multiple stdlib source-policy checks still enforce responsibility line limits using physical file lines, so adding required Japanese documentation comments can fail boundary policies even when implementation size does not grow.

## 影響

The policy signal can conflict with the stdlib documentation contract. Developers may satisfy source policy by reducing documentation quality instead of splitting oversized implementation responsibilities.

## 修正方針

Centralize NEPL implementation line counting in a shared source-policy helper and use it for stdlib `.nepl` responsibility limits. The helper strips `//` doc/comment lines and blank lines, so policies continue to catch implementation growth while allowing necessary API documentation.

## 対応

- `nodesrc/source_policy/stdlib_builder_owner.js` に `implementationLineCount` を追加し、comment stripping と non-empty line filtering を共通化した。
- core/mem、core/math、std/text、std/test、stdio/debug、streamio、diag/error、json、nm parser document、alloc/string 系、Vec sort 系の stdlib source-policy line limit を `implementationLineCount` に移した。
- `nodesrc/test_stdlib_string_integer_boundary.js` の重複 `codeLineCount` helper を共通 helper へ統合した。
- `doc/stdlib_doc_comment_policy.md` に、source policy の file-size / responsibility 境界は doc-comment を含む物理行数ではなく実装行数で監視することを明記した。

## 検証

Run changed source-policy checks, source-policy aggregate, issues index/check, and diff whitespace checks.

- `node --check nodesrc/source_policy/stdlib_builder_owner.js`: pass
- `node --check nodesrc/test_stdlib_core_mem_boundary.js`: pass
- `node --check nodesrc/test_stdlib_math_module_split.js`: pass
- `node --check nodesrc/test_stdlib_text_boundary.js`: pass
- `node --check nodesrc/test_stdlib_string_integer_boundary.js`: pass
- `node --check nodesrc/test_stdlib_vec_sort_module_split.js`: pass
- `node nodesrc/test_stdlib_core_mem_boundary.js`: pass
- `node nodesrc/test_stdlib_math_module_split.js`: pass
- `node nodesrc/test_stdlib_text_boundary.js`: pass
- `node nodesrc/test_stdlib_string_integer_boundary.js`: pass
- `node nodesrc/test_stdlib_string_search_boundary.js`: pass
- `node nodesrc/test_stdlib_string_slice_boundary.js`: pass
- `node nodesrc/test_stdlib_vec_sort_module_split.js`: pass
- `node nodesrc/test_stdlib_json_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: pass
- `node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_stdio_debug_boundary.js`: pass
