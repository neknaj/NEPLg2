---
id: ISS-20260604T034256176Z-SELFHOST-PARSER-CAN-ACCEPT-UNCLOSED--C0703412
title: "selfhost parser can accept unclosed raw blocks and excess dedent as Ok"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl, stdlib/neplg2/core/syntax/parser/module_parser/state.nepl"
---

# ISS-20260604T034256176Z-SELFHOST-PARSER-CAN-ACCEPT-UNCLOSED--C0703412: selfhost parser can accept unclosed raw blocks and excess dedent as Ok

## 概要

Subagent audit found raw/offside parser state where active raw EOF can return Ok ast and dedent depth can saturate to 0. plan.md states indentation errors are errors, and Zenn guidance requires invalid states to be explicit Result/enum outcomes.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl, stdlib/neplg2/core/syntax/parser/module_parser/state.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found raw/offside parser state where active raw EOF can return Ok ast and dedent depth can saturate to 0. plan.md states indentation errors are errors, and Zenn guidance requires invalid states to be explicit Result/enum outcomes.

## 影響

Malformed source can be accepted by selfhost parser paths, weakening parser diagnostics and any tooling that depends on selfhost parsing.

## 修正方針

Add diagnostic variants such as InvalidDedent and UnclosedRawBlock, make parser state transitions return Result, and remove saturating dedent for invalid input.

## 検証

Add regular tests for unclosed raw block EOF, extra dedent, valid nested block, and recovery diagnostics through source and hand-built token streams.

## 対応

- `SelfhostParserDiagnosticCode::RawBlockUnclosed` と `SelfhostParserDiagnosticCode::InvalidDedent` を追加し、raw block EOF と余分な `Dedent` を typed parser diagnostic として扱うようにした。
- `selfhost_parser_depth_dec` の 0 saturating helper を廃止し、`selfhost_parser_depth_after_dedent -> Option i32` で invalid dedent を明示する状態遷移へ変更した。
- parser loop の EOF / token stream end 処理で pending raw mode と active raw mode を正常終了にせず、`RawBlockExpectedIndent` または `RawBlockUnclosed` へ落とすようにした。
- `tests/stdlib/neplg2_parser.n.md` に pending raw EOF、active raw EOF、top-level excess dedent の regression を追加した。
- `nodesrc/test_selfhost_parser_invalid_state_contract.js` を追加し、saturating dedent helper の復帰、raw EOF の Ok 受理、typed diagnostic mapping の抜けを検出する。
