---
id: ISS-20260426T020001000Z-SELFHOST-REQ-HASHKEY-4B6D8F10
title: "self-host requirement test for user-defined HashMap keys still fails"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/tests/selfhost_req.rs, nepl-core/src/typecheck.rs, stdlib/core/traits/hash_key.nepl, stdlib/alloc/collections/hashmap.nepl"
source: doc/neplg2/pre_selfhost_audit_20260426.md
---

# ISS-20260426T020001000Z-SELFHOST-REQ-HASHKEY-4B6D8F10: self-host requirement test for user-defined HashMap keys still fails

## 概要

`nepl-core/tests/selfhost_req.rs` の `test_req_trait_extensions` は ignored で、実行すると失敗する。
直接の診断は `TypeInherentImplUnsupported` で、fixture 内の `impl Point:` が拒否され、その後 `hashmap_new` / `hashmap_insert` も未解決になる。

## 根拠

- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions -- --ignored` は失敗する。
- 失敗 diagnostics には `TypeInherentImplUnsupported`、`TypeUndefinedIdentifier`、`TypeAnnotationMismatch`、`TypeStackExtraValues` が含まれる。
- `stdlib/alloc/collections/hashmap.nepl` は `.K: HashKey` を要求し、`stdlib/core/traits/hash_key.nepl` は custom key は `HashKey` を impl すると記述している。

## 問題

self-host compiler の symbol table / type table / diagnostic table は、最初は `str` key だけで進められる可能性がある。
しかし user-defined key を generic collection に載せる要件が ignored のままだと、S3 以降で AST/HIR node id や interned symbol を構造体 key にしたくなった時に参照実装との差分が発覚する。

## 影響

セルフホスト実装側が collection 設計を過度に `str` / `i32` key へ寄せる圧力になる。
後から user-defined key を導入すると、collection API、trait impl、typecheck、fixture をまとめて変える必要がある。

## 修正方針

`test_req_trait_extensions` の期待要件を、inherent impl なのか `impl HashKey for Point` なのかに分離する。
self-host で必要な最小要件は `HashKey` trait impl を user-defined struct に実装し、`HashMap<Point, V, DefaultHash32>` で `new` / `insert` / `get` が通ることとして fixture を作り直す。
inherent impl を言語機能として残す場合は、別 issue / 別 fixture で診断と実装範囲を固定する。

## 検証

- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions -- --ignored`
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl --no-tree -o tmp/hashmap-user-key-tests.json -j 1`
