---
id: ISS-20260520T033313620Z-SELF-HOST-LEXER-REMAINS-A-FLAT-IMPLE-4314DA2B
title: "self-host lexer remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/lexer/**, doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T033313620Z-SELF-HOST-LEXER-REMAINS-A-FLAT-IMPLE-4314DA2B: self-host lexer remains a flat implementation file

## 概要

The self-host lexer still keeps state model, directive classification, keyword classification, literal scanning, raw block handling, offside stack handling, diagnostic conversion, and public entry points in one 1300+ line file. This contradicts the self-host source tree plan and risks copying the Rust compiler's flat lexer shape into the NEPL implementation.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/lexer/**, doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は Rust `lexer.rs` の flat file を self-host 側へ移植しない方針として、`syntax/lexer/` 配下への責務分割を明記していた。
- 変更前の `stdlib/neplg2/core/syntax/lexer.nepl` は 1300 行を超え、diagnostic model、raw mode enum、directive enum、byte scanner、literal scanner、indent stack、directive/keyword classifier、raw block state、token loop が同居していた。
- この状態で lexer / parser の追加実装を続けると、raw mode や directive の enum/match 検査が巨大 file 内の局所規則として埋もれ、self-host compiler の source tree 設計が Rust 側の flat 構造へ戻る。

## 問題

The self-host lexer still keeps state model, directive classification, keyword classification, literal scanning, raw block handling, offside stack handling, diagnostic conversion, and public entry points in one 1300+ line file. This contradicts the self-host source tree plan and risks copying the Rust compiler's flat lexer shape into the NEPL implementation.

## 影響

Adding parser-facing lexer behavior into the same file makes review and static verification harder, hides responsibility boundaries, and makes it easy to weaken enum/match based checks when raw mode, directive, keyword, or offside rules grow.

## 修正方針

Split the lexer into responsibility modules under syntax/lexer/ while keeping syntax/lexer.nepl as an implementation-free facade. Preserve typed enums and exhaustive matches; do not delete documentation comments to reduce size. Add source policy that prevents facade regression and keeps split files below the agreed threshold.

## 検証

Run focused lexer doctests, the Rust parity lexer harness, the self-host proof/source-tree policy check, issues index/check, and git diff --check.

## 対応結果

- `stdlib/neplg2/core/syntax/lexer.nepl` を implementation-free facade に変更し、実装を `syntax/lexer/*` へ責務分割した。
- 分割先は `diagnostic.nepl`、`byte.nepl`、`literal.nepl`、`token_build.nepl`、`indent.nepl`、`directive.nepl`、`keyword.nepl`、`raw_mode.nepl`、`next.nepl`、`error.nepl`、`tokenize.nepl` である。
- `SelfhostLexerRawMode` と `SelfhostLexerDirectiveKind` は enum のまま保持し、raw mode / directive / keyword の分岐は既存の exhaustive match source policy が分割後の実装を監視するように更新した。
- `nodesrc/test_selfhost_lexer_split_contract.js` を追加し、facade に実装 declaration が戻らないこと、split module 一覧が明示されていること、各 split file が 450 行閾値を超えないこと、submodule が facade を import して依存を曖昧化しないことを固定した。
- `nodesrc/run_source_policy_regressions.js` に split contract を登録した。
- focused lexer doctest は 13/13 passed。各 case の compile は 30-40 秒程度で、今回の local command timeout は外側 360 秒枠が 13 case 合計時間に足りなかったためであり、per-case default 60 秒 timeout の超過ではなかった。

## 検証結果

- `node nodesrc/test_selfhost_lexer_split_contract.js`: passed
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`: passed
- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc/test_stdlib_match_decision_trees.js`: passed
- `node nodesrc/test_selfhost_lexer_rust_parity.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/agent1-lexer-split-lexer.json -j 1 --dist web/dist --assert-io`: 13/13 passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl --no-tree -o tmp/agent1-lexer-split-parser-module.json -j 1 --dist web/dist --assert-io`: 1/1 passed
