---
id: ISS-20260426T074114888Z-STR-UNIFIES-WITH-I32-AND-ACCEPTS-RAW-A824A1D7
title: "str unifies with i32 and accepts raw handles as text"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/types.rs
---

# ISS-20260426T074114888Z-STR-UNIFIES-WITH-I32-AND-ACCEPTS-RAW-A824A1D7: str unifies with i32 and accepts raw handles as text

## 概要

TypeCtx::unify が TypeKind::Str と TypeKind::I32 を相互に Ok として扱うため、let s <str> 0 や json_string 0 のように raw i32 handle を str として受理できる。JSON typed value の compile_fail 追加中に、json_string 0 が成功して再現した。

## 対象

- `nepl-core/src/types.rs`

## 根拠

- `nepl-core/src/types.rs` の `TypeCtx::unify` は `(TypeKind::Str, TypeKind::I32) | (TypeKind::I32, TypeKind::Str)` を `Ok(self.i32_ty)` として扱っている。
- `tests/stdlib/json_typed_values.n.md` の作成中、`let _v <JsonValue> json_string 0;` が compile_fail にならず成功した。
- `json_array 0` は typed `Vec<JsonValue>` payload へ移行すると compile_fail になるため、JSON array/object では raw handle を型で排除できる一方、string だけは core の str/i32 unify に阻害される。

## 問題

TypeCtx::unify が TypeKind::Str と TypeKind::I32 を相互に Ok として扱うため、let s <str> 0 や json_string 0 のように raw i32 handle を str として受理できる。JSON typed value の compile_fail 追加中に、json_string 0 が成功して再現した。

## 影響

str が型として raw pointer/handle と分離されず、文字列 API、JSON string payload、diagnostic/source text、self-host parser の text 境界で無効な pointer や非文字列値を型で排除できない。stdlib 側で String payload を str にしても、raw handle 混入を完全には防げない。

## 修正方針

str と i32 の unify を原則禁止し、文字列 literal / string allocation / explicit cast や boundary intrinsic など、必要な変換経路だけを明示 API として残す。既存コードが raw i32 に依存している箇所は str API または MemPtr/RegionToken/ByteBuf へ分離する。

## 検証

compiler regression test に let s <str> 0 と json_string 0 の compile_fail を追加し、正規の string literal と string builder から得た str は通ることを確認する。stdlib JSON test でも raw string handle が拒否されることを固定する。
