---
id: ISS-20260527T034609921Z-NEPLG2-1-NEEDS-POSTFIX-FREE-SYNTAX-F-6F6C5FD9
title: "NEPLg2.1 needs postfix-free syntax for type-only layout generic calls"
area: neplg21
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-27
updated: 2026-05-27
target: "nepl-core/src/parser/**; nepl-core/src/typecheck/**; stdlib/core/mem/{layout,types}.nepl"
---

# ISS-20260527T034609921Z-NEPLG2-1-NEEDS-POSTFIX-FREE-SYNTAX-F-6F6C5FD9: NEPLg2.1 needs postfix-free syntax for type-only layout generic calls

## 概要

NEPLg2.1 removes postfix generic calls, but size_of<T> and align_of<T> carry their type argument only in type space and return plain i32. Current inference has no value/result evidence to recover T after a purely mechanical cleanup.

## 対象

- `nepl-core/src/parser/**; nepl-core/src/typecheck/**; stdlib/core/mem/{layout,types}.nepl`

## 根拠

- `size_of<T>` / `align_of<T>` は値引数を持たず、戻り値も `i32` だけであるため、`alloc_region<T>` のように戻り値の `Result RegionToken T str` から `T` を復元する経路がない。
- core/mem の positive doctest cleanup を進める過程で、`alloc_region<T>` / `dealloc_region<T>` / `region_ptr_at<T,U>` などは typed local の `%Result ...` 注釈へ移せる一方、type-only layout query だけは同じ方法では移せないことを確認した。
- NEPLg2.1 の後置ジェネリクス撤廃を完了させるには、型だけを問い合わせる組込み/stdlib helper に対する公式の前置記法、または compiler 側の明示的な推論・例外設計が必要である。

## 問題

NEPLg2.1 removes postfix generic calls, but size_of<T> and align_of<T> carry their type argument only in type space and return plain i32. Current inference has no value/result evidence to recover T after a purely mechanical cleanup.

## 影響

Positive core/mem doctests for size_of<i32>, align_of<i32>, and size_of<MemPtr<i32>> cannot be migrated to postfix-free source without either a dedicated surface form or compiler inference support. Leaving the gap undocumented would hide a real NEPLg2.1 migration boundary.

## 修正方針

Define an official postfix-free source form or compiler-owned exception for type-only layout queries, then migrate stdlib/core/mem/layout.nepl and stdlib/core/mem/types.nepl positive doctests away from postfix generic calls.

## 検証

Add parser/typechecker coverage for the selected postfix-free type-only generic form and extend the NEPLg2.1 core/mem doccomment source-policy so size_of and align_of no longer need an exclusion.
