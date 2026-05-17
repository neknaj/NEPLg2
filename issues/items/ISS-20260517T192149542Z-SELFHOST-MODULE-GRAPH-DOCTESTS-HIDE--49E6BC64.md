---
id: ISS-20260517T192149542Z-SELFHOST-MODULE-GRAPH-DOCTESTS-HIDE--49E6BC64
title: "selfhost module graph doctests hide std/test reports in metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-18
target: "tests/stdlib/neplg2_module_graph.n.md, nodesrc/test_selfhost_module_graph_report_contract.js"
---

# ISS-20260517T192149542Z-SELFHOST-MODULE-GRAPH-DOCTESTS-HIDE--49E6BC64: selfhost module graph doctests hide std/test reports in metadata

## 概要

tests/stdlib/neplg2_module_graph.n.md has three doctests that call checks_print_report and checks_exit_code, but their manifests still use ret: 0 without stdout and exit_code expectations. The import graph reports are emitted but not fixture-checked.

## 対象

- `tests/stdlib/neplg2_module_graph.n.md, nodesrc/test_selfhost_module_graph_report_contract.js`

## 根拠

- `tests/stdlib/neplg2_module_graph.n.md` の3件はすべて `checks_print_report` で assertion report をstdoutへ出してから `checks_exit_code` を返していた。
- manifest は3件とも `ret: 0` のままで、report stdout と process exit code をfixture契約として固定していなかった。
- focused run ではtransitive graph成功系が9件、missing module / cycle診断系が2件ずつの deterministic report を出しており、検査対象にできる情報が捨てられていた。

## 問題

tests/stdlib/neplg2_module_graph.n.md has three doctests that call checks_print_report and checks_exit_code, but their manifests still use ret: 0 without stdout and exit_code expectations. The import graph reports are emitted but not fixture-checked.

## 影響

Self-host module graph regressions can change assertion count, import edge ordering, or diagnostic details while the doctests still pass by return value only. This weakens self-host runner parity and keeps .n.md exit semantics ambiguous.

## 修正方針

Move the three doctests to neplg2:test[stdio, normalize_newlines], add exit_code: 0 and deterministic stdout report fixtures, and add a source policy contract for the file.

## 対応内容

- 3件すべてを `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- transitive graph成功系9件、missing import / cycle診断系2件ずつの report をfixtureとして固定した。
- `nodesrc/test_selfhost_module_graph_report_contract.js` を追加し、`ret:` 不使用、stdout report、`exit_code: 0`、report出力順を検査するようにした。
- `nodesrc/run_source_policy_regressions.js` へ新contractを登録した。
- 親 issue `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` に進捗を追記した。

## 検証

- `node nodesrc/test_selfhost_module_graph_report_contract.js`: passed
- `node nodesrc/tests.js -i tests\stdlib\neplg2_module_graph.n.md --no-tree -o tmp\agent1-neplg2-module-graph-report-metadata.json -j 1 --dist web\dist --assert-io`: total=3, passed=3
- `node nodesrc/issues.js check --dir issues`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `git diff --check`: CRLF warnings only
