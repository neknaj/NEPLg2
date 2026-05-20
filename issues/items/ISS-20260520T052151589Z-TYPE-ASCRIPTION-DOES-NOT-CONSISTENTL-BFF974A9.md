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

## 2026-05-20 Stage C expected-result shape pruning

multiple overload かつ明示 type args がない場合に、declared result と expected target を `TypeCtx::type_pattern_matches` で照合し、結果型が明らかに合わない候補を checkpoint / instantiate 前に除外するようにした。

設計上の意図:

- generic result の type parameter は pattern variable として扱い、`Option<T>` と `Option<i32>` のように成立し得る候補は残す。
- `i32` と `bool` のように top-level で成立しない候補は fresh type variable を作る前に落とす。
- explicit type args がある候補では、未代入の declared result だけで判断すると誤除外する可能性があるため、既存の substitution 後 unify に任せる。

今回完了した範囲:

- `select_overload_candidate` に `type_pattern_matches(result, expectation.target())` による expected result shape pruning を追加。
- source policy で、この pruning が checkpoint / instantiate より前にあることを監視。

残作業:

- candidate count guard / test-only counter。
- ambiguity 診断の理由構造化。
- Stage D の不足 constraint object 化。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_overload_cast_like -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_make_none_from_context -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_ascription_mismatch_is_error -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass

## 2026-05-20 Stage C/E overload candidate count guard

Stage C の候補 pruning と Stage E の performance guard を接続するため、overload candidate の rejection reason と materialization count を typed state として記録するようにした。

実装:

- `OverloadCandidateRejection` enum を追加し、候補除外理由を文字列ではなく typed variant として扱う。
- `OverloadCandidateStats` を追加し、considered / materialized / accepted / reason別 rejection count を保持する。
- `record_rejection` は `match` で全 variant を網羅し、reason 追加時に compile-time の見落としを起こしにくくする。
- `assert_materialization_guard` で `materialized + pre_materialized_rejections <= considered` を `debug_assert!` し、事前 pruning した候補を materialize してしまう退行を検出しやすくした。
- source policy で enum、exhaustive match、materialization guard、materialization count が残っていることを監視する。

残作業:

- ambiguity 診断の理由 payload 化。
- Stage D の不足 type parameter / trait application constraint object 化。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_overload_cast_like -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_make_none_from_context -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass
- `node nodesrc/issues.js check`: pass

## 2026-05-20 Stage D trait inference constraint object

trait application inference が `expected_ret.map(|expectation| expectation.target())` で `TypeExpectation` を `TypeId` に落としていたため、expected result 由来の制約であることを型で追跡できていなかった。これを `TypeParamInferenceSource` / `TypeParamInferenceConstraint` へ移した。

実装:

- `TypeParamInferenceSource::{Argument, ExpectedResult}` を追加。
- `TypeParamInferenceConstraint { source, original, actual }` を追加。
- `infer_trait_application_args` は `Option<TypeExpectation>` を直接受け、argument と expected result を同じ constraint list に正規化する。
- constraint ごとの type parameter 推論は `match self.source` を通すため、制約 source を増やす場合に網羅性検査が効く。
- `trait_call_apply.rs` は `expected_ret.map(...target...)` で expected evidence を消さず、そのまま inference へ渡す。

残作業:

