---
id: ISS-20260429T231611047Z-STD-TEST-ASSERTION-DISCARD-SOURCE-PO-B9226736
title: "std/test assertion discard source policy is missing"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nodesrc/test_doctest_std_test_assertion_report_contract.js, .github/workflows/ci.yml, stdlib/**/*.nepl, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260429T231611047Z-STD-TEST-ASSERTION-DISCARD-SOURCE-PO-B9226736: std/test assertion discard source policy is missing

## 概要

std/test assertion helpers return TestAssertion values that must be aggregated into a report. A doctest can accidentally call assert_* or check_* as a semicolon-terminated statement and discard the owner value without a source-policy regression.

## 対象

- `nodesrc/test_doctest_std_test_assertion_report_contract.js, .github/workflows/ci.yml, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` の direct std/test assertion discard subcase では、`std/test::assert_*` / `check_*` が `TestAssertion` owner を返すため、`checks_push` / `test_report_add` / `checks_print_report` へ渡さず捨てる書き方を禁止する必要がある。
- 既存の source policy は `std/test` 実装自体の unsafe unwrap や tutorial chapter の書き方は検査しているが、`.n.md` / NEPL doc-comment doctest の `std/test` assertion 戻り値破棄までは検査していなかった。
- `core/test` の immediate assertion と違い、`std/test` assertion は report 集約 API の一部なので、doctest authoring の段階で誤用を検出する方が owner diagnostic に任せるより原因が明確になる。

## 問題

std/test assertion helpers return TestAssertion values that must be aggregated into a report. A doctest can accidentally call assert_* or check_* as a semicolon-terminated statement and discard the owner value without a source-policy regression.

## 影響

The doctest may compile or fail later through owner diagnostics instead of explaining the test authoring error, and the migration from ret-only tests to stdout assertion reports can regress silently.

## 修正方針

Add a source-policy test that scans .n.md doctests and NEPL doc-comment doctests importing std/test, rejecting semicolon-terminated bare assert/check calls that discard TestAssertion values. Keep helper functions that return assertions as expressions allowed.

## 検証

Run the new source-policy test, the parent doctest metadata tests, focused migrated doctests, and issue index checks.

## 対応結果

`nodesrc/test_doctest_std_test_assertion_report_contract.js` を追加し、`tests` / `tutorials` / `stdlib` / `examples` の `.n.md` と NEPL doc-comment doctest を走査する source policy を作成した。

この policy は `#import "std/test" as ...` を含む doctest だけを対象にし、semicolon で終わる bare `assert*` / `check*` statement を禁止する。helper 関数が `assert_*` / `check_*` を末尾式として返し、その戻り値を caller が `checks_push` へ渡す書き方は許可する。

既存の違反として、`stdlib/alloc/collections/vec.nepl`、`stdlib/alloc/diag/error.nepl`、`stdlib/alloc/string.nepl`、`stdlib/core/mem.nepl`、`stdlib/core/result.nepl`、`stdlib/core/traits/{debug,hash,serialize,stringify}.nepl` の doc-comment doctest に、`std/test` assertion を semicolon で捨てる古い形式が残っていた。これらは `checks_new` / `checks_push` / `checks_exit_code` 形式へ移行した。

CI の Source policy regressions にも追加したため、`std/test` assertion owner を report へ集約せず捨てる regression は push 時に検出される。

`stdlib/alloc/string.nepl` の migrated doctest では、`Result<(),str>` を返す helper が `assert_eq_i32` の `TestAssertion` をそのまま返していた既存の型不整合も検出した。同じ `std/test` report API 誤用なので、helper 側は `check_eq_i32` を返す形へ直した。

## 検証結果

- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
- `node nodesrc/test_doctest_diag_code_metadata.js`: passed
- `node nodesrc/test_doctest_exit_code_metadata.js`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/diag/error.nepl -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/diag/error.nepl -n 2 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 6 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 8 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 7 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/core/traits/debug.nepl -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/core/traits/hash.nepl -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/core/traits/serialize.nepl -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/core/traits/stringify.nepl -n 1 --dist web/dist`: passed

`stdlib/core/mem.nepl` doctest #5/#6 は migrated report 形式としては source policy を満たすが、compile phase で `resource.cell.uninit` になる。これは `memset_u8` / `fill_i32` の raw fill helper が caller 側の initialized cell state として Resource IR に summary されていない既存 core 問題であり、`ISS-20260429T233515324Z-RESOURCE-IR-DOES-NOT-SUMMARIZE-RAW-F-48450939` として分離した。
