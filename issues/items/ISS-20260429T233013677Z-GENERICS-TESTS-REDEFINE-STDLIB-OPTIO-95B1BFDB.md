---
id: ISS-20260429T233013677Z-GENERICS-TESTS-REDEFINE-STDLIB-OPTIO-95B1BFDB
title: "generics tests redefine stdlib Option after importing core mem"
area: core
status: verified
resolved: true
priority: P2
type: test
created: 2026-04-29
updated: 2026-04-30
target: nepl-core/tests/generics.rs
---

# ISS-20260429T233013677Z-GENERICS-TESTS-REDEFINE-STDLIB-OPTIO-95B1BFDB: generics tests redefine stdlib Option after importing core mem

## 概要

複数の generics integration tests が `#import "core/mem" as *` の後に local `enum Option<.T>` を定義している。現在の `core/mem` は public API で `Option` を使うため `core/option` を読み込み、test 側の `Option` 再定義が `Resolve(ItemNameConflict)` で失敗する。

## 対象

- `nepl-core/tests/generics.rs`

## 根拠

- `cargo test -p nepl-core --test generics generics_enum_option_and_match -- --nocapture`: `Resolve(ItemNameConflict)` / `name already used by another item`
- `cargo test -p nepl-core --tests -- --nocapture`: deep-chain / doc / drop は通過した後、`generics` の 8 tests が同じ item name conflict で失敗した。
- Resource IR lowering 差分を stash した状態でも同じ失敗が再現したため、Resource IR 修正の副作用ではなく test fixture の stdlib 名衝突として切り分けた。

## 問題

複数の generics integration tests が `#import "core/mem" as *` の後に local `enum Option<.T>` を定義している。現在の `core/mem` は public API で `Option` を使うため `core/option` を読み込み、test 側の `Option` 再定義が `Resolve(ItemNameConflict)` で失敗する。

## 影響

compiler generics の回帰テストが stdlib の基本 `Option` 名と衝突し、`cargo test -p nepl-core --tests` が generics まで進んだ時点で失敗する。Resource IR / drop 修正の全体回帰確認も妨げる。

## 修正方針

stdlib の `Option` と衝突しない test-only enum 名へ置き換え、これらの tests が generic enum 自体を検証し続けるようにする。variant `Some` / `None` は scrutinee 型文脈の generic enum variant 解決を検証するため維持する。

## 対応

- `core/mem` を import する generics integration tests の local `Option` enum を `TestOption` に変更した。
- `TestOption::Some` / `TestOption::None` と `TestOption<T>` に型参照を揃え、stdlib `Option` との item name conflict を回避した。
- compile-fail 系の test-only enum も `TestOption` に揃え、test file 内の fixture 命名を一貫させた。

## 検証

- `cargo test -p nepl-core --test generics generics_enum_option_and_match -- --nocapture`: reproduced before fix
- `cargo test -p nepl-core --test generics -- --nocapture`: `24 passed`
- `rustfmt --check nepl-core/tests/generics.rs`: passed
- `git diff --check -- nepl-core/tests/generics.rs`: passed