- constraint が閉じても一意に決まらない場合の typed diagnostic payload。
- generic function 側の不足 constraint object 化。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test generics generics_make_some_wrapper -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_make_none_from_context -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test typeannot -- --nocapture`: 12 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass

## 2026-05-20 Stage C overload ambiguity reason payload

overload ambiguity が最終的に `"ambiguous overload"` だけを出しており、どの narrowing stage の後に候補が残ったかを typed state として保持していなかった。`OverloadCandidateNarrowingStage` と `OverloadAmbiguityReason` を追加し、diagnostic message を payload から生成するようにした。

実装:

- `OverloadCandidateNarrowingStage` enum を追加し、pure preference、signature dedup、ordinary preference、concrete preference、type parameter count、instantiated specificity、declared specificity を typed stage として表す。
- `OverloadAmbiguityReason { after_stage, remaining_candidates }` を追加。
- final ambiguity diagnostic は `OverloadAmbiguityReason::diagnostic_message()` から作る。
- `unannotated_result_overload_reports_typed_ambiguity_reason` を追加し、型注釈なし overload ambiguity が stage/candidate count を含むことを確認する。
- source policy で narrowing stage enum、ambiguity payload、payload 由来診断を監視する。

残作業:

- generic function 側の不足 constraint object 化。
- trait/generic constraint が閉じても一意に決まらない場合のより詳細な typed diagnostic payload。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test overload unannotated_result_overload_reports_typed_ambiguity_reason -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_overload_cast_like -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics generics_make_none_from_context -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass
- `node nodesrc/issues.js check`: pass

## 2026-05-20 Stage D generic call constraint object

selected generic function call では、argument mismatch と expected result mismatch がそれぞれ直接 `ctx.unify(...)` を呼び、制約の由来を型で追跡できていなかった。さらに `id<T>(T)->T` のような関数では、期待戻り値から `T` を拘束する前に char literal 引数を検査するため、`let x <u8> id '\x02'` のような十分な注釈つき call でも expected-check の context が引数へ届きにくかった。

実装:

- `generic_call_constraints.rs` を追加し、`GenericCallConstraintSource::{Argument, ExpectedResult}` と `GenericCallConstraint { source, declared, instantiated, actual, span }` を導入した。
- selected call は expected result constraint を先に `GenericCallConstraint::check` へ通し、その後 argument constraint を同じ typed object で検査する。
- implicit generic type argument は `infer_instantiated_type_arg` の whole-signature fallback ではなく、保持した call constraints から `resolve_generic_type_args_from_constraints` で解く。
- source policy は generic call constraint enum / struct、`match self.source` による type parameter 推論、expected-result-before-argument の順序、直接 `ctx.unify(c_result, expectation.target())` への退行禁止を監視する。

残作業:

- constraint が閉じても一意に決まらない場合の typed diagnostic payload 化。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test generics -- --nocapture`: 25 passed
- `cargo test -p nepl-core --test typeannot -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass

## 2026-05-20 Stage D generic constraint conflict payload

generic call constraint object を selected call に入れただけでは不十分だった。`same<T>(T,T)` に `i32` と `bool` を渡すような呼び出しは、selected call に到達する前に overload selection の候補検査で落ちるため、矛盾が `OverloadNoMatch` に潰れていた。

実装:

- `TypeDiagnosticCode::GenericConstraintConflict` を追加し、constraint が閉じた後に type parameter へ複数の異なる型が要求されたことを typed diagnostic として表す。
- `TypeArgumentInference::{NoEvidence, Unique, Conflict}` と `TypeArgumentResolution { resolved_args, conflicts }` を追加し、no evidence と conflict を `Option<TypeId>` の `None` にまとめないようにした。
- selected call は conflict payload を `GenericConstraintConflict` として報告する。
- overload selection も expected result と argument evidence を `GenericCallConstraint` に集約し、候補棄却理由 `OverloadCandidateRejection::GenericConstraintConflict` として保持する。
- overload selection では expected result constraint を先に適用してから argument constraint を検査するため、期待型が char literal や generic parameter の context として働く。
- `generics_same_type_param_mismatch_is_error` は `GenericConstraintConflict` を明示的に要求する regression になった。
- source policy は overload selection が `GenericCallConstraint` を使い、payload 由来で `GenericConstraintConflict` を報告することを監視する。

残作業:

- trait application 側の constraint conflict / ambiguity payload は別途 Stage D の残作業として扱う。
- `infer_type_param_from_instantiated_pair` 内部の単一制約内 conflict 表現は、必要なら別 issue として型付き inference payload へ拡張する。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test generics -- --nocapture`: 25 passed
- `cargo test -p nepl-core --test typeannot -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_overload_cast_like -- --nocapture`: 1 passed
- `cargo test -p nepl-core diagnostic_codes_have_unique_serialized_names -- --nocapture`: pass
- `node nodesrc/test_type_expectation_model_policy.js`: pass
- `node nodesrc/issues.js check`: pass

## 2026-05-20 Stage D shared type argument inference and trait conflict payload

generic function call と trait application がそれぞれ別に type argument 推論の merge を持つと、静的検査の設計が分散し、no evidence と conflict を再び混同しやすい。前 commit で generic call 側に導入した conflict payload を trait 側へ個別コピーせず、共通の type argument inference model へ切り出した。

実装:

