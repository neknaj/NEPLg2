---
id: ISS-20260427T225104799Z-MOVE-CHECK-KEEPS-RAW-HELPER-OFFSETS--89FF7C37
title: "move_check keeps raw helper offsets unknown after literal instantiation"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/neplg2.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T225104799Z-MOVE-CHECK-KEEPS-RAW-HELPER-OFFSETS--89FF7C37: move_check keeps raw helper offsets unknown after literal instantiation

## 概要

`slot_ptr(base, 0)` のような raw address helper が、call-site の literal i32 引数を反映する前に `base+?` として要約されていた。その結果、実際には disjoint な後続 store が live non-Copy raw place の上書きとして誤診断されていた。

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/neplg2.rs, tests/compiler/move_effect.n.md`

## 根拠

- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture` が D3100 `overwriting raw memory place containing non-Copy value: p+?` で失敗した。
- HIR では `slot_ptr<LocalToken,i32> p 0` が `slot_ptr__...` user call として残り、`size_of<.T>` も `size_of__...` user call として現れる。
- `move_check` は non-literal offset を `base+?` として保守的に扱う必要があるが、literal 引数と `size_of` 定数で確定できる helper call まで `base+?` に潰していた。

## 問題

raw address helper functions such as `slot_ptr(base, 0)` are summarized with `base+?` before call-site literal i32 arguments are applied. This can make a known disjoint store look like an overwrite of a live non-Copy raw place.

## 影響

False D3100 diagnostics block valid generic raw memory helpers, and the same imprecision pressures developers to weaken raw ownership checks instead of preserving precise provenance.

## 修正方針

Track i32 constants in move_check alias contexts and specialize raw alias summaries for user calls with actual argument aliases/constants, so helper return addresses can collapse back to concrete raw offsets when the call site provides literals or `size_of` constants.

## 対応結果

- `MoveCheckContext` に i32 定数 alias を追加し、`let` / `set` / branch merge / alias summary context で保持するようにした。
- raw address add / `mem_ptr_add` の offset 判定で literal だけでなく、`size_of<T>` と i32 `add` / `sub` / `mul` から得られる定数を評価するようにした。
- `size_of<T>` が monomorphized user call として現れる経路を、関数本体の `#intrinsic "size_of"` から定数化するようにした。
- user call の raw return alias が `base+?` になる場合だけ、call-site の実引数 alias / 定数を入れた context で関数 body を再要約し、`slot_ptr p 0` を `p` に戻せるようにした。
- raw memory effect 全般の不明 offset は引き続き保守的に扱い、今回の特殊化は戻り raw address alias に限定して過剰な再要約を避けた。

## 検証

`literal helper offset` の正常系と `non-literal helper offset` の compile_fail を追加し、既存の unknown-offset 保守検査を維持する。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 -- --nocapture`: 60/60 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-helper-constant-offset-summary.json -j 1`: 94/94 passed
- `cargo check -p nepl-core --tests`: pass
