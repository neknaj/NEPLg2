---
id: ISS-20260517T191427404Z-SELFHOST-IMPORT-SPEC-DOCTESTS-HIDE-S-3E0A657F
title: "selfhost import spec doctests hide std/test reports in metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-18
target: "tests/stdlib/neplg2_import_spec.n.md, nodesrc/test_selfhost_import_spec_report_contract.js"
---

# ISS-20260517T191427404Z-SELFHOST-IMPORT-SPEC-DOCTESTS-HIDE-S-3E0A657F: selfhost import spec doctests hide std/test reports in metadata

## 概要

tests/stdlib/neplg2_import_spec.n.md has three doctests that call checks_print_report and checks_exit_code, but their manifests still use ret: 0 without stdout and exit_code expectations. The import parser reports are emitted but not fixture-checked.

## 対象

- `tests/stdlib/neplg2_import_spec.n.md, nodesrc/test_selfhost_import_spec_report_contract.js`

## 根拠

- `tests/stdlib/neplg2_import_spec.n.md` の3件はすべて `checks_print_report` で assertion report をstdoutへ出してから `checks_exit_code` を返していた。
- manifest は3件とも `ret: 0` のままで、process exit-code と言語戻り値の責務が混在していた。
- focused run では成功系が7件、エラー診断系が2件ずつの deterministic report を出しており、fixture化できる情報を検査していなかった。

## 問題

tests/stdlib/neplg2_import_spec.n.md has three doctests that call checks_print_report and checks_exit_code, but their manifests still use ret: 0 without stdout and exit_code expectations. The import parser reports are emitted but not fixture-checked.

## 影響

Self-host import directive parsing and diagnostic regressions can change assertion count, ordering, or expected diagnostic details while the doctests still pass by return value only. This keeps .n.md exit semantics ambiguous.

## 修正方針

Move the three doctests to neplg2:test[stdio, normalize_newlines], add exit_code: 0 and deterministic stdout report fixtures, and add a source policy contract for the file.

## 対応内容

- 3件すべてを `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- import spec parserの成功系7件、missing quote / trailing text診断系2件ずつの report を fixtureとして固定した。
- `nodesrc/test_selfhost_import_spec_report_contract.js` を追加し、`ret:` 不使用、stdout report、`exit_code: 0`、report出力順を検査するようにした。
- `nodesrc/run_source_policy_regressions.js` へ新contractを登録した。
- 親 issue `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` に進捗を追記した。

## 検証

- `node nodesrc/test_selfhost_import_spec_report_contract.js`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_import_spec.n.md -n 1 --assert-io --dist web\dist`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_import_spec.n.md -n 2 --assert-io --dist web\dist`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_import_spec.n.md -n 3 --assert-io --dist web\dist`: passed
- `node nodesrc/tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\agent1-neplg2-import-spec-report-metadata.json -j 1 --dist web\dist --assert-io`: total=3, passed=3
- `node nodesrc/issues.js check --dir issues`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `git diff --check`: CRLF warnings only