- `type_argument_inference.rs` を追加し、`TypeArgumentConstraint`、`TypeArgumentInference::{NoEvidence, Unique, Conflict}`、`TypeArgumentResolution`、`TypeArgumentConflict` を共通化した。
- `generic_call_constraints.rs` は call source enum を保持したまま、type argument の解決を共通 resolver に委譲する。
- `trait_check.rs` は `merge_inferred_instantiation` を使わず、`TypeParamInferenceConstraint` から共通 `TypeArgumentConstraint` を作って resolver に渡す。
- `TraitMethodResolution::ConstraintConflict` と `TypeDiagnosticCode::TraitConstraintConflict` を追加し、trait application の argument evidence と expected result evidence が矛盾した場合に typed diagnostic を出す。
- `prefix_check.rs` の trait method value 推論も `resolve_trait_application_args` を使い、conflict payload が出た場合は同じ診断へ流す。
- `trait_application_type_param_conflict_has_type_code` を追加し、`Mapper<T>::map(Self,T)->T` に `T=i32` と期待結果 `T=bool` が同時に要求された場合を固定した。

残作業:

- trait application が conflict ではなく複数候補を残す ambiguity payload は、後続の Stage D 作業で対応済み。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test neplg2 trait_application_type_param_conflict_has_type_code -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 trait_method_call_with_impl_compiles -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 impl_type_params_in_trait_args_allowed_for_concrete_target -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics -- --nocapture`: 25 passed
- `cargo test -p nepl-core --test typeannot -- --nocapture`: 12 passed
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test overload test_overload_cast_like -- --nocapture`: 1 passed
- `cargo test -p nepl-core diagnostic_codes_have_unique_serialized_names -- --nocapture`: pass
- `node nodesrc/test_type_expectation_model_policy.js`: pass
- `node nodesrc/issues.js check`: pass

## 2026-05-20 Stage D trait self type ambiguity payload

trait application の type argument conflict は共通 resolver に移ったが、trait method の self type 推論はまだ「候補なし」と「候補が複数あり一意に選べない」を `Option<TypeId>` の `None` に畳み込める構造だった。このままだと、`Factory::make` のような unbound trait method value を generic context 内で使ったとき、`.A: Factory` と `.B: Factory` の両方が候補になる状況を fresh `Self` や後段の `TraitBoundUnsatisfied` に潰してしまう。

実装:

- `TraitSelfTypeInference::{NoEvidence, Unique, Ambiguous}` と `TraitSelfTypeAmbiguity` を追加し、trait application identity と候補 self type を typed payload として保持する。
- `infer_unique_type_param_for_trait_ref -> Option<TypeId>` を廃止し、`resolve_self_type_param_for_trait_ref` が no evidence / unique / ambiguous を enum で返すようにした。
- `TraitMethodResolution::SelfTypeAmbiguous` と `TypeDiagnosticCode::TraitSelfTypeAmbiguous` を追加し、prefix trait method value 構築と unbound trait method call の両方で ambiguity payload から診断を生成する。
- `selected_call_apply.rs` も `TraitMethodResolution` の新 variant を明示的に match し、trait method resolution state を optional fallback に戻さない。
- source policy は trait self type inference enum / ambiguity payload / 旧 optional helper 禁止 / prefix と call apply の typed diagnostic を監視する。
- `trait_self_type_ambiguity_has_type_code` を追加し、同一 trait bound を持つ複数 type parameter のもとで `Factory::make` の self type が一意に決まらないケースを固定した。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test neplg2 trait_self_type_ambiguity_has_type_code -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 trait_application_type_param_conflict_has_type_code -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 trait_method_call_with_impl_compiles -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test generics -- --nocapture`: 25 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass

## 2026-05-20 Stage E overload materialization phase guard

Stage E の探索量 guard は `OverloadCandidateStats::pre_materialized_rejections()` が rejection reason の手書き合計を返していた。この構造では、将来 `OverloadCandidateRejection` に新しい variant を追加したときに、その variant が instantiate 前に落ちるのか後に落ちるのかを guard 側へ入れ忘れやすい。型注釈で expected result pruning を行う目的は「十分な注釈がある経路では探索を広げない」ことなので、rejection reason 自身が materialization phase を持つ必要がある。

実装:

- `OverloadCandidateMaterializationPhase::{BeforeInstantiation, AfterInstantiation}` を追加した。
- `OverloadCandidateRejection::materialization_phase()` を追加し、全 rejection reason を exhaustive match で pre/post instantiation に分類する。
- `OverloadCandidateStats::record_rejection()` が reason の typed phase から `rejected_before_materialization` / `rejected_after_materialization` を更新するようにした。
- `pre_materialized_rejections()` は手書き reason 合計ではなく `rejected_before_materialization` を返す。
- source policy は materialization phase enum、phase 分類 match、phase counter 更新、既存 materialization guard を監視する。

検証:

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test overload test_explicit_type_annotation_prefix -- --nocapture`: 1 passed
- `node nodesrc/test_type_expectation_model_policy.js`: pass
