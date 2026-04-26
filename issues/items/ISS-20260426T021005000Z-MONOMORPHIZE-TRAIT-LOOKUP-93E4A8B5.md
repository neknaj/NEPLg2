---
id: ISS-20260426T021005000Z-MONOMORPHIZE-TRAIT-LOOKUP-93E4A8B5
title: "monomorphize trait impl resolution falls back to linear scans"
area: core
status: open
resolved: false
priority: P2
type: performance
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/monomorphize.rs
source: doc/neplg2/pre_selfhost_performance_audit_20260426.md
---

# ISS-20260426T021005000Z-MONOMORPHIZE-TRAIT-LOOKUP-93E4A8B5: monomorphize trait impl resolution falls back to linear scans

## 概要

`monomorphize.rs` の trait impl resolution は exact key lookup の後、`impl_map` と `impl_entries` を線形走査する。
trait call 数、impl 数、generic instantiation 数が増えると、monomorphize の lookup cost が増えやすい。

## 根拠

- `nepl-core/src/monomorphize.rs:457` に `resolve_trait_impl_name` がある。
- `monomorphize.rs:472` は `impl_map.iter()` を走査し、trait 名 / method / self type を比較する。
- `monomorphize.rs:480` は `impl_entries.iter()` を走査し、trait args の pattern match を行う。
- self-host compiler は `HashKey` / `Clone` / `Copy` / `Ord` / stream traits など、多数の trait call を stdlib 経由で使う。

## 問題

現在の exact key で見つからない generic impl は、候補全体を毎回走査する。
同じ trait / method / self type の組み合わせが繰り返し出る場合でも cache がなく、型正規化と pattern match を繰り返す。

## 影響

stdlib の trait 化を進めるほど monomorphize 時間が増える可能性がある。
self-host compiler の core pass が generic collection と trait helper を多用すると、Rust 参照 compiler と self-host compiler の両方で trait dispatch 解決が hot path になり得る。

## 修正方針

`(trait_name, method)`、可能なら `(trait_name, method, resolved_self_base)` で候補を index 化する。
generic impl の pattern match 結果は、正規化済み `trait_args` と `self_ty` の cache key で memoize する。
cache invalidation が不要な monomorphize phase 内の immutable table として構築する。

## 検証

- generic trait impl が複数ある fixture で exact / generic / no match の結果が変わらないことを確認する。
- trait call 数を増やした synthetic fixture で monomorphize lookup 回数または実行時間を測る。
