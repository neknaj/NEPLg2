---
id: ISS-20260430T133512620Z-STDLIB-STRING-DOCTESTS-KEEP-OLD-STD--4220AD4F
title: "stdlib string doctests keep old std/test result contracts and ret metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/string.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T133512620Z-STDLIB-STRING-DOCTESTS-KEEP-OLD-STD--4220AD4F: stdlib string doctests keep old std/test result contracts and ret metadata

## 概要

stdlib/tests/string.n.md still has std/test doctests that return checks_exit_code checks without printing reports; string_find_byte_index also mixes assert_eq_i32 TestAssertion with a Result-returning helper and now fails to compile.

## 対象

- `stdlib/tests/string.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/stdlib-string-report-before-agent1.json -j 1 --dist web/dist` で `string_find_byte_index` が compile fail していた。
- compile error は `expect_find_some` が `Result<(),str>` を返す helper なのに、`assert_eq_i32` の `TestAssertion` を返していたことが原因だった。
- ほかの `std/test` doctest も `checks_exit_code checks` だけで成功可否を返しており、stdout report を fixture として固定していなかった。

## 問題

stdlib/tests/string.n.md still has std/test doctests that return checks_exit_code checks without printing reports; string_find_byte_index also mixes assert_eq_i32 TestAssertion with a Result-returning helper and now fails to compile.

## 影響

The string stdlib regression file is not a clean CI signal, and assertion report formatting is not pinned for self-host runner parity.

## 修正方針

Use Result-returning check helpers where helper signatures require Result<(),str>, add checks_print_report before checks_exit_code, and migrate the std/test cases from ret metadata to exit_code plus stdout fixtures.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/stdlib-string-report-agent1.json -j 1 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`expect_find_some` は `Result<(),str>` を返す helper として `check_eq_i32` を使うよう修正した。5件の `std/test` doctest は `checks_print_report` を通して stdout report を出し、metadata を `exit_code: 0` + `stdout: mlstr` へ移行した。

`ret: 1` の先頭4件は `std/test` assertion suite ではなく、言語レベルの戻り値を検証する fixture なので今回の scope から分離した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/stdlib-string-report-agent1.json -j 1 --dist web/dist`: total=9, passed=9
- `rg -n '^ret: 0|checks_exit_code checks|assert_eq_i32' stdlib/tests/string.n.md`: no matches
