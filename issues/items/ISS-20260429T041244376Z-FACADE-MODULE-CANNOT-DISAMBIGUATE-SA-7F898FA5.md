---
id: ISS-20260429T041244376Z-FACADE-MODULE-CANNOT-DISAMBIGUATE-SA-7F898FA5
title: "facade module cannot disambiguate same-name imported implementation with alias-qualified call"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resolve.rs, nepl-core/src/module_graph.rs, stdlib/alloc/string.nepl"
---

# ISS-20260429T041244376Z-FACADE-MODULE-CANNOT-DISAMBIGUATE-SA-7F898FA5: facade module cannot disambiguate same-name imported implementation with alias-qualified call

## 概要

alloc/string.nepl が alloc/string/scanner.nepl を as scanner で import し、同じ public API 名の wrapper から scanner::str_* を呼ぶと、import された同名関数が shadow warning として扱われた後に scanner::str_* が D3001 undefined になる。

## 対象

- `nepl-core/src/resolve.rs, nepl-core/src/module_graph.rs, stdlib/alloc/string.nepl`

## 根拠

- 未記入

## 問題

alloc/string.nepl が alloc/string/scanner.nepl を as scanner で import し、同じ public API 名の wrapper から scanner::str_* を呼ぶと、import された同名関数が shadow warning として扱われた後に scanner::str_* が D3001 undefined になる。

## 影響

stdlib の巨大 file 分割で public facade が既存 API 名を保ったまま実装 submodule へ同名委譲できず、実装関数へ module-specific prefix を付ける不自然な回避が必要になる。

## 修正方針

alias-qualified lookup と local symbol shadowing を分離し、同名 local wrapper が存在しても imported module namespace の pub item を scanner::name で解決できるようにする。

## 検証

最小 fixture で submodule pub fn f と facade fn f が共存し、facade body の sub::f 呼び出しが compile することを固定する。
