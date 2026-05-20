---
id: ISS-20260520T180237730Z-INITIALIZED-RAW-RANGE-COUNT-ALIASES--A1FBF011
title: "Initialized raw range count aliases are not preserved through loaded count values"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/cell_state_raw_range_value_alias.rs, nepl-core/src/resource/cell_state_raw_range_cover_tests.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260520T180237730Z-INITIALIZED-RAW-RANGE-COUNT-ALIASES--A1FBF011: Initialized raw range count aliases are not preserved through loaded count values

## 概要

Raw byte range の count が raw memory cell から temporary/local に読み出されたあと、同じ i32 count として guarded symbolic offset の証明に使えなければならない。しかし count alias 伝播が count place の型を厳密に保たず、誤った型の count evidence を作れる経路があった。

## 対象

- `nepl-core/src/resource/cell_state_raw_range_value_alias.rs, nepl-core/src/resource/cell_state_raw_range_cover_tests.rs`

## 根拠

- `cargo test -p nepl-core cell_state -- --test-threads=1` で `resource::cell_state_raw_range_cover_tests::byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local` が失敗した。
- 調査の結果、`replace_raw_range_count_value_prefix` が source/target の型不一致を拒否せず、suffix を付け替えるときに古い count place の型を残したまま target root へ置換できることが分かった。
- `range_count_alias_replacement` も canonical scalar equality を型一致より前に確認しており、型の合わない alias copy を先に受理できる順序になっていた。
- raw range count は indexed access の i32 証明に使う scalar であり、payload の `u8` と混同してはいけない。

## 問題

range count evidence の value-copy 伝播が型境界を明示的に検査していなかったため、次の 2 つの不正確さが混在していた。

- 正しい i32 count を raw memory cell から local に読み出した場合でも、guarded symbolic offset の証明と結合できない。
- 型の違う payload value を count alias として扱う余地があり、Resource IR の証明が型安全な値関係だけに基づくという前提を壊す。

## 影響

The checker can reject valid raw-memory access patterns that load a length or used count from storage before guarding an indexed access. また、型不一致の count alias を拒否する根拠が弱く、将来の Resource IR 拡張時に誤った初期化済み範囲証明を合成する危険があった。

## 修正方針

- `replace_raw_range_count_value_prefix` は source/target の型が一致しない場合に置換しない。
- `range_count_alias_replacement` は canonical equality や alias overlap より前に range/source/target の型一致を検査する。
- 回帰テストは count を i32 として表現し、payload 型と count 型を分離した上で、正しい loaded count が guarded symbolic offset を覆えることを確認する。
- 型不一致の value copy が count alias を合成しない negative regression を追加する。

## 検証

- `cargo test -p nepl-core cell_state_raw_range_value_alias_tests -- --test-threads=1`
- `cargo test -p nepl-core cell_state_raw_range_cover_tests -- --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_bulk -- --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized -- --test-threads=1`
- `node nodesrc/test_resource_raw_cell_lifecycle_policy.js`
- `node nodesrc/test_resource_checker_responsibility.js`
