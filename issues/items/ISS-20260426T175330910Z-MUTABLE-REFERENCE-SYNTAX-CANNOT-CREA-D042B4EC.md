---
id: ISS-20260426T175330910Z-MUTABLE-REFERENCE-SYNTAX-CANNOT-CREA-D042B4EC
title: "mutable reference syntax cannot create unique borrow"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/parser.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs"
---

# ISS-20260426T175330910Z-MUTABLE-REFERENCE-SYNTAX-CANNOT-CREA-D042B4EC: mutable reference syntax cannot create unique borrow

## 概要

型システムには &mut T と BorrowKind::Unique があり、unique borrow 用の診断IDも存在するが、式構文の & は常に shared reference を生成する。&mut x を受理しないため、&mut T parameter や local unique borrow を通常の NEPLg2 ソースから作れず、unique borrow 検査が実質的に到達しない。

## 対象

- `nepl-core/src/parser.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs`

## 根拠

- `nepl-core/src/parser.rs` の expression parser は `TokenKind::Ampersand` を常に `Symbol::AddrOf` として扱い、後続の `mut` を消費しない。
- `nepl-core/src/typecheck.rs` の `Symbol::AddrOf` lowering は `ctx.reference(a, false)` を固定で生成し、式から `&mut T` を作れない。
- `nepl-core/src/passes/move_check.rs` には `BorrowKind::Unique` と unique borrow 用診断があるが、通常ソースから local unique borrow を生成する経路がない。

## 問題

型システムには &mut T と BorrowKind::Unique があり、unique borrow 用の診断IDも存在するが、式構文の & は常に shared reference を生成する。&mut x を受理しないため、&mut T parameter や local unique borrow を通常の NEPLg2 ソースから作れず、unique borrow 検査が実質的に到達しない。

## 影響

borrow 検査が形式上だけ存在する状態になり、self-host compiler や owning collection が一意可変参照を必要としたときに、型安全な API を表現できない。&mut T を期待する関数を呼べないため、mutable reference と shared reference の排他検査を回帰テストで固定できない。

## 修正方針

parser/AST/typecheck で &mut expr を expression syntax として扱い、HIR AddrOf の型を &mut T として生成する。move_check は AddrOf の reference mutability から BorrowKind::Shared / Unique を決め、local borrow と reference argument の両方で unique borrow を保持・解放・衝突検査する。

## 検証

&mut の一時 borrow 呼び出し、local unique borrow が owner move を阻止するケース、shared/unique borrow の相互排他、last-use release の compiler 回帰テストを追加する。
