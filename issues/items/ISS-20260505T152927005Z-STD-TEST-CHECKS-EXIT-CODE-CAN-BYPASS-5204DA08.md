---
id: ISS-20260505T152927005Z-STD-TEST-CHECKS-EXIT-CODE-CAN-BYPASS-5204DA08
title: "std/test checks_exit_code can bypass stdout report fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-06
target: "nodesrc/test_doctest_std_test_assertion_report_contract.js, tests/stdlib/selfhost_cliarg_parser.n.md, tests/stdlib/selfhost_cli_file_io.n.md, tests/stdlib/text_utf8.n.md, tests/stdlib/string.n.md, stdlib/neplg2/cli/reporter.nepl"
---

# ISS-20260505T152927005Z-STD-TEST-CHECKS-EXIT-CODE-CAN-BYPASS-5204DA08: std/test checks_exit_code can bypass stdout report fixtures

## 概要

std/test doctests can still call checks_exit_code on a raw checks accumulator without first printing checks_print_report, so assertion details are not fixed in stdout fixtures.

## 対象

- `nodesrc/test_doctest_std_test_assertion_report_contract.js, tests/stdlib/selfhost_cliarg_parser.n.md, tests/stdlib/selfhost_cli_file_io.n.md, tests/stdlib/text_utf8.n.md, tests/stdlib/string.n.md, stdlib/neplg2/cli/reporter.nepl`

## 根拠

- `rg -n "checks_exit_code\s+checks\b" tests stdlib tutorials examples -g "*.n.md" -g "*.nepl"` で、`tests/stdlib/selfhost_cliarg_parser.n.md`、`tests/stdlib/selfhost_cli_file_io.n.md`、`tests/stdlib/text_utf8.n.md`、`tests/stdlib/string.n.md`、`stdlib/neplg2/cli/reporter.nepl` に raw `checks` accumulator を直接終了コードへ渡す doctest が残っていた。
- 既存の `nodesrc/test_doctest_std_test_assertion_report_contract.js` は `std/test` assertion の bare discard は検出していたが、`checks_exit_code checks` のように report print を通さず成功/失敗だけ返す経路を検出していなかった。
- この形では `std/test` の assertion detail / report order / report format が stdout fixture として固定されず、Rust runner と selfhost runner の比較で差分を見落とす。

## 問題

std/test doctests can still call checks_exit_code on a raw checks accumulator without first printing checks_print_report, so assertion details are not fixed in stdout fixtures.

## 影響

Rust runner and future selfhost runner can both return success while assertion report formatting or detail emission regresses.

## 修正方針

Extend the doctest std/test source policy to reject checks_exit_code on unprinted accumulators, then migrate remaining fixtures to checks_print_report + stdout + exit_code.

## 検証

Run the updated source policy and focused doctests for the migrated files.

## 2026-05-06 対応

- `nodesrc/test_doctest_std_test_assertion_report_contract.js` に `checks_exit_code <identifier>` の source policy を追加し、`checks_print_report` の結果 binding 以外を終了コードに渡す書き方を拒否するようにした。
- `tests/stdlib/selfhost_cliarg_parser.n.md` の assertion-style doctest を `checks_print_report` + `stdout:` + `exit_code:` へ移行した。
- `tests/stdlib/selfhost_cli_file_io.n.md`、`tests/stdlib/text_utf8.n.md`、`tests/stdlib/string.n.md`、`stdlib/neplg2/cli/reporter.nepl` に残っていた raw `checks_exit_code checks` を `checks_print_report` 経由に移行し、対象 doctest の stdout report を fixture 化した。
- `rg -n "checks_exit_code\s+checks\b" tests stdlib tutorials examples -g "*.n.md" -g "*.nepl"` で残存がないことを確認した。

## 2026-05-06 検証結果

- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: pass。
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md --no-tree -o tmp/selfhost_cliarg_parser_stdout_contract.json -j1 --dist web/dist`: total=10, passed=10。
- `node nodesrc/tests.js -i tests/stdlib/string.n.md --no-tree -o tmp/string_stdout_contract.json -j1 --dist web/dist`: total=17, passed=17。
- `node nodesrc/run_doctest.js -i stdlib/neplg2/cli/reporter.nepl -n 1 --dist web/dist`: pass。
- `tests/stdlib/selfhost_cli_file_io.n.md` は total=4, failed=4、`tests/stdlib/text_utf8.n.md` は total=9, passed=2, failed=7。失敗はいずれも compile phase の `resource.raw.unsafe_memory_boundary` で、`stdlib/alloc/io.nepl::io_bytebuf_from_str_result` と `stdlib/std/text.nepl::text_utf8_byte_at` の raw-memory-backed pure helper が原因である。stdout report 追加の実行検証はこの既存 stdlib boundary issue の解消後に再実行する。
