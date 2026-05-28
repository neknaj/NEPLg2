---
id: ISS-20260528T085628387Z-NEPLG2-1-NEEDS-POSTFIX-FREE-TYPE-EVI-43646AFF
title: "NEPLg2.1 needs postfix-free type evidence for zero-arg user generic calls"
area: neplg21
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-28
updated: 2026-05-28
target: "nepl-core/src/parser/**; nepl-core/src/typecheck/**; tests/compiler/sizeof.n.md"
---

# ISS-20260528T085628387Z-NEPLG2-1-NEEDS-POSTFIX-FREE-TYPE-EVI-43646AFF: NEPLg2.1 needs postfix-free type evidence for zero-arg user generic calls

## 概要

NEPLg2.1 removes postfix generic calls, but user-defined generic functions with no value parameters and an erased scalar result, such as a size_of_t<.T> wrapper returning i32, have no argument or result evidence from which the typechecker can recover .T after removing size_of_t<T>.

## 対象

- `nepl-core/src/parser/**; nepl-core/src/typecheck/**; tests/compiler/sizeof.n.md`

## 根拠

- `size_of %T` / `align_of %T` は compiler-owned layout query なので、callee 名から専用構文として認識できる。
- user-defined `size_of_t <.T> %fn unit i32` のような wrapper は普通の関数であり、`size_of_t %T` を一般化すると `callee %T` が type arg なのか value ascription なのか曖昧になる。
- `size_of_t<i32>` を単に削除すると、値引数にも戻り値にも `.T` を決める証拠が残らない。これは receiver / argument / expected result から解ける通常の NEPLg2.1 postfix-free migration と別の root problem である。
- `tests/compiler/sizeof.n.md` の migration では `size_of %T` 本体は新構文へ移行できたが、`size_of_t<i32>` call は本 issue の未解決例として残った。

## 問題

NEPLg2.1 removes postfix generic calls, but user-defined generic functions with no value parameters and an erased scalar result, such as a size_of_t<.T> wrapper returning i32, have no argument or result evidence from which the typechecker can recover .T after removing size_of_t<T>.

## 影響

Mechanical corpus migration must either keep old postfix type arguments for this narrow shape or invent ad hoc wrapper APIs. Leaving it implicit would hide a real source migration boundary separate from compiler-owned layout queries.

## 修正方針

Define an official postfix-free type evidence form or expected-type propagation rule for zero-argument user generic calls whose type parameters appear only in the callee body, then migrate size_of_t-style fixtures away from size_of_t<T>.

## 検証

Add focused compile/pass tests covering a zero-argument generic user function whose type parameter is used only in the body, and confirm that the old explicit postfix call can be removed without changing downstream Resource IR.
