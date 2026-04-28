---
id: ISS-20260428T000149871Z-SELF-HOST-CLI-ARG-CLASSIFIERS-USE-NE-25938DB7
title: "self-host CLI arg classifiers use nested if instead of match"
area: selfhost
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/args.nepl, nodesrc/test_stdlib_match_decision_trees.js"
---

# ISS-20260428T000149871Z-SELF-HOST-CLI-ARG-CLASSIFIERS-USE-NE-25938DB7: self-host CLI arg classifiers use nested if instead of match

## 概要

selfhost_cli_arg_kind、selfhost_cli_parse_target_value、selfhost_cli_parse_emit_value、selfhost_cli_parse_profile_value は、有限の option/value 集合を深い if/else if 連鎖で分類している。stdlib 側で match へ寄せる方針を固定した後に、self-host CLI で同種の不自然な decision tree が再発している。

## 対象

- `stdlib/neplg2/cli/args.nepl, nodesrc/test_stdlib_match_decision_trees.js`

## 根拠

- `stdlib/neplg2/cli/args.nepl` の `selfhost_cli_arg_kind` は `--check`, `--run`, `--attach-source`, `--lib`, `--verbose`, `--target`, `--emit`, `--stdlib-root`, `-o`, `-i`, `--profile`, `--` を深い `if` 連鎖で分類している。
- 同じ file の `selfhost_cli_parse_target_value`, `selfhost_cli_parse_emit_value`, `selfhost_cli_parse_profile_value` も有限集合の value parser だが、enum 変換表として読める `match` ではなく nested if になっている。
- `nodesrc/test_stdlib_match_decision_trees.js` は json/nm/html の classifier だけを対象にしており、selfhost CLI classifier の再発を検出できない。

## 問題

selfhost_cli_arg_kind、selfhost_cli_parse_target_value、selfhost_cli_parse_emit_value、selfhost_cli_parse_profile_value は、有限の option/value 集合を深い if/else if 連鎖で分類している。stdlib 側で match へ寄せる方針を固定した後に、self-host CLI で同種の不自然な decision tree が再発している。

## 影響

option 追加時の抜け漏れや順序依存をレビューしにくく、compiler bug 回避のような不自然な書き方が self-host 実装の標準形として残る。既存の match decision tree regression test も対象 file を限定しており、この再発を検出できていない。

## 修正方針

文字列 option を直接 match できない場合は、token を小さな enum/literal key に正規化する層を用意し、分類後の分岐は match で表す。target/emit/profile は enum 変換表として読み取れる形へ整理し、静的 regression test の対象に selfhost CLI classifier を追加する。

## 検証

nodesrc/test_stdlib_match_decision_trees.js を selfhost CLI classifier まで拡張し、該当関数に if decision tree が戻らないことを確認する。stdlib/neplg2 CLI args doctest も通す。
