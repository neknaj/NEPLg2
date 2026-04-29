---
id: ISS-20260429T041244376Z-FACADE-MODULE-CANNOT-DISAMBIGUATE-SA-7F898FA5
title: "facade module cannot disambiguate same-name imported implementation with alias-qualified call"
area: core
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/typecheck/env.rs, nepl-core/src/typecheck/driver.rs, nepl-core/tests/import_clause.rs"
---

# ISS-20260429T041244376Z-FACADE-MODULE-CANNOT-DISAMBIGUATE-SA-7F898FA5: facade module cannot disambiguate same-name imported implementation with alias-qualified call

## 概要

alloc/string.nepl が alloc/string/scanner.nepl を as scanner で import し、同じ public API 名の wrapper から scanner::str_* を呼ぶと、import された同名関数が shadow warning として扱われた後に scanner::str_* が D3001 undefined になる。

## 対象

- `nepl-core/src/typecheck/env.rs`
- `nepl-core/src/typecheck/driver.rs`
- `nepl-core/tests/import_clause.rs`

## 根拠

- `Env::remove_duplicate_func` が同名・同シグネチャ関数を file/module 境界なしに削除していた。
- loader は import graph を flat module として typecheck へ渡すため、facade wrapper と実装 submodule の同名関数は同じ global callable scope に並ぶ。
- facade wrapper 登録時に実装 submodule 側の callable binding が削除され、`impls::scan` の qualified lookup が alias target file を持っていても候補 binding を見つけられなかった。

## 問題

alloc/string.nepl が alloc/string/scanner.nepl を as scanner で import し、同じ public API 名の wrapper から scanner::str_* を呼ぶと、import された同名関数が shadow warning として扱われた後に scanner::str_* が D3001 undefined になる。

## 影響

stdlib の巨大 file 分割で public facade が既存 API 名を保ったまま実装 submodule へ同名委譲できず、実装関数へ module-specific prefix を付ける不自然な回避が必要になる。

## 修正方針

同一 source file 内での duplicate function replacement と、別 file/module の同名 callable binding を分離する。同名 local wrapper が存在しても imported module namespace の pub item を `scanner::name` で解決できるようにする。

## 検証

最小 fixture で submodule pub fn f と facade fn f が共存し、facade body の sub::f 呼び出しが compile することを固定する。

## 対応結果

2026-04-29 に、同名・同シグネチャ関数の置換を同一 source file 内に限定した。これにより、同じ flat env に存在する別 module の同名関数は qualified lookup 用に保持される。

追加 regression:

- `nepl-core/tests/import_clause.rs::alias_qualified_call_survives_same_name_facade_wrapper`

検証:

- `cargo test -p nepl-core --test import_clause alias_qualified_call_survives_same_name_facade_wrapper -- --nocapture`
- `cargo test -p nepl-core --test import_clause -- --nocapture`
- `cargo check -p nepl-core --tests`
