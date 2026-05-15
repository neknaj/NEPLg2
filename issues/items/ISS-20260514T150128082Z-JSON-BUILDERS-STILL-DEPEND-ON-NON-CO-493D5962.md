---
id: ISS-20260514T150128082Z-JSON-BUILDERS-STILL-DEPEND-ON-NON-CO-493D5962
title: "JSON builders still depend on non-Copy Vec payloads"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
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

## 初期対応方針

2026-05-14 時点では、この issue は open とした。JSON builder を単純に Copy-only Vec へ合わせると、JSON array/object の owner model を壊す可能性がある。`RV-STDLIB-004` の non-Copy payload collection 設計、または JSON value representation の owner discipline と合わせて修正する。

## 2026-05-15 Agent 1 修正

JSON array/object の表現を `Vec<JsonValue>` / `Vec<JsonMember>` から、Copy な typed fragment である `JsonArray` / `JsonObject` へ再設計した。

対応内容:

- `JsonValue::Array` は `JsonArray`、`JsonValue::Object` は `JsonObject` を payload に持つ。
- `JsonArray` は bracket 内の compact JSON body を `items <str>` として保持し、`JsonObject` は member body を `members <str>` として保持する。
- `json_array_new` / `json_object_new` は `Result<JsonArray, StdErrorKind>` / `Result<JsonObject, StdErrorKind>` を返し、空 fragment を作る。
- `json_array_push` / `json_object_push` は値を即時 `json_serialize` して fragment に追加する。現段階では typed tree traversal API ではなく、安全な JSON document builder として責務を定義する。
- `json_as_array` / `json_as_object` は `Option<JsonArray>` / `Option<JsonObject>` を返す。
- serializer から `core/mem` / raw `load<JsonValue>` / `load<JsonMember>` / `mem_ptr_addr` 依存を削除し、typed fragment を `[` `]` / `{` `}` で包むだけにした。
- `nodesrc/test_json_builder_fragment_contract.js` を追加し、JSON builder が `Vec<Json*>` と raw memory storage 読みに戻らないことを固定した。

この修正は `Vec` の Copy-only 境界を緩めず、non-Copy payload collection 未完成の状態で JSON tree を raw collection storage に載せる入口を閉じる。`RV-STDLIB-004` / raw-memory-backed API migration の親 issue は引き続き open だが、この JSON builder 固有の不整合は解消した。

検証:

- `node nodesrc/test_json_builder_fragment_contract.js`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/encoding/json/builders.nepl -n 1 --assert-io --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/encoding/json/serialize.nepl -n 1 --assert-io --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/encoding/json.nepl -n 1 --assert-io --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/encoding/json/access.nepl -n 1 --assert-io --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/encoding/json/types.nepl -n 1 --assert-io --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/stdlib/json_typed_values.n.md --no-tree -o tmp/agent1-json-fragment-typed-values.json -j 1 --dist web/dist --assert-io`: total=7, passed=7
- `node nodesrc/tests.js -i stdlib/tests/json.n.md --no-tree -o tmp/agent1-json-fragment-stdlib-json.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/alloc/encoding/json -i stdlib/alloc/encoding/json.nepl -i stdlib/tests/json.n.md -i tests/stdlib/json_typed_values.n.md --no-tree -o tmp/agent1-json-fragment-all.json -j 1 --dist web/dist --assert-io`: total=14, passed=14
