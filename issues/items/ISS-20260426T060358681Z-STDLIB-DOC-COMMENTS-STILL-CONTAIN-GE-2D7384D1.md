---
id: ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1
title: "stdlib doc comments still contain generated boilerplate"
area: stdlib
status: fixed
resolved: true
priority: P2
type: doc
created: 2026-04-26
updated: 2026-04-28
target: "stdlib/core/field.nepl, stdlib/nm/html_gen.nepl, stdlib/core/rand/xorshift32.nepl, stdlib/alloc/hash/fnv1a32.nepl, stdlib/alloc/collections/vec/sort.nepl"
---

# ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1: stdlib doc comments still contain generated boilerplate

## 概要

doc/stdlib_doc_comment_policy.md は手書きで具体的な説明を書く方針だが、stdlib の複数ファイルに 主な用途、定義済み処理をそのまま呼び出す薄いラッパ、引数の値は関数呼び出しで移動するため再利用時は束縛し直す といった汎用テンプレート文が大量に残っている。

## 対象

- `stdlib/core/field.nepl, stdlib/nm/html_gen.nepl, stdlib/core/rand/xorshift32.nepl, stdlib/alloc/hash/fnv1a32.nepl, stdlib/alloc/collections/vec/sort.nepl`

## 根拠

- 未記入

## 問題

doc/stdlib_doc_comment_policy.md は手書きで具体的な説明を書く方針だが、stdlib の複数ファイルに 主な用途、定義済み処理をそのまま呼び出す薄いラッパ、引数の値は関数呼び出しで移動するため再利用時は束縛し直す といった汎用テンプレート文が大量に残っている。

## 影響

コメントが API 固有のアルゴリズム、所有権、制約、計算量を説明せず、利用者や self-host 実装者が実装を誤読しやすい。コメントの品質問題が実装レビューのノイズにもなる。

## 修正方針

ファイル単位で public API と内部 helper を分け、テンプレート文を実際の責務、失敗条件、所有権、計算量に置き換える。大きいファイルは RV-STDLIB-009 の分割と合わせて進める。

## 検証

対象ファイルで boilerplate marker を検索し、代表 API に具体的な doc comment と doctest があることを確認する。

## 2026-04-28 再レビュー

`string.nepl`, `json.nepl`, `nm/parser.nepl`, `core/cast.nepl` は個別改善が入り、以前の対象からは外れつつある。一方で次の boilerplate marker がまだ残っている。

- `stdlib/core/field.nepl`: `get` / `put` が「主な用途」「定義済み処理をそのまま呼び出す薄いラッパ」のまま。
- `stdlib/nm/html_gen.nepl`: `escape_html`, `render_inlines`, `render_nodes` が renderer contract や escaping scope を説明していない。
- `stdlib/core/rand/xorshift32.nepl`: generator の周期、seed 0 の扱い、統計的制約が説明されていない。
- `stdlib/alloc/hash/fnv1a32.nepl`: hash algorithm の初期値、update rule、collision/security caveat が説明されていない。
- `stdlib/alloc/collections/vec/sort.nepl`: sort family が algorithm 別の安定性、計算量、in-place 条件を説明しない boilerplate のまま。

self-host 用 stdlib review では、hash/table、parser/htmlgen、sort/collection は仕様説明の不足がそのまま実装判断の不足になる。単なる文言置換ではなく、各 API の契約・失敗条件・計算量・doctest を具体化する。

## 2026-04-28 対応結果

- `stdlib/core/field.nepl` の `get` / `put` を、所有値の取り出し、borrowed field access、field overwrite の契約として説明し直した。
- `stdlib/nm/html_gen.nepl` の HTML escape、inline serializer、source serializer に、escape scope、sanitizer ではない制約、direct serializer 方針、code fence の扱いを追記した。
- `stdlib/core/rand/xorshift32.nepl` の module / struct / constructor / next を、seed 0 の固定点、非暗号用途、Xorshift32 の更新式、state の受け渡し契約として説明し直した。
- `stdlib/alloc/hash/fnv1a32.nepl` の module / struct / constructor / update / finalize を、offset basis、update rule、32-bit bit pattern、collision/security caveat として説明し直した。
- `stdlib/alloc/collections/vec/sort.nepl` の sort family コメントを、algorithm ごとの安定性、計算量、in-place 条件、raw helper の境界保証へ置き換えた。
- `nodesrc/test_stdlib_doc_comments_no_boilerplate.js` を追加し、今回対象ファイルの boilerplate marker 再発と主要契約文言の欠落を検出するようにした。

## 実行した検証

- `node nodesrc/test_stdlib_doc_comments_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_cast_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_json_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_string_doc_no_boilerplate.js`: pass
- `node nodesrc/tests.js -i stdlib/core/field.nepl -i stdlib/nm/html_gen.nepl -i stdlib/core/rand/xorshift32.nepl -i stdlib/alloc/hash/fnv1a32.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/stdlib-doc-comments-focused-after-policy.json -j 1`: 7/7 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/core/field.nepl -i stdlib/nm/html_gen.nepl -i stdlib/core/rand/xorshift32.nepl -i stdlib/alloc/hash/fnv1a32.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/stdlib-doc-comments-after-resource-effect-boundaries.json -j 1`: 7/7 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
