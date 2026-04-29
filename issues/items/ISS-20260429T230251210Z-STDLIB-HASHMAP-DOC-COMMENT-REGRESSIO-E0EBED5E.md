---
id: ISS-20260429T230251210Z-STDLIB-HASHMAP-DOC-COMMENT-REGRESSIO-E0EBED5E
title: "stdlib HashMap doc comment regression test asserts stale phrase"
area: stdlib
status: verified
resolved: true
priority: P2
type: test
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/tests/doc_comments.rs, stdlib/alloc/collections/hashmap.nepl"
---

# ISS-20260429T230251210Z-STDLIB-HASHMAP-DOC-COMMENT-REGRESSIO-E0EBED5E: stdlib HashMap doc comment regression test asserts stale phrase

## 概要

stdlib HashMap の構造体 doc は現在 `hash map [本体/ほんたい]` と説明しているが、regression test は古い `hash table の[本体/ほんたい]` という文言を直接要求している。そのため doc parser の添付自体は正しくても、妥当な stdlib コメント変更で `cargo test -p nepl-core --test doc_comments` が失敗する。

## 対象

- `nepl-core/tests/doc_comments.rs, stdlib/alloc/collections/hashmap.nepl`

## 根拠

- `cargo test -p nepl-core --test doc_comments stdlib_hashmap_struct_has_doc_comment -- --nocapture`: `assertion failed: doc.contains("hash table の[本体/ほんたい]")`
- 対象 `HashMap` 構造体 doc は `## HashMap` と現在の責務説明を保持しており、loader の doc attachment 自体は壊れていない。

## 問題

stdlib HashMap の構造体 doc は現在 `hash map [本体/ほんたい]` と説明しているが、regression test は古い `hash table の[本体/ほんたい]` という文言を直接要求している。そのため doc parser の添付自体は正しくても、妥当な stdlib コメント変更で `cargo test -p nepl-core --test doc_comments` が失敗する。

## 影響

core 全体テストが stdlib コメントの文言差分で失敗し、doc comment attachment の回帰と stdlib 文書改善を区別できない。CI が赤くなるため静的検証修正の検証も妨げる。

## 修正方針

doc comment test を現行 HashMap 構造体 doc の安定した不変条件に合わせる。具体的には `## HashMap` heading と、要素数・容量・tombstone・storage・hasher を保持するという構造体の責務を確認し、古い `hash table` 表現には依存しないようにする。

## 対応

- `stdlib_hashmap_struct_has_doc_comment` を `stdlib_hashmap_struct_doc_is_attached_to_hashmap_struct` に改名し、test の意図を doc attachment の確認として明確にした。
- assertion は `## HashMap` heading と HashMap 構造体の責務語彙（要素数、tombstone、storage、hasher）を確認する形に変更した。
- module doc の誤添付を検出できるように、`# hashmap` が構造体 doc に混入していないことも確認する。

## 検証

- `cargo test -p nepl-core --test doc_comments stdlib_hashmap_struct_has_doc_comment -- --nocapture`: reproduced before fix
- `cargo test -p nepl-core --test doc_comments -- --nocapture`: `3 passed`
- `rustfmt --check nepl-core/tests/doc_comments.rs`: passed
