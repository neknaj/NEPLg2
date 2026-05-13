---
id: ISS-20260513T160056481Z-GENERIC-TRAIT-LOOKUP-REGRESSION-FIXT-21C1894C
title: "generic trait lookup regression fixture relies on implicit math import"
area: core
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/tests/neplg2.rs
---

# ISS-20260513T160056481Z-GENERIC-TRAIT-LOOKUP-REGRESSION-FIXT-21C1894C: generic trait lookup regression fixture relies on implicit math import

## 概要

generic_trait_impl_method_resolves_by_trait_args was intended to guard generic trait argument based impl resolution, but the fixture calls eq without importing core/math. Current NEPLg2 no longer treats eq as an implicit prelude symbol, so the test fails with IdentifierUndefined before exercising trait lookup.

## 対象

- `nepl-core/tests/neplg2.rs`

## 根拠

- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture` が `Resolve(IdentifierUndefined)` で失敗した。
- primary span は fixture 内の `eq a b` を指しており、trait argument に基づく impl resolution に到達する前に、fixture の未import symbolで落ちていた。
- 同じテストは monomorphize trait lookup / abstraction static verification の回帰境界であり、数学関数の暗黙prelude有無を検査する目的ではない。

## 問題

generic_trait_impl_method_resolves_by_trait_args was intended to guard generic trait argument based impl resolution, but the fixture calls eq without importing core/math. Current NEPLg2 no longer treats eq as an implicit prelude symbol, so the test fails with IdentifierUndefined before exercising trait lookup.

## 影響

The abstraction regression suite reports a generic/trait failure that is not caused by trait resolution, obscuring real static verification issues and making the trait lookup gate unreliable.

## 修正方針

Import core/math explicitly in the fixture so the test remains focused on generic trait impl resolution by trait arguments.

## 対応

- fixture に `#import "core/math" as *` を追加し、`eq` を明示依存にした。
- trait / generic の本題である `Hasher<.K>` impl resolution は変更していない。

## 検証

- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
