---
id: ISS-20260521T081047776Z-COLLECTION-SLOT-RETURN-TRANSFER-PROD-1D005BEE
title: "Collection slot return-transfer producer analysis is not exhaustive"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T081047776Z-COLLECTION-SLOT-RETURN-TRANSFER-PROD-1D005BEE: Collection slot return-transfer producer analysis is not exhaustive

## 概要

Return-transfer producer analysis ends ResourceOp matching with a wildcard arm, so adding a new value-producing ResourceOp can silently skip collection slot state transfer instead of failing at compile time.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- `collection_slot_summary_return_collect.rs` の return-transfer producer 逆追跡は `match &ops[index]` の最後を wildcard arm にしていた。
- そのため `ResourceOp` に新しい value producer が追加されても、collection slot return transfer だけが compile-time exhaustiveness check を受けず、無音で過去の op へ探索を続ける可能性があった。
- Stage 6 の方針では、静的検査実装の誤りも enum / match の網羅性で発見しやすい形にする必要がある。

## 問題

Return-transfer producer analysis ends ResourceOp matching with a wildcard arm, so adding a new value-producing ResourceOp can silently skip collection slot state transfer instead of failing at compile time.

## 影響

Future Resource IR lowering additions can weaken memory safety for non-Copy collection payloads because caller live slot state may stop propagating across newly introduced value producers without any compiler error.

## 修正方針

Replace wildcard ResourceOp handling with explicit arms for every variant, separating value-producing and non-producing operations so Rust exhaustiveness checking guards the static-check implementation itself.

## 修正内容

- `ResourceOp` producer 解析の wildcard arm を削除し、全 variant を明示的に列挙した。
- `Borrow`、`FunctionValue`、`RawMemory`、`RawAddressAlias`、`RawAddressView`、`StorageOrigin`、initializer なし `DeclareLocal` のように対象 place を定義するが collection slot return-transfer の structural source ではない op は、その時点で逆追跡を止めるようにした。
- 非 producer / 対象外 producer は明示 arm で無視し、新しい `ResourceOp` variant が増えた場合は `cargo check` が producer 解析の更新漏れを検出できるようにした。
- stdlib 名や関数名の列挙ではなく、Resource IR enum の網羅性そのものを検査境界にした。

## 検証

cargo check must compile the exhaustive match, and collection slot call summary regressions must still pass.

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
