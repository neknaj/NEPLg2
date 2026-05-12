---
id: ISS-20260512T125509232Z-RESOURCE-VARIANT-COMPARISON-ALLOCATE-30C96E24
title: "Resource variant comparison allocates normalized strings on hot paths"
area: core
status: fixed
resolved: true
priority: P2
type: performance
created: 2026-05-12
updated: 2026-05-12
target: nepl-core/src/resource/variant_name.rs
---

# ISS-20260512T125509232Z-RESOURCE-VARIANT-COMPARISON-ALLOCATE-30C96E24: Resource variant comparison allocates normalized strings on hot paths

## 概要

variant_name::variant_names_match normalizes both operands into owned String values for every comparison even though comparison only needs the borrowed tail after ::.

## 対象

- `nepl-core/src/resource/variant_name.rs`

## 根拠

- `variant_name::normalize_variant_name` は enum payload place key を作るために owned `String` を返す必要がある。
- 一方で `variant_name::variant_names_match` は equality 判定だけでよく、owned `String` を 2 つ作る必要はない。
- owner summary variant dedupe、payload type lookup、owner variant path scans は static check の hot path になり得るため、比較だけの処理で allocation を増やす設計は避けるべきである。

## 問題

variant_name::variant_names_match normalizes both operands into owned String values for every comparison even though comparison only needs the borrowed tail after ::.

## 影響

Owner summary and Resource IR variant path scans can allocate repeatedly while deduplicating or matching enum payload variants, increasing compile-time memory churn in static checks.

## 修正方針

Introduce a borrowed variant_name_tail helper, keep normalize_variant_name for owned Place keys, and implement variant_names_match by comparing borrowed tails without allocation.

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core resource::variant_name::tests -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: passed
- `node nodesrc/issues.js check --dir issues`: passed

## 対応記録

- `variant_name_tail` を追加し、`::` 区切りの末尾 variant 名を borrowed `&str` として返すようにした。
- `normalize_variant_name` は owned `PlaceProjection::EnumPayload` key が必要な場合だけ `String` 化する。
- `variant_names_match` は borrowed tail 同士を比較し、comparison hot path での一時 `String` allocation を避ける。
- `variant_name.rs` に canonical tail と qualified variant 比較の unit test を追加した。
