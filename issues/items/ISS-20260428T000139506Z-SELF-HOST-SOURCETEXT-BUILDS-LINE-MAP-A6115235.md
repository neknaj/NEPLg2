---
id: ISS-20260428T000139506Z-SELF-HOST-SOURCETEXT-BUILDS-LINE-MAP-A6115235
title: "self-host SourceText builds line maps with per-byte recursion"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/infra/text.nepl, tests/stdlib/neplg2_text.n.md, nodesrc/test_selfhost_source_text_no_recursive_line_map.js"
---

# ISS-20260428T000139506Z-SELF-HOST-SOURCETEXT-BUILDS-LINE-MAP-A6115235: self-host SourceText builds line maps with per-byte recursion

## 概要

source_text_collect_line_starts は source の各 byte ごとに再帰し、LF/CR/CRLF を見つけるたびに同じ関数を呼び直す。大きな stdlib file や将来の generated source を読むと、line map 構築だけで call stack を byte 長に比例して消費する。

## 対象

- `stdlib/neplg2/core/infra/text.nepl, tests/stdlib/neplg2_text.n.md`

## 根拠

- `stdlib/neplg2/core/infra/text.nepl` の `source_text_collect_line_starts` は `idx` を 1 byte または CRLF 2 byte 進めるたびに自分自身を呼び直す。
- `doc/neplg2/self_host_plan.md` の S4 は deep traversal を explicit stack で実装する方針を置いており、source text でも入力長比例の call stack 消費は避けるべきである。
- 現在の stdlib には `stdlib/core/math.nepl` 4435 lines、`stdlib/alloc/string.nepl` 2479 lines など大きな入力が既にあり、self-host compiler はこれらを継続的に読む。

## 問題

source_text_collect_line_starts は source の各 byte ごとに再帰し、LF/CR/CRLF を見つけるたびに同じ関数を呼び直す。大きな stdlib file や将来の generated source を読むと、line map 構築だけで call stack を byte 長に比例して消費する。

## 影響

self-host compiler が stdlib/core/math.nepl や alloc/string.nepl 程度の入力を継続的に扱う段階で、診断ではなく stack overflow / trap に到達する可能性がある。S4 で deep HIR traversal を explicit stack にする方針とも矛盾する。

## 修正方針

source_text_collect_line_starts を index 付き while/loop 相当の反復 helper に置き換える。CRLF の 2 byte advance と allocation failure の Result 伝播は維持し、line start table の契約を変えない。

## 検証

長い 1 行、数万行、CRLF 混在の large source fixture を追加し、source_text_new と offset lookup が panic/trap せず O(n) で完走することを stdlib/neplg2_text focused test で確認する。

## 解決

- `source_text_collect_line_starts` を per-byte 再帰から `while` による 1-pass 走査へ置き換えた。
- CRLF は `source_text_next_index_after_cr` に切り出し、`'\r\n'` を 1 改行、`'\r'` 単独も 1 改行として扱う既存契約を維持した。
- `Vec<i32>` への line start 追加が失敗した場合は `StdErrorKind::OutOfMemory` を返し、消費済み owner を空 `Vec` に差し替えて loop 後の owner 状態を明確にした。
- `tests/stdlib/neplg2_text.n.md` に `ret: 0` を追加し、既存 fixture の runtime failure が見逃されないようにした。`"alpha\nbeta\n"` の byte 長は 11 で、EOF offset も 11 が正しいため期待値を修正した。
- 4096 行の generated source fixture を追加し、line count、EOF location、末尾 content line span を検証するようにした。
- `nodesrc/test_selfhost_source_text_no_recursive_line_map.js` を追加し、line map builder が明示 loop を使い、関数本体で自己再帰しないことを静的に確認する。

## 検証結果

- `node nodesrc/test_selfhost_source_text_no_recursive_line_map.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_text.n.md --no-tree -o tmp/source-text-iter-line-map.json -j 1`
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/text.nepl --no-tree -o tmp/source-text-iter-line-map-docs.json -j 1`
