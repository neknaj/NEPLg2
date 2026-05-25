---
id: ISS-20260525T213016826Z-NEPLG2-1-TYPE-ARITY-PRELOAD-DROPS-CY-F74EBA18
title: "NEPLg2.1 type arity preload drops cyclic facade exports"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-25
updated: 2026-05-26
target: "nepl-core/src/loader.rs, nepl-core/src/parser.rs"
---

# ISS-20260525T213016826Z-NEPLG2-1-TYPE-ARITY-PRELOAD-DROPS-CY-F74EBA18: NEPLg2.1 type arity preload drops cyclic facade exports

## 概要

When a NEPLg2.1 prefix type expression depends on a type constructor re-exported by a facade that is already on the import stack, arity preloading skips the processing module and %Vec Diag can be parsed as bare Vec followed by a stray identifier.

## 対象

- `nepl-core/src/loader.rs, nepl-core/src/parser.rs`

## 根拠

- `alloc/diag/error/types.nepl` の `items %Vec Diag` は、`alloc/collections/vec` facade が処理中の import stack にある場合、旧実装では `Vec` の arity hint を取得できなかった。
- NEPLg2.1 prefix 型式は arity hint がないと型適用境界を決められないため、`%Vec Diag` を bare `Vec` と後続 identifier に分けてしまう。
- full load で処理中 module を再読込すると cycle になるため、cycle recovery は body parse ではなく declaration head と public facade re-export だけを見る必要がある。

## 問題

When a NEPLg2.1 prefix type expression depends on a type constructor re-exported by a facade that is already on the import stack, arity preloading skips the processing module and %Vec Diag can be parsed as bare Vec followed by a stray identifier.

## 影響

Stdlib modules that participate in cyclic facade imports can fail during parser lowering before typecheck, blocking corpus migration and main merge validation.

## 修正方針

Collect shallow declaration-head arity metadata from processing modules and their facade re-export dependencies without fully parsing the cycle.

## 対応結果

- parser に `type_arity_hints_from_source` を追加し、body parse を行わずに source text から type constructor arity metadata だけを収集できるようにした。
- loader の arity preload は、依存 module がすでに processing stack にある場合、full load を避けて shallow arity scan に切り替える。
- shallow scan は通常 import を辿らず、public import / merge facade / include だけを再帰対象にする。これにより cycle recovery が stdlib 実装 graph 全体へ広がらないようにした。
- 最小 cyclic facade fixture を `nepl-core/tests/typeannot.rs` に追加し、facade が処理中でも re-export 先の `Vec<.T>` arity で `%Vec i32` を parse できることを固定した。

## 検証

- `cargo test -p nepl-core --test typeannot test_neplg21_prefix_type_arity_preload_reads_cyclic_facade_exports -- --exact --nocapture`: pass。
- `cargo test -p nepl-core --test typeannot neplg21 -- --nocapture`: 5/5 pass。
- `cargo test -p nepl-core --test functions neplg21 -- --nocapture`: 8/8 pass。
- `cargo check -p nepl-core`: pass。
