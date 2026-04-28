---
id: ISS-20260428T005509048Z-GETTING-STARTED-TUTORIAL-NEEDS-FULL--A70253F2
title: "getting_started tutorial needs full rewrite for current NEPLg2"
area: doc
status: open
resolved: false
priority: P1
type: doc
created: 2026-04-28
updated: 2026-04-28
target: "doc/neplg2/tutorial_rewrite_plan.md, tutorials/getting_started/**"
---

# ISS-20260428T005509048Z-GETTING-STARTED-TUTORIAL-NEEDS-FULL--A70253F2: getting_started tutorial needs full rewrite for current NEPLg2

## 概要

tutorials/getting_started is outdated and inconsistent with current NEPLg2. It mixes introductory chapters with competitive programming catalog content, uses older signature explanations, has inconsistent std/test patterns, and does not systematically teach current Result/Option/match/ownership/string-byte-vs-char practices.

## 対象

- `doc/neplg2/tutorial_rewrite_plan.md, tutorials/getting_started/**`

## 関連ドキュメント

- [NEPLg2 tutorial 全面書き直し計画](../../doc/neplg2/tutorial_rewrite_plan.md)

## 根拠

- `tutorials/getting_started/00_index.n.md` は入門章と競技プログラミング catalog を同じ getting started 導線に置いている。
- `tutorials/getting_started/01_hello_world.n.md` は `fn main <()*> ()> ():` のような古い / 読みにくい signature 説明を含む。
- 章ごとに `std/test` の check pattern が揺れており、現在の `checks_*` / `Result` ベースの推奨形として統一されていない。
- `char` / byte / UTF-8 / collection ownership / panic helper 回避など、現在の NEPLg2 で重要な安全方針が tutorial 全体で体系化されていない。

## 問題

tutorials/getting_started is outdated and inconsistent with current NEPLg2. It mixes introductory chapters with competitive programming catalog content, uses older signature explanations, has inconsistent std/test patterns, and does not systematically teach current Result/Option/match/ownership/string-byte-vs-char practices.

## 影響

New users learn obsolete or inconsistent style, and self-host/std feature work has no reliable tutorial surface to demonstrate the current language. Partial edits will keep the tutorial hard to verify because chapter order, examples, and safety guidance are not aligned.

## 修正方針

Rewrite the tutorial according to doc/neplg2/tutorial_rewrite_plan.md. Rebuild the chapter structure around current NEPLg2: minimal execution, std/test, values/functions/control flow, Option/Result, string/byte/char, collection ownership, modules/generics/traits, and a separate advanced competitive track. All examples must be neplg2:test runnable and avoid raw memory or panic-oriented helpers in normal code.

## 検証

Run node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-rewrite.json -j 4 and focused runs per rewritten chapter. Check markdown links by rg or a link checker if available.
