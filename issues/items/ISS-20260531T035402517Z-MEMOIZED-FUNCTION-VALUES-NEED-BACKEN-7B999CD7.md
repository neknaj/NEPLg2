---
id: ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7
title: "memoized function values need backend representation and identity-observation ban"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/codegen; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/effect_check.rs"
---

# ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7: memoized function values need backend representation and identity-observation ban

## 概要

Existing backend function values are lowered as table indices or i32-like ids and do not carry private cache environment state, while memo_call returns a function value with hidden private cache storage.

## 対象

- `nepl-core/src/codegen; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/effect_check.rs`

## 根拠

- 未記入

## 問題

Existing backend function values are lowered as table indices or i32-like ids and do not carry private cache environment state, while memo_call returns a function value with hidden private cache storage.

## 影響

Without a backend representation and identity-observation ban, memoized function values can either be impossible to lower or can leak closure/cache allocation identity through equality, hash, raw store/load, cast, layout query, or debug output.

## 修正方針

Choose a Phase 1 representation for memoized functions, such as compiler-generated wrappers with hidden private cache regions or a closure object with sealed identity, and forbid pure public APIs that observe function address, closure allocation id, cache region id, equality, hash, or raw representation.

## 検証

Regression tests should accept calling a memoized pure named function and reject identity/hash/address/cast/raw-store observation, function-value key usage, public cache field exposure, and backend paths that require an unsealed closure id.

## 2026-06-01 checkpoint

HIR の `MemoizedFunctionValue` を Resource IR lowering で plain `FunctionValue` と同化しないようにした。`ResourceOp::FunctionValue` は `ResourceFunctionValueKind::{Plain, Memoized}` を持つ。

現時点の backend codegen は、sealed private cache backend が未実装であるため、`MemoizedFunctionValue` を既存の function table value と同じ可観測結果へ lower する。ただし Resource IR と body hash では memoized kind を保持するため、Resource proof cache と将来 backend 実装は plain `@func` と `memo_call @func` を区別できる。

検証:

- `cargo test -p nepl-core function_memo_call --test functions -- --nocapture`
- `cargo test -p nepl-core resource_function_body_hash_tracks_memoized_function_value_kind --lib -- --nocapture`

残件:

- memoized function value の sealed backend representation。
- function identity equality / hash / raw address / debug observation の禁止を backend と typecheck へ明示接続すること。
- `memo_call @pure_named_func` の呼び出し実行時に private cache を実際に利用すること。

## 2026-06-01 function alias kind checkpoint

`FunctionAliasTable` は `FunctionValueIdentity` だけでなく `ResourceFunctionValueKind` も
運ぶようになった。これにより、同じ underlying function identity を持つ plain function value
と memoized function value が、copy、aggregate field、branch merge、match merge、indirect call
候補伝播で同一候補として dedupe されない。

既存の indirect call summary consumer はまだ function value kind を解釈せず、underlying
function symbol で borrow / effect / owner / initialized / collection-slot summary を引く。
そのため、plain と memoized が同じ symbol を指す場合は、summary 適用前に symbol を重複排除する。
これは現行 backend が memoized value を plain function table value と同じ可観測結果へ lower する
段階の互換境界であり、memoized kind を捨てるものではない。

この checkpoint は sealed backend representation そのものではない。目的は、今後 private
cache region identity や sealed wrapper identity を function value alias に載せる前提として、
既存の Resource IR 解析が memoized kind を落とさない運搬面を固定することである。

検証:

- `cargo test -p nepl-core function_alias --lib -- --nocapture`

## 2026-06-01 sealed memo cache proof dependency

sealed backend representation は
`ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7` の proof を下流依存にする。

backend が private cache storage を実際に持つ前に、sealed region が public value、raw address、
function equality/hash/debug observation、cache stats/clear/ref API へ出ないことを Resource IR 側で
証明する。`MemoizedFunctionValue` を plain function table value と同じ可観測結果へ lower している
現 checkpoint は、sealed representation 完了ではなく fail-closed な足場として扱う。
