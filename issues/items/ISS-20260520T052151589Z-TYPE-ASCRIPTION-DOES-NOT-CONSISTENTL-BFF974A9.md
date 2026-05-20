---
id: ISS-20260520T052151589Z-TYPE-ASCRIPTION-DOES-NOT-CONSISTENTL-BFF974A9
title: "Type ascription does not consistently enter expected-check mode"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/trait_call_apply.rs, nepl-core/src/typecheck"
---

# ISS-20260520T052151589Z-TYPE-ASCRIPTION-DOES-NOT-CONSISTENTL-BFF974A9: Type ascription does not consistently enter expected-check mode

## 概要

Prefix type annotations are parsed and applied, but the checker still relies on post-inference unification in several paths. Fully annotated expressions should be checked against the expected type without unnecessary overload or generic search.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/trait_call_apply.rs, nepl-core/src/typecheck`

## 根拠

- `plan.md` は `<T>` 形式の型注釈を続く式への前置注釈として定義している。
- `nepl-core/src/parser.rs` / `nepl-core/src/ast.rs` には `PrefixItem::TypeAnnotation` があり、構文上は任意式の前置型注釈を扱えている。
- `nepl-core/src/typecheck/prefix_check.rs` は `pending_ascription` により型注釈を保持するが、tuple 状態であり、推論モードと期待型に対する検査モードの責務分割が型で明示されていない。
- `nepl-core/src/typecheck/overload_selection.rs`、`trait_call_apply.rs` には `expected_ret` が渡る経路があるが、expected type constraint と必要探索の境界が設計として固定されていない。
- 設計方針として、型安全・静的検査・enum/match による網羅性・技術的負債を残さないことが必須である。

## 問題

Prefix type annotations are parsed and applied, but the checker still relies on post-inference unification in several paths. Fully annotated expressions should be checked against the expected type without unnecessary overload or generic search.

## 影響

Compile time can grow from avoidable candidate exploration, and the typechecker design remains unclear about when it is inferring versus verifying an annotated expression.

## 修正方針

Introduce an explicit expected-check mode for type ascription boundaries while preserving inference when no annotation is present. Use expected types to prune overload, trait, and generic resolution before expensive exploration.

設計書: [NEPLg2 型注釈 expected-check 設計計画](../../doc/neplg2/type_ascription_expected_check_plan.md)

型注釈は探索を完全に停止する印ではない。generic type parameter、trait application、receiver type、overload argument の制約が不足している場合は、必要十分な探索を行う。ただし探索は無制限な総当たりにせず、expected type、argument constraints、effect context、trait bounds を typed constraint として集約し、候補を採用または拒否した根拠を enum / struct で保持する。

`<T>` が十分な情報を与える場合は、その式を `Check(T)` として処理し、不要な候補展開を避ける。`<T>` だけでは不十分な場合は、注釈なし推論へ無秩序に戻るのではなく、不足している制約だけを探索し、制約が閉じても一意に決まらなければ曖昧性または注釈不足として診断する。

## 検証

Add focused compiler regressions and source-policy tests proving annotated expressions select/check directly, unannotated expressions still infer, and no string allowlist replaces typed expected-type evidence.

追加する regression は少なくとも次を含める。

- 注釈なしでは overload ambiguous になる式が、期待戻り値注釈で一意に決まる。
- 注釈があっても generic type parameter が不足する式では、引数型や trait bound から必要探索して解ける。
- 注釈があっても制約が閉じた後に複数候補が残る式では、無理に選ばず曖昧性診断を出す。
- 注釈あり経路でも stdlib 関数名 allowlist や文字列 key に依存しない。

## 2026-05-20 Stage B TypeExpectation model 導入

実装前半として、`pending_ascription: Option<(TypeId, usize)>` と call reduction の expected tuple を `TypeExpectation` model へ移した。`TypeExpectation` は target type、base depth、span、source enum を保持し、現時点では挙動変更を最小化して既存の ascription 適用タイミングを維持する。

追加した source は次の通り。

- `ExplicitAscription`: `<T>` による明示注釈。
- `BlockResult`: function body や block の最終式に要求される戻り値型。
- `OuterConsumerArgument`: pipe target など外側 consumer から来る引数期待型。

`prefix_check.rs` と `call_reduction.rs` は期待型を naked tuple として扱わず、`TypeExpectation::target` / `base_depth` / `call_result_target_after_args` を通す。`call_resolution.rs` の pipe pending segment も `TypeExpectation::outer_consumer_argument` を使う。明示注釈由来の mismatch は注釈 span を診断 primary span として使い、block result / outer consumer 由来の mismatch は式側 span を使う。

この stage は typed state の導入であり、overload / generics / trait の探索削減そのものは次 stage で行う。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test typeannot -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_enum_none_typed_by_ascription -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_ascription_mismatch_is_error -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass
- `node nodesrc/issues.js check`: pass

補足: `cargo test -p nepl-core --test overload -- --nocapture` は 5/8 passed, 3 failed。失敗内容は今回追加した TypeExpectation ではなく、既存 stdlib shadow warning を `overload.rs` の helper が diagnostic 非空として panic する既存状態だったため、今回の commit の完了条件からは focused case を採用した。

## 2026-05-20 Stage C/D bridge expected result evidence 伝播

Stage C の前半として、call reduction から先の `expected_ret: Option<TypeId>` 境界を `Option<TypeExpectation>` へ移した。対象は `apply_function`、`select_overload_candidate`、selected call、indirect call、trait method call である。

これにより、明示注釈 / block result / outer consumer argument の由来を call 適用層まで保持したまま、overload 選択や generic / trait result constraint に使える。特に selected generic call では、type args を確定して monomorphization 情報を作る前に `c_result` と expected target を unify するため、結果型注釈が generic instantiation の根拠として失われない。

今回完了した範囲:

- `TypeExpectation::call_result_expectation_after_args` で call result に適用できる期待型そのものを返す。
- `select_overload_candidate` は expected result を `TypeExpectation` として受ける。
- selected direct call / indirect call / trait method call は expected result mismatch を `TypeExpectation::diagnostic_span` で診断する。
- `nodesrc/test_type_expectation_model_policy.js` で call 適用層が `expected_ret: Option<TypeId>` に戻らないことを監視する。

残作業:

- Stage C 後半: instantiate 前の候補分類と候補数 guard。
- Stage D 後半: 不足している type parameter / trait application を constraint object として保持し、必要十分な探索だけにする。
- Stage E: compile-time regression guard。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test typeannot -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test generics -- --nocapture`: 24 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass

## 2026-05-20 Stage C overload declared-shape pruning

Stage C 後半の一部として、`select_overload_candidate` で checkpoint / instantiate する前に binding の declared function shape を分類するようにした。これにより、関数でない候補、明示 type args 数不一致、capture 数不整合、arity 不一致は fresh type variable を作る前に除外される。

設計上の意図:

- expected-check は単に最後の unify を早くするだけでなく、探索対象そのものを typed rule で減らす。
- stdlib 関数名や module allowlist ではなく、候補の型シグネチャそのものから削れる候補を削る。
- source policy で、`select_overload_candidate` が declared shape を見てから checkpoint / instantiate する順序を監視する。

今回完了した範囲:

- declared `TypeKind::Function` の取得を instantiate 前へ移動。
- explicit type args count / capture len / arity の mismatch を instantiate 前に `continue` する。
- `nodesrc/test_type_expectation_model_policy.js` に declared-shape pruning の順序検査を追加。

残作業:

- expected result の型構造まで使った pre-instantiate pruning。
- candidate count guard / test-only counter。
- ambiguity 診断の理由構造化。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_overload_cast_like -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_make_none_from_context -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_make_some_wrapper -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass
