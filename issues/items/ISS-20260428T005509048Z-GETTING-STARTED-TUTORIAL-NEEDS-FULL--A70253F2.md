---
id: ISS-20260428T005509048Z-GETTING-STARTED-TUTORIAL-NEEDS-FULL--A70253F2
title: "getting_started tutorial needs full rewrite for current NEPLg2"
area: doc
status: fixed
resolved: true
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

## 2026-04-28 CI ambiguity 部分対応

GitHub Actions run `25045198144` の `tutorials-test` で、`tutorials/getting_started/22_competitive_io_and_arith.n.md::doctest#1/#2` が `D3005 ambiguous overload` になった。原因は `add read sc read sc` の形で `read(StreamScanner)` の戻り値 overload を `add` の期待型だけから解決しようとしていた点である。

今回の対応では、2 つの入力をそれぞれ `let a <i32> read sc` / `let b <i32> read sc`、および `i64` 版の型付き local に分けた。tutorial 本文としても現在の overload 解決規則に沿い、入力値の型を読み取り時点で明示する形へ修正した。

この旧 22 章は全面 rewrite で入門本文から削除し、競技プログラミング向けの内容は `90` 以降の Advanced track へ移した。そのため、この ambiguity は旧章の局所修正ではなく、current tutorial の再構成に吸収した。

## 対応結果

- `tutorials/getting_started/00_index.n.md` を current NEPLg2 向けの章立てへ更新した。
- 古い `02`〜`27` 章を削除し、`02_test_harness`〜`24_project_byte_output`、`90` 以降の Advanced track、`99_migration_notes` へ再構成した。
- 本文の runnable example を `std/test` / `Result` / `Option` / `match` / `char` / `str_char_*` / `Vec` の current API に合わせた。
- 競技プログラミング catalog は入門本文から外し、Advanced track の設計メモへ分離した。
- `alloc_raw` / `MemPtr` / `unwrap_ok` / `uwok` / old spaced impure-unit signature が runnable example に戻らないよう `nodesrc/test_tutorial_getting_started_current_style.js` を追加した。
- `doc/neplg2/tutorial_rewrite_plan.md` に char 実装後の扱いと実装結果を追記した。

## 修正後の検証

- `trunk build`: pass
- `node nodesrc/tests.js -i tutorials/getting_started/11_bytebuf_and_text_io.n.md -i tutorials/getting_started/13_vec_basics.n.md -i tutorials/getting_started/14_collection_reads.n.md --no-tree -o tmp/tutorials-rewrite-focused-fixes.json -j 3`: 3/3 passed
- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-rewrite.json -j 4`: 24/24 passed
- `trunk build`: pass after rebase onto `17ca4b2`
- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-rewrite-after-rebase.json -j 4`: 24/24 passed
- `trunk build`: pass after rebase onto `0165fa0`
- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-rewrite-after-second-rebase.json -j 4`: 24/24 passed
- `node nodesrc/test_tutorial_getting_started_current_style.js`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
