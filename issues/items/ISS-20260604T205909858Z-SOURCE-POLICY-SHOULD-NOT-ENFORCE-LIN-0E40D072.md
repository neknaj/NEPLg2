---
id: ISS-20260604T205909858Z-SOURCE-POLICY-SHOULD-NOT-ENFORCE-LIN-0E40D072
title: "source policy should not enforce line count limits"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: test
created: 2026-06-04
updated: 2026-06-04
target: "nodesrc/test_* nodesrc/source_policy"
---

# ISS-20260604T205909858Z-SOURCE-POLICY-SHOULD-NOT-ENFORCE-LIN-0E40D072: source policy should not enforce line count limits

## 概要

Source policy line-count budgets conflict with the current documentation-comment policy because careful comments, doctests, contracts, complexity notes, and constraints should not be constrained by numeric file or implementation line limits.

## 対象

- `nodesrc/test_* nodesrc/source_policy`

## 根拠

- Zenn の開発方針と `doc/stdlib_doc_comment_policy.md` は、ドキュメントコメントを API contract として扱い、目的、アルゴリズム、計算量、制約、doctest を丁寧に残すことを要求している。
- 行数上限は、コメント行を数えるかどうかに関係なく、説明を短くする誘因になり得る。source policy は責務境界を守るための検査であり、ドキュメント量を制限するための検査ではない。
- 責務再集中は、facade に実装本体が戻ったか、分割 module が re-export されているか、raw memory / unsafe helper / owner boundary が正しい場所にあるか、という構造的な条件で検査できる。

## 問題

Source policy line-count budgets conflict with the current documentation-comment policy because careful comments, doctests, contracts, complexity notes, and constraints should not be constrained by numeric file or implementation line limits.

## 影響

Agents may reduce useful documentation or avoid necessary comments to satisfy artificial line-count checks instead of preserving structural responsibility boundaries.

## 修正方針

Remove line-count limit checks from source-policy tests, keep structural responsibility checks such as facade re-exports and forbidden ownership boundaries, remove unused line-count helpers, and add a regression guard that rejects new line-count limit checks.

## 対応

- `nodesrc/test_selfhost_*_split_contract.js`、Rust compiler responsibility policy、stdlib boundary policy から、file lines / implementation lines / split threshold / line budget による上限検査を削除した。
- facade re-export、implementation body の逆流禁止、module ownership、raw memory / unsafe unwrap boundary、typed enum / proof surface などの構造的な責務検査は残した。
- `implementationLineCount` helper と `rust_source_lines.js` を削除し、source policy に行数測定 API が残らないようにした。
- `nodesrc/test_source_policy_no_line_count_limits.js` を追加し、`nodesrc/test_*.js` と `nodesrc/source_policy` に行数上限検査が再導入されないことを確認する。
- `doc/stdlib_doc_comment_policy.md` と parser/backend responsibility split plan を、行数上限ではなく構造的責務境界を監視する方針へ更新した。

## 検証

- `node --check` for 41 changed JavaScript files: pass
- changed `nodesrc/test_*.js` files: pass
- `node nodesrc/test_source_policy_no_line_count_limits.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: exit 0. Four warnings remain from unrelated active worktree / existing policy gaps: `test_neplg21_vec_type_arity_imports.js`, `test_resource_gate_order.js`, `test_diagnostic_code_first_boundary.js`, and `test_nepl_doc_report_metadata_policy.js`.
