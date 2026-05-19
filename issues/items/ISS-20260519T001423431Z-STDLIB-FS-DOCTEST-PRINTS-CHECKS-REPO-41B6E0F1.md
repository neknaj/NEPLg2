---
id: ISS-20260519T001423431Z-STDLIB-FS-DOCTEST-PRINTS-CHECKS-REPO-41B6E0F1
title: "stdlib fs doctest prints checks report without stdout fixture"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/tests/fs.n.md, nodesrc/test_stdlib_fs_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T001423431Z-STDLIB-FS-DOCTEST-PRINTS-CHECKS-REPO-41B6E0F1: stdlib fs doctest prints checks report without stdout fixture

## 概要

stdlib/tests/fs.n.md calls checks_print_report and checks_exit_code for the missing-file case, but the manifest does not pin stdout / exit_code metadata and still uses legacy checks_* helpers.

## 対象

- `stdlib/tests/fs.n.md, nodesrc/test_stdlib_fs_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/fs.n.md` は `fs_read_to_string "__definitely_missing_file__.txt"` が `Result::Err` になることを確認していた。
- 旧実装は `checks_print_report` / `checks_exit_code` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなかった。
- unexpected success branch は `Result::Ok s` の文字列 payload を受け取るため、canonical report 移行時にも成功payloadの消費を明示する必要があった。

## 問題

stdlib/tests/fs.n.md calls checks_print_report and checks_exit_code for the missing-file case, but the manifest does not pin stdout / exit_code metadata and still uses legacy checks_* helpers.

## 影響

The fs facade missing-file behavior can regress without a fixture-checked assertion label and stdout report contract.

## 修正方針

Migrate the fs doctest to named TestReport stdout fixture, add exit_code metadata, and add a source policy contract that rejects checks_* / ret-only regression.

## 検証

Run the fs source policy contract, focused fs doctest, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `fs_main` を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- 旧 `checks_*` helper を named `TestReport` API へ置き換えた。
- `missing file returns error` assertion を stdout に固定した。
- unexpected success branch では `test_consume_str s` を呼び、成功payload ownerを捨てたままにしないようにした。
- `nodesrc/test_stdlib_fs_nmd_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式、unexpected success payload 未消費への退行を source policy で拒否する。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
