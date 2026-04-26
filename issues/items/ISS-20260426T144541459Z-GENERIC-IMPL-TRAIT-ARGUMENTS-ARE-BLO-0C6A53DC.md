---
id: ISS-20260426T144541459Z-GENERIC-IMPL-TRAIT-ARGUMENTS-ARE-BLO-0C6A53DC
title: "generic impl trait arguments are blocked for non-capability traits"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/typecheck.rs; stdlib/core/traits/hash.nepl"
---

# ISS-20260426T144541459Z-GENERIC-IMPL-TRAIT-ARGUMENTS-ARE-BLO-0C6A53DC: generic impl trait arguments are blocked for non-capability traits

## 概要

capability を持たない trait に対して `impl<.K> Trait<.K> for Concrete` を書くと、target 型が concrete でも D3082 `impl type parameters are not supported yet` で拒否される。

## 対象

- `nepl-core/src/typecheck.rs; stdlib/core/traits/hash.nepl`

## 根拠

`Hasher<.K>` から独自 `#capability clone/copy` を外すと `impl<.K: HashKey> Hasher<.K> for DefaultHash32` がこの制限に当たる。generic 型引数は trait application 側だけに現れており、impl target は `DefaultHash32` の concrete 型である。

## 問題

capability を持たない trait に対して `impl<.K> Trait<.K> for Concrete` を書くと、target 型が concrete でも D3082 `impl type parameters are not supported yet` で拒否される。

## 影響

stdlib の `Hasher` が標準 `Clone` / `Copy` に寄せられず、hash collection が compiler 制限を回避するための不自然な capability trait に依存し続ける。self-host compiler の symbol table / intern table でも generic trait impl を使った基盤型を作りにくい。

## 修正方針

impl の type parameter を一律拒否せず、target 型が unbound type variable を含む場合だけ従来通り non-capability trait では拒否する。trait argument 側にだけ現れる type parameter は `TraitBoundRef` / impl method checking の既存経路で扱えるため許可する。

## 検証

non-capability trait `Apply<.T>` に対する `impl<.T> Apply<.T> for Concrete` が通る回帰を追加し、generic target `impl<.T> Trait for Box<.T>` の拒否は維持する。RV-STDLIB-012 の HashKey/Hasher cleanup 後に stdlib hash tests を実行する。

## 対応結果

`nepl-core/src/typecheck.rs` の impl type parameter 事前拒否を外し、既存の concrete target 判定へ一本化した。これにより、impl target が concrete な `impl<.T> Trait<.T> for i32` は通り、target 型自体が generic な `impl<.T> Trait for Box<.T>` は D3084 のまま拒否される。

## 回帰テスト

- `nepl-core/tests/neplg2.rs::impl_type_params_in_trait_args_allowed_for_concrete_target`
- `tests/compiler/generic_impl_trait_args.n.md`
