---
id: ISS-20260514T154316014Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-5F916C0F
title: "Stdlib documentation contract declaration doctest baseline regressed"
area: stdlib
status: fixed
resolved: true
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

## 2026-05-15 Agent 1 resolution

直近の owner boundary / facade 分割で増えた stdlib declaration doctest gap を、baseline を緩めずに executable documentation 側で解消した。

対応内容:

- `HashMap` / `HashSet` の public API に、`new` / `with_capacity` / `insert` / `get` / `contains` / `remove` / `len` / `free` の典型的な owner flow doctest を追加した。
- `HashMapBucketState` / `HashSetBucketState` と insert slot 型に、numeric sentinel ではなく enum / struct を match・field access で使う例を追加した。
- `hashmap_normalize_capacity` / `hashmap_load_limit` / `hashset_normalize_capacity` / `hashset_load_limit` に、容量下限と 75% load limit の executable example を追加した。
- `vec/storage/api` と `vec/sort/merge` 周辺に、public facade から利用する例と、raw helper が public facade へ漏れない compile_fail doctest を追加した。
- `merge/buffer` / `merge/api` は source-policy の module split 上限を維持し、重い重複 doctest を内部 raw helper に押し込まない形へ整理した。

検証:

- `node nodesrc/test_stdlib_documentation_contract.js`
  - `moduleNoDoctest=309`
  - `declarationNoDoctest=1032`
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap/api.nepl -i stdlib/alloc/collections/hashmap/types.nepl -i stdlib/alloc/collections/hashmap/storage.nepl -i stdlib/alloc/collections/hashset/api.nepl -i stdlib/alloc/collections/hashset/types.nepl -i stdlib/alloc/collections/hashset/storage.nepl -i stdlib/alloc/collections/vec/storage/api.nepl -i stdlib/alloc/collections/vec/sort/merge.nepl -i stdlib/alloc/collections/vec/sort/merge/api.nepl -i stdlib/alloc/collections/vec/sort/merge/buffer.nepl -i stdlib/alloc/collections/vec/sort/merge/range.nepl --no-tree -o tmp/agent1-stdlib-doc-contract-doctests.json -j 1 --dist web/dist --assert-io`
  - `total=37, passed=37`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
  - all source policy regressions passed
