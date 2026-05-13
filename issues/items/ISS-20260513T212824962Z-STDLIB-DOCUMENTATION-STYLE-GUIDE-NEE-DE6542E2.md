---
id: ISS-20260513T212824962Z-STDLIB-DOCUMENTATION-STYLE-GUIDE-NEE-DE6542E2
title: "stdlib documentation style guide needs updated audit policy"
area: docs
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-13
updated: 2026-05-13
target: "doc/neplg2/stdlib_documentation_style_guide.md, doc/neplg2/stdlib_documentation_contract_plan.md, doc/stdlib_doc_comment_policy.md"
---

# ISS-20260513T212824962Z-STDLIB-DOCUMENTATION-STYLE-GUIDE-NEE-DE6542E2: stdlib documentation style guide needs updated audit policy

## 概要

stdlib documentation policy exists, but it is split between an older root policy and a coverage baseline plan. It does not yet consolidate the latest guidance for Japanese n.md-style comments, ruby usage, computer-science accuracy, typical-use doctests, and raw/performance stdlib examples such as kpgraph.

## 対象

- `doc/neplg2/stdlib_documentation_style_guide.md, doc/neplg2/stdlib_documentation_contract_plan.md, doc/stdlib_doc_comment_policy.md`

## 根拠

- `doc/stdlib_doc_comment_policy.md` は 2026-03-15 時点の root 方針であり、手書き・nm ruby・doctest の責務分離を示しているが、現在の NEPLg2 stdlib の static-check / raw-memory-backed API 移行とは接続が薄い。
- `doc/neplg2/stdlib_documentation_contract_plan.md` は coverage baseline と段階計画を定めているが、文章品質、自然な日本語、計算機科学的正確性、典型例としての doctest の選び方は詳細化していない。
- `stdlib/kp/kpgraph.nepl` は module doc と module doctest があり、密行列・BFS・O(n^2)・0-index/1-index を説明している一方、helper / owner-bearing type の declaration doc は raw memory、解放責務、失敗時 owner contract の説明が薄い。

## 問題

stdlib documentation policy exists, but it is split between an older root policy and a coverage baseline plan. It does not yet consolidate the latest guidance for Japanese n.md-style comments, ruby usage, computer-science accuracy, typical-use doctests, and raw/performance stdlib examples such as kpgraph.

## 影響

Documentation comments can satisfy baseline coverage while still being thin, inconsistent, or misleading. If doctests are used as broad regression tests instead of typical usage examples, API docs become hard to read and future stdlib/self-host users lose the source-level contract.

## 修正方針

Audit kpgraph and existing documentation policy, create a NEPLg2 stdlib documentation style guide, and link the contract plan/root policy to it. Clarify module/type/function documentation requirements, Japanese/ruby style, CS accuracy, doctest responsibilities, and migration priorities.

## 検証

Run stdlib documentation contract policy and issue metadata checks.

## 対応内容

- `doc/neplg2/stdlib_documentation_style_guide.md` を追加した。
- `kpgraph` を具体例として、良い点と不足点を整理した。
- module / type / function documentation で書くべき内容を、所有権、effect、raw memory、target、計算量、失敗時 contract の観点で定義した。
- 日本語文体、nm ruby、見出し、計算機科学的な正確性、doctest と `tests/` の責務分離を整理した。
- `kp` performance layer では、入力制約、計算量、raw boundary、cleanup、一般 stdlib へ昇格する時の修正点を書く方針を追加した。
- `doc/neplg2/stdlib_documentation_contract_plan.md` から style guide へリンクし、coverage baseline と文章品質方針を分離した。
- root の `doc/stdlib_doc_comment_policy.md` を更新し、NEPLg2 現行 stdlib の詳細方針は style guide を正とすることを明記した。

## 検証結果

- `node nodesrc/test_stdlib_documentation_contract.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
