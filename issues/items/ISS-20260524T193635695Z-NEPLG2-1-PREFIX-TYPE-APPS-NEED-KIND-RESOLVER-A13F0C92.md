---
id: ISS-20260524T193635695Z-NEPLG2-1-PREFIX-TYPE-APPS-NEED-KIND-RESOLVER-A13F0C92
title: "NEPLg2.1 prefix type apps need kind resolver"
area: core
status: open
resolved: false
priority: P0
type: architecture
created: 2026-05-24
updated: 2026-05-24
target: "nepl-core/src/parser.rs, nepl-core/src/typecheck/**"
---

# ISS-20260524T193635695Z-NEPLG2-1-PREFIX-TYPE-APPS-NEED-KIND-RESOLVER-A13F0C92: NEPLg2.1 prefix type apps need kind resolver

## 概要

NEPLg2.1 の `Vec i32` / `Result i32 str` 型式は、現在 parser 内の hard-coded arity hints で境界を決めている。これは既知 stdlib 型には機能するが、import された user generic 型や後続宣言型では不十分である。

## 対象

- `nepl-core/src/parser.rs`
- `nepl-core/src/typecheck/**`

## 根拠

- NEPLg2.0 の `Vec<i32>` は `<>` により parser が型適用境界を決められた。
- NEPLg2.1 の `Vec i32` は bracketless prefix 型式なので、型名の kind / arity を知らなければ境界を一般には決められない。
- 現 checkpoint の parser は stdlib 代表型の arity hint で移行を前進させているが、これは frontend lowering boundary の暫定足場であり、長期仕様ではない。
- subagent review でも、hard-coded arity table は imported/user generic type で破綻し得ると指摘された。

## 問題

Hard-coded type arity hints leave NEPLg2.1 prefix type application incomplete and can misparse `%Foo i32 expr` when `Foo` is not pre-registered.

## 影響

NEPLg2.1 の実行対象 corpus は既知 stdlib 型中心なので一部を先に通せるが、selfhost 実装や user code では prefix 型式が安定しない。Resource IR へ影響させず frontend 内で解決する必要がある。

## 修正方針

- parser は必要なら unresolved prefix type item list を保持し、type/kind resolver で arity 境界を決める。
- struct / enum / trait / import metadata から type constructor arity を収集する。
- `%T expr` の expected-type lowering は、kind 解決後に既存 `TypeExpr::Apply` へ正規化する。
- Resource IR / ownership / codegen へ NEPLg2.1 専用の型構文 node を流さない。

## 検証

- imported generic type `Foo .T` を `%Foo i32 value` として使う parser/typecheck fixture。
- 同一 module 後続宣言型を prefix 型式から参照する fixture。
- stdlib 既存型で hard-coded arity table を消しても `cargo test -p nepl-core --test typeannot neplg21` が通ること。
