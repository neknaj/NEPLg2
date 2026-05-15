---
id: ISS-20260515T132628228Z-CORE-TRAITS-DOC-COMMENT-DOCTESTS-STI-69D31B25
title: "core traits doc-comment doctests still use legacy checks_exit_code reports"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/core/traits/{debug,deserialize,hash,serialize,stringify}.nepl"
---

# ISS-20260515T132628228Z-CORE-TRAITS-DOC-COMMENT-DOCTESTS-STI-69D31B25: core traits doc-comment doctests still use legacy checks_exit_code reports

## 概要

Core trait documentation examples still use legacy checks_exit_code or result_exit_code without fixing the canonical TestReport stdout contract.

## 対象

- `stdlib/core/traits/{debug,deserialize,hash,serialize,stringify}.nepl`

## 根拠

- `stdlib/core/traits/stringify.nepl` / `serialize.nepl` / `hash.nepl` / `debug.nepl` は doc-comment doctest で `checks_exit_code` だけを使い、stdout に assertion label / expected / actual を固定していなかった。
- `stdlib/core/traits/deserialize.nepl` は `result_exit_code` のみで成功を表していた。focused doctest 移行時に、`Deserialize<u8>` の `cast` import / typed cast 式も現在の parser/typecheck で compile できない潜在不具合として露出した。

## 問題

Core trait documentation examples still use legacy checks_exit_code or result_exit_code without fixing the canonical TestReport stdout contract.

## 影響

Documentation examples can pass by exit code while losing assertion labels and expected/actual stdout details required by the shared doctest contract.

## 修正方針

Migrate the affected doc-comment doctests to neplg2:test[stdio, normalize_newlines], exit_code metadata, and test_report_print_stdout/test_report_exit_code.

## 対応

- `Stringify` / `Serialize` / `Hash` / `Debug` / `Deserialize` の public doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、canonical `TestReport` stdout へ移行した。
- `Deserialize<u8>` の実装で `core/cast` を明示 import し、`let byte <u8> cast v` として型注釈付き cast を分離した。これにより doc-comment doctest だけでなく trait module 全体の compile drift も解消した。
- `nodesrc/test_core_traits_doc_report_contract.js` を追加し、5 file の doc-comment doctest が `checks_exit_code` / `result_exit_code` へ戻らないことを source policy に固定した。
- `nodesrc/run_source_policy_regressions.js` に同 contract を追加した。

## 検証

Run focused doc-comment doctests for updated trait files, the std/test report contract policy, issues check, and diff whitespace check.

- `node nodesrc/tests.js -i stdlib/core/traits/stringify.nepl -i stdlib/core/traits/serialize.nepl -i stdlib/core/traits/hash.nepl -i stdlib/core/traits/debug.nepl -i stdlib/core/traits/deserialize.nepl --no-tree -o tmp/agent1-core-traits-doc-report.json -j 1 --dist web/dist --assert-io`: 5 passed
- `node --check nodesrc/test_core_traits_doc_report_contract.js`: pass
- `node nodesrc/test_core_traits_doc_report_contract.js`: pass
- `node --check nodesrc/run_source_policy_regressions.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: source-policy warning なし
