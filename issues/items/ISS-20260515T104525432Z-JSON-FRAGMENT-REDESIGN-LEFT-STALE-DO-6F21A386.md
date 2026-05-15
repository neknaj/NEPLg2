---
id: ISS-20260515T104525432Z-JSON-FRAGMENT-REDESIGN-LEFT-STALE-DO-6F21A386
title: "JSON fragment redesign left stale documentation contracts"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/alloc/encoding/json/{types,builders,serialize}.nepl,nodesrc/test_stdlib_json_doc_no_boilerplate.js,nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260515T104525432Z-JSON-FRAGMENT-REDESIGN-LEFT-STALE-DO-6F21A386: JSON fragment redesign left stale documentation contracts

## 概要

After JSON array/object builders moved from Vec<JsonValue>/Vec<JsonMember> payloads to Copy JsonArray/JsonObject fragments, the JSON documentation policy still expects old owner-transfer wording and the global stdlib documentation contract reports declaration doctest gaps increased to 1036 > 1032.

## 対象

- `stdlib/alloc/encoding/json/{types,builders,serialize}.nepl,nodesrc/test_stdlib_json_doc_no_boilerplate.js,nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- `node nodesrc/test_stdlib_json_doc_no_boilerplate.js` が、旧 `Vec<JsonValue>` / `Vec<JsonMember>` payload traversal 前提の `array owner transfer contract` と `serialize.nepl must own array payload traversal` で失敗した。
- `node nodesrc/test_stdlib_documentation_contract.js` が `stdlib declaration doctest gaps increased: 1036 > 1032` を報告した。
- 現在の JSON builder は `JsonArray` / `JsonObject` の Copy fragment を保持し、serializer は raw collection storage を走査せず fragment を括弧で包む設計である。

## 問題

After JSON array/object builders moved from Vec<JsonValue>/Vec<JsonMember> payloads to Copy JsonArray/JsonObject fragments, the JSON documentation policy still expects old owner-transfer wording and the global stdlib documentation contract reports declaration doctest gaps increased to 1036 > 1032.

## 影響

The source policy signal stays noisy and the public JSON fragment types lack executable examples for the current Copy fragment contract. Leaving the old wording would document a non-existent owner transfer model and obscure the Stage 6 collection safety boundary.

## 修正方針

Update JSON doc-policy required phrases to the JsonArray/JsonObject fragment contract, add meaningful doctests to the public JSON fragment/type declarations that introduced the gap, and keep the JSON builder implementation unchanged.

## 検証

node nodesrc/test_stdlib_json_doc_no_boilerplate.js; node nodesrc/test_stdlib_documentation_contract.js; focused JSON doctests; node nodesrc/issues.js check

## 2026-05-15 修正

JSON fragment 再設計後の documentation contract を現行設計へ合わせた。

- `JsonArray` / `JsonObject` / `JsonValue` / `JsonMember` の declaration doc に、典型的な構築・field access・serialize を示す doctest を追加した。
- `json_object` の注意書きに、`JsonObject` が Copy な value fragment であり、builder API では返った値を受け取り直す方針であることを追記した。
- `nodesrc/test_stdlib_json_doc_no_boilerplate.js` の required phrase を、旧 owner-transfer wording から Copy fragment contract に更新した。
- `serialize.nepl` は raw `mem_ptr_addr` / `load<Json*>` traversal を所有するのではなく、typed fragment を `[` `]` / `{` `}` で包む責務に変わったため、source policy もその形を検査するように変更した。
- `serialize.nepl` の module doc に残っていた「Vec に push」表現を builder push に修正した。

検証:

- `node nodesrc/test_stdlib_json_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_documentation_contract.js`: pass (`declarationNoDoctest=1032`)
- `node nodesrc/tests.js -i stdlib/alloc/encoding/json --no-tree -o tmp/agent1-json-doc-contract-all.json -j 1 --dist web/dist --assert-io`: total=9, passed=9
