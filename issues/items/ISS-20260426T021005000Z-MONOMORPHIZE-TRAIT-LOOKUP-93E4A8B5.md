---
id: ISS-20260426T021005000Z-MONOMORPHIZE-TRAIT-LOOKUP-93E4A8B5
title: "monomorphize trait impl resolution falls back to linear scans"
area: core
status: verified
resolved: true
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

## 対応

- `monomorphize_internal` で impl table を作る時点で、exact impl 用の `(trait_name, method)` index と generic trait application 用の `(base_trait_name, method)` index を同時に構築した。
- `resolve_trait_impl_name` は exact key lookup の後、index 済み候補だけを調べるようにし、`impl_map` / `impl_entries` 全体走査を削除した。
- `trait_lookup_cache` を追加し、正規化済み `trait_args` と `self_ty` ごとの成功/失敗結果を monomorphize phase 内で再利用するようにした。
- `nepl-core/tests/neplg2.rs` に `generic_trait_impl_method_resolves_by_trait_args` を追加し、`Hasher<.K>` 型の generic trait impl が trait argument 経由で解決されることを固定した。

## 検証

- generic trait impl が複数ある fixture で exact / generic / no match の結果が変わらないことを確認する。
- trait call 数を増やした synthetic fixture で monomorphize lookup 回数または実行時間を測る。

## 確認済み

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args`: pass
- `cargo test -p nepl-core --test neplg2 trait`: 6/6 passed
- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md -i tests/stdlib/traits_order.n.md -i tests/compiler/neplg2.n.md --no-tree -o tmp/monomorphize-trait-index-nodesrc-after-trunk.json -j 1`: 51/51 passed
- `trunk build`: pass（既存 Rust warning は残存）
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-monomorphize-trait-index.json`: 13/13 passed

補足: `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions` は今回の変更を外した baseline でも `left: 0, right: 5` で失敗したため、`ISS-20260426T053112317Z-SELFHOST-REQ-HASHKEY-FIXTURE-FAILS-U-34A22E8C` として別 issue 化した。
