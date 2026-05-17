---
id: ISS-20260517T193057449Z-SELFHOST-STDLIB-MAP-DOCTESTS-STILL-U-320C9452
title: "selfhost stdlib map doctests still use ret metadata and hide reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "tests/stdlib/neplg2_stdlib_map.n.md, nodesrc/test_selfhost_stdlib_map_report_contract.js"
---

# ISS-20260517T193057449Z-SELFHOST-STDLIB-MAP-DOCTESTS-STILL-U-320C9452: selfhost stdlib map doctests still use ret metadata and hide reports

## 概要

tests/stdlib/neplg2_stdlib_map.n.md has three doctests that call checks_print_report and checks_exit_code, but their manifests still use ret: 0 without stdout and exit_code expectations. Focused runs currently fail with return value mismatch expected 0 actual null, so the stale ret metadata already prevents the intended std/test report contract from being checked.

## 対象

- `tests/stdlib/neplg2_stdlib_map.n.md, nodesrc/test_selfhost_stdlib_map_report_contract.js`

## 根拠

- `tests/stdlib/neplg2_stdlib_map.n.md` の3件は `checks_print_report` と `checks_exit_code` を呼んでいたが、manifest は `ret: 0` のままだった。
- `node nodesrc/test_selfhost_stdlib_map_report_contract.js` で3件の `stdio, normalize_newlines` tag、`exit_code: 0`、deterministic stdout report、report 出力後に exit code を返す順序を固定した。
- actual doctest execution は report metadata ではなく ResourceIR owner summary の残件で compile phase failure になったため、`ISS-20260517T200909433Z-RESOURCEIR-OWNER-SUMMARY-STILL-TREAT-10D9318A` として分離した。

## 問題

tests/stdlib/neplg2_stdlib_map.n.md has three doctests that call checks_print_report and checks_exit_code, but their manifests still use ret: 0 without stdout and exit_code expectations. Focused runs currently fail with return value mismatch expected 0 actual null, so the stale ret metadata already prevents the intended std/test report contract from being checked.

## 影響

Self-host stdlib path mapping and mapped module graph regressions are not represented as deterministic stdout fixtures, and stale ret metadata causes false failures in the runner. This weakens self-host runner parity and keeps .n.md exit semantics ambiguous.

## 修正方針

Move the three doctests to neplg2:test[stdio, normalize_newlines], add exit_code: 0 and deterministic stdout report fixtures, and add a source policy contract for the file.

## 検証

Run the new source policy, run tests/stdlib/neplg2_stdlib_map.n.md with --assert-io, run issue checks, run source policy regressions, and run git diff checks.

## 2026-05-17 修正

`tests/stdlib/neplg2_stdlib_map.n.md` の3件を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout: mlstr:` へ移行した。テスト本体は既に `checks_print_report` / `checks_exit_code` を呼んでいたため、検査ロジックを弱めず manifest 側で report を固定した。

`nodesrc/test_selfhost_stdlib_map_report_contract.js` を追加し、3件が `ret:` に戻らないこと、stdout report 件数が 8 / 9 / 1 件であること、report 表示後に exit code を返すことを source policy にした。`nodesrc/run_source_policy_regressions.js` にも登録した。

実行確認中、`Result` の未解決 variant payload 予約が Copy source まで linear owner として予約する小さな ResourceIR 誤検出を確認し、`ISS-20260517T201648863Z-RESOURCEIR-VARIANT-OWNER-RESERVATION-F28E5200` として修正した。残る stdlib_map relative path resolution の `str` view use_after_move は `ISS-20260517T200909433Z-RESOURCEIR-OWNER-SUMMARY-STILL-TREAT-10D9318A` に分離して継続する。
