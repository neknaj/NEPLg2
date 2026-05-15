---
id: ISS-20260515T133717458Z-ALLOC-STRING-DOC-COMMENT-DOCTESTS-ST-AE5C1FAA
title: "alloc string doc-comment doctests still use legacy checks_exit_code reports"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/alloc/string/{find,integer,float}.nepl, stdlib/alloc/string/{search,integer,float}/**/*.nepl"
---

# ISS-20260515T133717458Z-ALLOC-STRING-DOC-COMMENT-DOCTESTS-ST-AE5C1FAA: alloc string doc-comment doctests still use legacy checks_exit_code reports

## 概要

alloc/string documentation examples still use checks_exit_code or return-value-only doctests instead of canonical TestReport stdout.

## 対象

- `stdlib/alloc/string/{find,integer,float}.nepl, stdlib/alloc/string/{search,integer,float}/**/*.nepl`

## 根拠

- `stdlib/alloc/string/find.nepl`、`integer.nepl`、`float.nepl`、`search/byte_find.nepl`、`search/compare.nepl`、`integer/parse.nepl`、`float/parse.nepl`、`integer/common/bool.nepl` の doc-comment doctest は `checks_exit_code` または戻り値だけで成功を表し、stdout に assertion label / expected / actual を固定していなかった。
- `from_bool` の doctest は `std/test` report を使わず `i32` 戻り値だけで成功を表しており、他の canonical stdlib doctest と観測形式がずれていた。
- focused doctest 移行時、`integer/parse.nepl` の doctest は parse module 単体 import では `from_i32` を解決できない潜在的な import 不備も露出した。
- source-policy の `alloc/string integer boundary` は doc-comment を含む物理行数で file size を判定しており、doc-comment を丁寧に増やす方針と衝突していた。境界検査の本来の対象は実装の肥大化であり、doc-comment 量ではない。

## 問題

alloc/string documentation examples still use checks_exit_code or return-value-only doctests instead of canonical TestReport stdout.

## 影響

String conversion and search docs can pass by exit code while losing assertion labels and expected/actual stdout details required by the shared doctest contract.

## 修正方針

Migrate the affected alloc/string doc-comment doctests to neplg2:test[stdio, normalize_newlines], exit_code metadata, and test_report_print_stdout/test_report_exit_code.

## 対応

- 対象 8 file / 9 doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` 期待へ移行した。
- `find` / `str_find` / `str_starts_with_at` / bool / integer / float parse の観測値を named `TestReport` に集約し、成功時にも assertion label、kind、expected、actual が stdout に残るようにした。
- parse 単体 doctest は `alloc/string/integer/format` を明示 import し、failure branch の `from_i32` 依存が module boundary を越えて暗黙に解決されない問題を解消した。
- `nodesrc/test_alloc_string_doc_report_contract.js` を追加し、対象 doctest が `ret:` / `checks_exit_code` / `result_exit_code` へ戻らないことを source-policy で固定した。
- `nodesrc/run_source_policy_regressions.js` に同 contract を追加した。
- `nodesrc/test_stdlib_string_integer_boundary.js` の file size policy は comment stripping 後の実装行数を数えるように直し、doc-comment を削らずに実装境界だけを監視する形へ戻した。

## 検証

Run focused doc-comment doctests for updated string files, a source-policy contract, issues check, and diff whitespace check.

- `node nodesrc/tests.js -i stdlib/alloc/string/find.nepl -i stdlib/alloc/string/integer.nepl -i stdlib/alloc/string/float.nepl -i stdlib/alloc/string/search/byte_find.nepl -i stdlib/alloc/string/search/compare.nepl -i stdlib/alloc/string/integer/parse.nepl -i stdlib/alloc/string/float/parse.nepl -i stdlib/alloc/string/integer/common/bool.nepl --no-tree -o tmp/agent1-alloc-string-doc-report.json -j 1 --dist web/dist --assert-io`: 9 passed
- `node --check nodesrc/test_alloc_string_doc_report_contract.js`: pass
- `node nodesrc/test_alloc_string_doc_report_contract.js`: pass
- `node --check nodesrc/run_source_policy_regressions.js`: pass
- `node nodesrc/test_stdlib_string_integer_boundary.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: source-policy warning なし
