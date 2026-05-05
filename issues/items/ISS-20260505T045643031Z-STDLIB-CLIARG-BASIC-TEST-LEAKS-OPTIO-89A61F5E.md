---
id: ISS-20260505T045643031Z-STDLIB-CLIARG-BASIC-TEST-LEAKS-OPTIO-89A61F5E
title: "stdlib cliarg basic test leaks Option<str> payload locals"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-05-05
updated: 2026-05-05
target: stdlib/tests/cliarg.n.md
---

# ISS-20260505T045643031Z-STDLIB-CLIARG-BASIC-TEST-LEAKS-OPTIO-89A61F5E: stdlib cliarg basic test leaks Option<str> payload locals

## 概要

stdlib/tests/cliarg.n.md の cliarg_basic が cliarg_get の戻り値 Option<str> を _a/_b/_p に束縛したまま消費せず、Resource IR が Option::Some payload の owned str leak を報告する。base commit 448ededb でも再現する既存問題。

## 対象

- `stdlib/tests/cliarg.n.md`

## 根拠

- 未記入

## 問題

stdlib/tests/cliarg.n.md の cliarg_basic が cliarg_get の戻り値 Option<str> を _a/_b/_p に束縛したまま消費せず、Resource IR が Option::Some payload の owned str leak を報告する。base commit 448ededb でも再現する既存問題。

## 影響

cliarg の n.md suite が 5/6 passed で止まり、stdlib/env/cliarg の実装変更検証時に本体の回帰と test fixture owner leak が混ざる。

## 修正方針

cliarg_basic を is_none<str> や match で Option payload を消費する形へ書き換え、所有権検査に通る test fixture にする。

## 検証

node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-tree -o tmp/cliarg-basic-owner-fixed.json -j 1 が 6/6 passed になること。
