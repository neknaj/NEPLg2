---
id: ISS-20260426T020000000Z-STRING-FIND-STUB-7C9A1E2B
title: "alloc/string find is a stub and always returns None"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/string.nepl
source: doc/neplg2/pre_selfhost_audit_20260426.md
---

# ISS-20260426T020000000Z-STRING-FIND-STUB-7C9A1E2B: alloc/string find is a stub and always returns None

## 概要

`stdlib/alloc/string.nepl` の `find` は public API として存在するが、実装は `Option::None<i32>` 固定である。
コメントも未実装であることを明記しており、呼び出し側からは実際に検索できない。

## 根拠

- `stdlib/alloc/string.nepl:2262` の `fn find <(str,str)->Option<i32>> (_s, _pat):` が引数を使わず `Option::None<i32>` を返す。
- self-host compiler の lexer / parser / module path handling は delimiter や prefix / suffix の探索に文字列検索を必要とする。

## 問題

検索 API が存在するため self-host 実装側が利用してしまうと、見つかるべき部分文字列も常に見つからない。
これは `Option::None` を正常な未検出として扱うため、実行時 trap より発見しにくい。

## 影響

source text の token 分割、diagnostic 表示、module path 正規化、CLI option 処理で誤った分岐に入る可能性がある。
セルフホスト開始前に、少なくとも ASCII byte based の仕様と doctest を固定する必要がある。

## 修正方針

`find` の仕様を byte index 返却として明文化し、空 pattern、pattern が source より長い場合、先頭一致、末尾一致、未検出を doctest 化する。
最初は naive search でよいが、`str` の UTF-8 保証 issue と矛盾しないよう、返す位置が byte offset であることを明記する。

## 検証

- `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/string-find-tests.json -j 1`
- self-host lexer の delimiter 探索 fixture で `find` を使ったケースを追加する。
