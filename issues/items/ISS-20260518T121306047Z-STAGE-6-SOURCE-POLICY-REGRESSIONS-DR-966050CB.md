---
id: ISS-20260518T121306047Z-STAGE-6-SOURCE-POLICY-REGRESSIONS-DR-966050CB
title: "Stage 6 source-policy regressions drift after stdlib boundary refactors"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nodesrc/test_stdlib_string_facade_boundary.js, nodesrc/test_stdlib_cliarg_report_contract.js, nodesrc/test_doctest_diag_code_metadata.js, nodesrc/test_selfhost_diag_code_enum.js, tests/stdlib/memory_safety.n.md"
---

# ISS-20260518T121306047Z-STAGE-6-SOURCE-POLICY-REGRESSIONS-DR-966050CB: Stage 6 source-policy regressions drift after stdlib boundary refactors

## 概要

Stage 6 raw-memory boundary refactors moved proof-bearing helpers and doctest counts, but several source-policy regressions still asserted the previous module layout or missing diag metadata.

## 対象

- `nodesrc/test_stdlib_string_facade_boundary.js, nodesrc/test_stdlib_cliarg_report_contract.js, nodesrc/test_doctest_diag_code_metadata.js, nodesrc/test_selfhost_diag_code_enum.js, tests/stdlib/memory_safety.n.md`

## 根拠

- `nodesrc/test_stdlib_string_facade_boundary.js` は `builder_ext.nepl` が直接 raw memory evidence を持つ前提だったが、ByteBuilder typed source helper へ委譲した後は wrapper module として扱うべきになった。
- `nodesrc/test_stdlib_cliarg_report_contract.js` は `stdlib/tests/cliarg.n.md` の doctest 数を旧 6 件に固定しており、bounded C string conversion の stdout report doctest を拾えていなかった。
- `nodesrc/test_doctest_diag_code_metadata.js` は `tests/stdlib/memory_safety.n.md` の compile_fail に missing `diag_code` を検出していた。
- `nodesrc/test_selfhost_diag_code_enum.js` は reporter facade を読んでおり、split 後の `reporter/render/single.nepl` にある `selfhost_diag_code_name` 使用を確認できていなかった。

## 問題

Stage 6 raw-memory boundary refactors moved proof-bearing helpers and doctest counts, but several source-policy regressions still asserted the previous module layout or missing diag metadata.

## 影響

Warn-only policy failures can hide real static-check regressions and make CI noise indistinguishable from intentional lint warnings.

## 修正方針

Update the affected source policies to follow the new typed helper boundaries, pin missing compile_fail diag metadata, and keep reporter diagnostic-code checks pointed at the split implementation module.

## 対応内容

- `builder_ext.nepl` を raw evidence required list から外し、StringBuilder wrapper として direct raw memory evidence を持たないことを確認する側へ移した。
- cliarg report contract を doctest 7 件へ更新し、`cliarg_cstr_bounded_conversion_reports` の stdout report / exit code contract を監視対象へ追加した。
- `alloc/string/byte_index` の arbitrary `i32` から raw byte reader を呼べない compile_fail に `diag_code: type.overload.no_match` を付与した。
- selfhost diagnostic reporter policy は split 後の render module を読み、`selfhost_diag_code_name *field::get_ref diag "code"` を確認する形へ更新した。
- 同時監査で残った `string_byte_at_checked_or_unreachable` の trap-based public helper は、設計修正が必要な別 issue `ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237` として分離した。

## 検証

- `node nodesrc/test_stdlib_string_facade_boundary.js`: passed
- `node nodesrc/test_stdlib_cliarg_report_contract.js`: passed
- `node nodesrc/test_selfhost_diag_code_enum.js`: passed
- `node nodesrc/test_doctest_diag_code_metadata.js`: passed
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-source-policy-stage6-drift-memory-safety.json -j 1 --dist web\dist --assert-io`: total=60, passed=60
- `node nodesrc/run_source_policy_regressions.js --warn-only`: 対象 4 policy は pass。残 warning は `ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237` として分離。
