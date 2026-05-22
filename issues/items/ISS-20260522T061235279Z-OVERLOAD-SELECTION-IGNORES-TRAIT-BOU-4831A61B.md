---
id: ISS-20260522T061235279Z-OVERLOAD-SELECTION-IGNORES-TRAIT-BOU-4831A61B
title: "Overload selection ignores trait bounds for same-signature generic candidates"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/trait_bound_apply.rs, nepl-core/tests/neplg2.rs"
---

# ISS-20260522T061235279Z-OVERLOAD-SELECTION-IGNORES-TRAIT-BOU-4831A61B: Overload selection ignores trait bounds for same-signature generic candidates

## 概要

Overload candidate narrowing deduplicates same-signature generic functions before checking type parameter trait bounds. A Copy-bound candidate can shadow a Drop-bound candidate with the same value signature, producing TraitBoundUnsatisfied even when another overload is valid. This blocks non-Copy Vec cleanup APIs from using generic source-proven Drop traversal without ad hoc names.

## 対象

- `nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/trait_bound_apply.rs, nepl-core/tests/neplg2.rs`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 関連 doc: [NEPLg2 型注釈 expected-check 設計計画](../../doc/neplg2/type_ascription_expected_check_plan.md)
- 開発方針: https://zenn.dev/bem130/articles/1b352797de94e7

## 問題

Overload candidate narrowing deduplicates same-signature generic functions before checking type parameter trait bounds. A Copy-bound candidate can shadow a Drop-bound candidate with the same value signature, producing TraitBoundUnsatisfied even when another overload is valid. This blocks non-Copy Vec cleanup APIs from using generic source-proven Drop traversal without ad hoc names.

## 影響

Bound-specific abstraction APIs cannot be expressed safely. Static verification for collection cleanup would either keep Copy-only APIs or introduce separate names, both of which avoid the compiler's overload/type-bound proof instead of relying on it.

## 修正方針

Filter overload candidates against their instantiated type parameter bounds before signature deduplication. Concrete unsatisfied bounds are rejected immediately. In overload sets, unresolved generic bounds are not selectable unless the current generic function contract proves them; single-candidate calls still report the ordinary post-selection trait-bound diagnostic. Do not use stdlib allowlists or string-based special cases.

## 検証

Add regression tests with same-signature Copy-like and Drop-like overloads so concrete calls choose the bound-satisfied candidate and unsatisfied bound candidates do not shadow valid ones.

## 解決内容

2026-05-22 に Agent 1 が overload candidate selection を trait-bound-aware にした。

- `select_overload_candidate` が overload 候補を signature dedup へ渡す前に、instantiated type argument と関数 type parameter bound を照合するようにした。
- 具体型に対して明らかに満たせない bound を持つ候補を候補から落とし、overload 集合では未確定型変数の bound が現在の generic 関数契約から証明できる候補だけを選択可能にした。
- 同一 function type で trait bound だけが違う overload について、関数登録 / shadow 判定 / duplicate 除去では value signature と normalized bound signature を合わせて比較するようにした。
- 単一候補の generic call は従来どおり選択後の `check_selected_function_trait_bounds` で `TraitBoundUnsatisfied` を出すため、既存診断の意味を変えない。
- stdlib 関数名や module 名の allowlist は使っていない。抽象化機能の trait bound proof を compiler の汎用 overload selection に反映した。

## 回帰テスト

- `cargo test -p nepl-core --test neplg2 overload_selection_ -- --test-threads=1 --nocapture`
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --test-threads=1 --exact --nocapture`
