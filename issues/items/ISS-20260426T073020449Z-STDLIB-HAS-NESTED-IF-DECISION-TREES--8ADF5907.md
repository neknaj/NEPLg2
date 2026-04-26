---
id: ISS-20260426T073020449Z-STDLIB-HAS-NESTED-IF-DECISION-TREES--8ADF5907
title: "stdlib has nested if decision trees that should be match expressions"
area: stdlib
status: verified
resolved: true
priority: P2
type: maintenance
created: 2026-04-26
updated: 2026-04-26
target: stdlib/
---

# ISS-20260426T073020449Z-STDLIB-HAS-NESTED-IF-DECISION-TREES--8ADF5907: stdlib has nested if decision trees that should be match expressions

## 概要

有限のリテラル値・enum variant・token kind の分岐を深い if / else if 連鎖で書いている箇所があり、意図としては match で表現すべき判定が読みづらくなっている。今回の json_escape の文字 escape 分岐のように、compiler bug 回避や一時実装の名残として不自然な制御構造が stdlib に残り得る。

## 対象

- `stdlib/`

## 根拠

- `stdlib/alloc/encoding/json.nepl` の `json_escape` は、JSON 文字列 escape 対象の固定文字集合を `if` の深いネストで判定している。
- 同種の書き方は、compiler bug 回避や match lowering の制約を隠したまま stdlib に残る可能性があるため、stdlib 全体の監査対象として追跡する。

## 問題

有限のリテラル値・enum variant・token kind の分岐を深い if / else if 連鎖で書いている箇所があり、意図としては match で表現すべき判定が読みづらくなっている。今回の json_escape の文字 escape 分岐のように、compiler bug 回避や一時実装の名残として不自然な制御構造が stdlib に残り得る。

## 影響

stdlib の仕様分岐がデータごとの対応表として読めず、抜け漏れ・順序依存・将来の追加漏れをレビューで見つけにくい。compiler bug 回避の workaround が通常の実装として残ると、compiler 側の不具合や未整備な match lowering を隠してしまう。

## 修正方針

stdlib 全体を監査し、有限集合の分岐・literal dispatch・variant dispatch は原則 match へ置き換える。match 化できない場合は stdlib 側の workaround で済ませず、阻害している compiler bug を別 Issue として登録し、回帰テストを追加する。json_escape の文字 escape 分岐はこの方針の先行対象として扱う。

## 検証

stdlib の対象箇所を match 化した上で、該当 module の doctest と stdlib 全体 doctestを実行する。必要に応じて literal match / wildcard match / enum match の compiler regression test を追加し、不自然な if 連鎖が再発していないことを静的検索で確認する。

## 対応

- `json_escape_kind` / `nm_json_escape_kind` / `html_escape_kind` / `html_heading_kind` を、mut 変数へ順次代入する `if` decision tree から scalar literal `match` へ置き換えた。
- 対象は enum classifier として読める有限値分岐に絞り、parser の状態遷移、範囲判定、長さ確認などの条件分岐は通常の制御構造として残した。
- `nodesrc/test_stdlib_match_decision_trees.js` を追加し、対象 classifier が `match` と wildcard arm を持ち、`if:` に戻らないことを静的に固定した。
- `tests/stdlib/nm.n.md` に H2 rendering の回帰テストを追加し、`html_heading_kind` の literal dispatch 経路を実行時にも確認した。

## 検証結果

- `node nodesrc/test_stdlib_match_decision_trees.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-match-decision-trees.json -j 1`: total=5, passed=5
- `node nodesrc/tests.js -i tests/stdlib/json_typed_values.n.md --no-tree -o tmp/json-match-decision-trees.json -j 1`: total=7, passed=7
- `node nodesrc/tests.js -i tests/compiler/match_literal_patterns.n.md --no-tree -o tmp/match-literal-patterns-stdlib-use.json -j 1`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-match-decision-trees.json -j 4`: total=404, passed=404
- `trunk build`: pass（既存 warning のみ）
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-match-decision-trees.json`: 13/13 passed
