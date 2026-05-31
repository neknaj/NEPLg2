---
id: ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE
title: "Higher-order function purity requires function value boundary design"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/typecheck; nepl-core/src/resource/lower_call.rs; doc/neplg2/private_effect_memoization_purity_design.md"
---

# ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE: Higher-order function purity requires function value boundary design

## 概要

memo_call returns a function value, so the purity design depends on higher-order function rules that distinguish function application from partial application and prevent function identity, captures, or raw function addresses from becoming observable pure results.

## 対象

- `nepl-core/src/typecheck; nepl-core/src/resource/lower_call.rs; doc/neplg2/private_effect_memoization_purity_design.md`

## 根拠

- `memo_call` の戻り値は関数値であり、通常の `func a` の引数不足を部分適用として扱うこととは別に設計する必要がある。
- NEPLg2.1 は curried-looking な関数型表記を採るが、部分適用は導入しない方針である。
- function identity、closure allocation id、raw function address、function equality/hash が pure API から観測できると、`memo_call(f)` が作る private cache identity も観測可能になる。
- 現行実装には function alias tracking があるが、capture 付き closure と private cache region の lifetime / owner transfer / effect propagation はまだ専用設計がない。

## 問題

memo_call returns a function value, so the purity design depends on higher-order function rules that distinguish function application from partial application and prevent function identity, captures, or raw function addresses from becoming observable pure results.

## 影響

If higher-order function values are treated as ordinary values without an identity and capture boundary, memo_call can leak its private cache through closure state or make allocation/identity observable while still being typed Pure.

## 修正方針

Specify saturated application, closure capture restrictions, known function effect propagation, and identity-observation bans for pure higher-order values; then connect those rules to memo_call and Resource IR function alias tracking.

## 2026-05-31 design checkpoint

- [NEPLg2 private effect / memoization purity design](../../doc/neplg2/private_effect_memoization_purity_design.md) に、高階関数境界を追加した。
- Phase 1 の `memo_call` は non-capturing named pure function value だけを受ける。capture 付き closure は memoization MVP に含めない。
- `memo_call func` は `memo_call` の saturated call が関数値を返す形であり、通常関数の部分適用ではないと定義した。
- function address、function identity equality、closure allocation id、private cache region id の pure public API は拒否対象にした。

## 2026-06-01 memo_call phase1 regression checkpoint

Phase 1 の `memo_call` は、`memo_call @pure_named_func` の直接形だけを
compiler-known primitive として受け入れる方針を focused regression で固定した。

追加した拒否ケースは次である。

- `let aliased @inc; memo_call aliased`: 明示 `@` を一度 local binding に入れた後の関数値は拒否する。
- `let selected id_func @inc; memo_call selected`: 高階関数へ渡して戻った関数値は拒否する。
- `let selected choose true; memo_call selected`: 高階関数から返った関数値は拒否する。
- `let local \x: ...; memo_call local`: 関数リテラル由来の関数値は拒否する。

いずれも期待診断は `MemoCallRequiresFunctionValue` である。これは、関数値そのものの型が
pure function であっても、private cache region、closure allocation identity、capture の
non-escape proof がまだ Resource IR に接続されていないためである。

subagent review では、capture 付き named function value は `memo_call` 専用診断へ寄せず、
既存の `FunctionValueCapturingUnsupported` 境界で拒否されるのが自然だと確認した。
したがって今回の test は、`memo_call` 入口に到達する「関数型の普通の値」を
Phase 1 で拒否する matrix に絞った。

## 検証

Typechecker regressions for pure higher-order arguments/results, rejected partial application, rejected observable function identity, and memo_call-specific accepted/rejected cases.
