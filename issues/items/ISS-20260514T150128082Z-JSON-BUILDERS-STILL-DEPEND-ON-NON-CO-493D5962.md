---
id: ISS-20260514T150128082Z-JSON-BUILDERS-STILL-DEPEND-ON-NON-CO-493D5962
title: "JSON builders still depend on non-Copy Vec payloads"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/encoding/json/builders.nepl, stdlib/alloc/encoding/json/types.nepl, stdlib/alloc/collections/vec/**"
---

# ISS-20260514T150128082Z-JSON-BUILDERS-STILL-DEPEND-ON-NON-CO-493D5962: JSON builders still depend on non-Copy Vec payloads

## 概要

json_array_new / json_object_new は Vec<JsonValue> / Vec<JsonMember> を作るが、現行 Vec constructor は Copy-only であり、JsonValue / JsonMember は owner-bearing non-Copy payload である。stdlib/alloc/encoding/json/builders.nepl の doctest は 	ype.trait_bound.unsatisfied で失敗する。

## 対象

- `stdlib/alloc/encoding/json/builders.nepl, stdlib/alloc/encoding/json/types.nepl, stdlib/alloc/collections/vec/**`

## 根拠

- `stdlib/alloc/encoding/json/builders.nepl` の doctest を実行すると、`Vec<JsonValue>` construction が `type.trait_bound.unsatisfied` で失敗する。
- 現行 `Vec.new<T>` / `with_capacity<T>` / `push<T>` は、`OwnedBuffer<T>` と initialized cell / drop traversal が完成するまで `.T: Copy` に閉じている。
- `JsonValue` は Array/Object payload として `Vec<JsonValue>` / `Vec<JsonMember>` owner を保持するため Copy ではない。`JsonMember` も `JsonValue` を含み、JSON builder が non-Copy collection payload に依存している。
- これは JSON builder 側だけの import drift ではなく、collection memory-safety contract と JSON value representation の設計不整合である。

## 問題

json_array_new / json_object_new は Vec<JsonValue> / Vec<JsonMember> を作るが、現行 Vec constructor は Copy-only であり、JsonValue / JsonMember は owner-bearing non-Copy payload である。stdlib/alloc/encoding/json/builders.nepl の doctest は 	ype.trait_bound.unsatisfied で失敗する。

## 影響

JSON builder API が現在の collection memory-safety contract と不整合で、JSON array/object を安全に構築できない。non-Copy payload collection の OwnedBuffer/drop traversal 完成前に、JSON 側の表現または builder contract を再設計する必要がある。

## 修正方針

JSON 担当作業で、Vec<JsonValue> 前提を見直す。短期的に Copy-only Vec に合わせるのではなく、JsonValue の ownership と collection storage/drop traversal の設計を RV-STDLIB-004 / Stage 6 と同期して決める。

## 検証

node nodesrc/tests.js -i stdlib/alloc/encoding/json/builders.nepl --no-tree -o tmp/agent1-json-builder-push-contract-probe.json -j 1 --dist web/dist --assert-io

## 調査結果

- `node nodesrc/tests.js -i stdlib/alloc/encoding/json/builders.nepl --no-tree -o tmp/agent1-json-builder-push-contract-probe.json -j 1 --dist web/dist --assert-io`: 1/1 fail
- failure phase: compile
- diagnostic: `type.trait_bound.unsatisfied`

## 対応方針

この issue は現時点では open とする。JSON builder を単純に Copy-only Vec へ合わせると、JSON array/object の owner model を壊す可能性がある。`RV-STDLIB-004` の non-Copy payload collection 設計、または JSON value representation の owner discipline と合わせて修正する。
