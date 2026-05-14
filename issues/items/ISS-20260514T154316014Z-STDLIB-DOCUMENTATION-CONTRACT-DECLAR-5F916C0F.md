---
id: ISS-20260514T154316014Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-5F916C0F
title: "Stdlib documentation contract declaration doctest baseline regressed"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/**/*.nepl, nodesrc/test_stdlib_documentation_contract.js, doc/neplg2/stdlib_documentation_contract_plan.md"
---

# ISS-20260514T154316014Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-5F916C0F: Stdlib documentation contract declaration doctest baseline regressed

## 概要

The global stdlib documentation contract policy reports declaration doctest gaps increased from the frozen baseline 1032 to 1052. This is a regression in executable documentation coverage and should not be hidden by warn-only source policy execution.

## 対象

- `stdlib/**/*.nepl, nodesrc/test_stdlib_documentation_contract.js, doc/neplg2/stdlib_documentation_contract_plan.md`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` が `stdlib declaration doctest gaps increased: 1052 > 1032` で失敗する。
- 2026-05-15 Agent 1 の再集計では、`declarations=1773`、`declarationNoDoc=531`、`declarationNoDoctest=1052`、frozen baseline との差分は `+20` である。
- `node nodesrc/run_source_policy_regressions.js --warn-only` では、SourceText / string facade policy を修正した後もこの documentation contract warning だけが残る。
- 不足数の多い領域には `stdlib/core/cast.nepl`、`stdlib/alloc/diag/error/{diag,outcome}.nepl`、`stdlib/core/char.nepl`、`stdlib/alloc/collections/hashmap/storage.nepl`、`stdlib/alloc/string/integer/common/u128.nepl`、`stdlib/alloc/string/scanner.nepl` などが含まれる。

## 問題

The global stdlib documentation contract policy reports declaration doctest gaps increased from the frozen baseline 1032 to 1052. This is a regression in executable documentation coverage and should not be hidden by warn-only source policy execution.

## 影響

Source policy no longer gives a clean signal, and new or changed stdlib APIs may lack typical-use doctests despite the project rule that documentation comments and doctests are part of the API contract.

## 修正方針

Audit the declarations that contributed to the 20-gap regression, add meaningful n.md-style doctests instead of lowering the bar, then reduce or update the baseline only after the measured gap is actually fixed.

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js, node nodesrc/run_source_policy_regressions.js --warn-only, issue checks, and focused doctests for files that receive new examples.

## 2026-05-15 Agent 1 triage

この issue は現在 open とする。baseline を `1052` へ上げて warning を消すだけでは、documentation を API contract として扱う方針に反する。修正時は、単に `neplg2:test` marker を増やすのではなく、各 declaration の典型的な使い方、所有権、失敗時 contract、計算量を説明する既存 documentation に合う doctest を追加する。

優先順位:

- まず直近のStage 6変更で増えた可能性が高い StringBuilder / ByteBuilder / collection owner boundary 周辺を確認する。
- 次に `doc/neplg2/stdlib_documentation_contract_plan.md` の Stage 1/2 方針に沿い、core / alloc の利用頻度が高い宣言から baseline を下げる。
- コンパイラ静的検査作業中にstdlib側の即時修正が必要でない場合、この issue はdocs整備フェーズへ回す。
