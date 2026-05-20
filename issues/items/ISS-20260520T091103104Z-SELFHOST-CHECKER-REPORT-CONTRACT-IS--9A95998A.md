---
id: ISS-20260520T091103104Z-SELFHOST-CHECKER-REPORT-CONTRACT-IS--9A95998A
title: "selfhost checker report contract is stale after declaration-header doctest"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "nodesrc/test_selfhost_checker_report_contract.js, tests/stdlib/neplg2_checker.n.md"
---

# ISS-20260520T091103104Z-SELFHOST-CHECKER-REPORT-CONTRACT-IS--9A95998A: selfhost checker report contract is stale after declaration-header doctest

## 概要

tests/stdlib/neplg2_checker.n.md now contains four stdout-normalized checker doctests, but nodesrc/test_selfhost_checker_report_contract.js still pins the old three-doctest baseline and fails before checking the new declaration-header report fixture.

## 対象

- `nodesrc/test_selfhost_checker_report_contract.js, tests/stdlib/neplg2_checker.n.md`

## 根拠

- `node nodesrc/test_selfhost_checker_report_contract.js` が `selfhost checker doctest count changed` で失敗し、actual 4 / expected 3 を報告した。
- `tests/stdlib/neplg2_checker.n.md` には `rejects_declaration_items_without_parser_header_evidence` の stdout-normalized doctest が追加済みだった。
- report contract は stdout report 形式を監視する役割なので、新しい doctest を監視対象から外したままにしてはいけない。

## 問題

tests/stdlib/neplg2_checker.n.md now contains four stdout-normalized checker doctests, but nodesrc/test_selfhost_checker_report_contract.js still pins the old three-doctest baseline and fails before checking the new declaration-header report fixture.

## 影響

Source policy regression runs can fail even though the checker doctest itself follows the current stdout report style, and the stale contract no longer monitors the new declaration-header diagnostic fixture.

## 修正方針

Update the checker report contract to include the new declaration-header doctest and its expected stdout check count without weakening the report-format assertions.

## 検証

Run node nodesrc/test_selfhost_checker_report_contract.js, node nodesrc/issues.js check, and git diff --check.

## 2026-05-20 Agent 1 修正

`nodesrc/test_selfhost_checker_report_contract.js` の期待 doctest list を現行の 4 fixture へ更新した。単なる数値配列ではなく、fixture title と expected check count の組にして、次に report fixture が増減した場合にどの契約がずれたか分かるようにした。

stdout、`exit_code: 0`、`stdio` / `normalize_newlines` tag、`checks_print_report` -> `checks_exit_code` の検査は維持しており、report 契約は弱めていない。

検証:

- `node nodesrc/test_selfhost_checker_report_contract.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
