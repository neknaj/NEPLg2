# NEPLg2 abstraction static verification plan

作成日: 2026-05-12

関連 issue:

- [ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429](../../issues/items/ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429.md)
- [静的検査の複雑化解消計画](./static_check_complexity_reduction_plan.md)

## 目的

NEPLg2 の抽象化機能、特に generics、trait、trait bound、overload、monomorphize は、型安全とメモリ安全の入口である。抽象化機能だけを「便利機能」として扱い、文字列 key や optional field で通すと、Resource IR や effect / owner 検査が後段で正確な根拠を失う。

この文書は、抽象化機能にも次の開発方針を適用するための再設計計画である。

- 技術的負債を残さない。
- 後方互換のために不適切な設計を残さない。
- 暫定実装は許容しても、暫定の雑設計は禁止する。
- 静的検査の正確性を必須とする。
- 数値や文字列ではなく enum / typed struct で管理し、`match` の網羅性検査を効かせる。

## 現状評価

### 良い点

- `nepl-core/src/typecheck/` は `traits.rs`、`trait_check.rs`、`trait_bound_apply.rs`、`trait_call_apply.rs`、`overload_selection.rs`、`selected_call_apply.rs` などに分割済みで、巨大 `typecheck.rs` へ戻ってはいない。
- `TraitCapability` は `Copy` / `Clone` / `Drop` の enum であり、capability 判定は数値 tag ではない。
- `monomorphize` public API は unresolved trait call を panic せず `MonomorphizeResult` で返す。
- `tests/compiler/generics.n.md`、`tests/compiler/overload.n.md`、`tests/compiler/generic_impl_trait_args.n.md`、`tests/compiler/trait_capability_copy.n.md`、`tutorials/getting_started/18_generics.n.md`、`19_traits_and_bounds.n.md` が抽象化機能の利用例と regression を持つ。

### 対応状況

2026-05-13 時点で、当初の不足点は Stage 1-6 の作業で解消済みである。

1. trait application identity は typed model に移行済みである。
   - `parse_trait_ref_name` と `TraitBoundRef.name` は削除済みであり、表示文字列を authority として trait argument を復元しない。
   - typecheck は `TraitApplication { trait_id: TraitId, args }`、HIR は `HirTraitApplication`、Resource IR は `ResourceTraitApplication`、monomorphize は `MonoTraitApplication` を保持する。
   - `format_trait_ref_name` は `traits.rs` の diagnostic/display helper としてだけ残し、impl lowering / monomorphize lookup / Resource IR の判断材料には使わない。

2. impl model は `ImplKind` enum に移行済みである。
   - `ImplInfo` は `kind: ImplKind` を持ち、trait impl identity は `ImplKind::Trait { application, self_ty }` に集約した。
   - `trait_name: Option<String>` や `trait_base_name: Option<String>` のような optional field combination は再導入禁止にした。

3. type parameter bound identity は `BoundEnv` と `TypeParamId` に移行済みである。
   - 公開境界の raw `BTreeMap<TypeId, Vec<TraitBound>>` と label fallback は削除済みである。
   - `BoundEnv` 内部 key も `TypeParamId` になり、call site は type parameter declaration identity を明示して挿入する。

4. pending trait check は named model に移行済みである。
   - `pending_trait_bound_checks` は `Vec<PendingTraitCheck>` で保持し、`bound` / `target_ty` / `span` の意味を型と field 名で固定する。

5. monomorphize trait lookup は typed key に移行済みである。
   - `impl_map`、`impl_method_index`、`trait_lookup_cache` は `MonoTraitLookupKey` / `MonoTraitMethodKey` を key にする。
   - `impl_entry_index` は重複 index として削除済みであり、trait name / method string tuple key には戻さない。

6. source policy は final contract として監視済みである。
   - `nodesrc/test_abstraction_static_verification_policy.js` は baseline 増加抑止ではなく、typed model / enum model / named state / typed lookup key の存在と旧 model 再導入禁止を直接検査する。

## 目標設計

### TraitApplication

trait reference は表示文字列ではなく typed value として保持する。

```rust
struct TraitApplication {
    trait_id: TraitId,
    args: Vec<TypeId>,
}
```

`TraitId` は外部表示名ではなく compiler identity として扱う。現行の名前解決が保持する正規の宣言 identity を包み、`TraitApplication` の表示文字列や diagnostic 文面から逆算しない。

### TraitBound

type parameter bound は `TraitBoundRef.name` を廃止し、diagnostic だけが表示名を生成する。

```rust
struct TraitBound {
    application: TraitApplication,
    self_ty: TypeId,
}
```

bound satisfaction は `TraitApplication` 同士を比較し、表示文字列から type argument を parse しない。

### ImplKind

impl は field combination ではなく enum で表す。

```rust
enum ImplKind {
    Inherent,
    Trait {
        application: TraitApplication,
        self_ty: TypeId,
    },
}
```

これにより、trait impl なのに `trait_base_name` がない、inherent impl なのに trait args がある、といった不正状態を型で排除する。

### PendingTraitCheck

deferred check は named struct / enum にする。

```rust
struct PendingTraitCheck {
    bound: TraitBound,
    candidate: TypeId,
    span: Span,
}
```

将来、`ConcreteImplRequired`、`TypeParamBoundRequired`、`CapabilityRequired` のように分類が必要になったら enum variant として追加する。

### MonoTraitLookupKey

monomorphize の trait lookup key は typed struct にする。

```rust
struct MonoTraitLookupKey {
    application: TraitApplication,
    method: TraitMethodId,
    self_ty: TypeId,
}
```

`TraitMethodId` も display string ではなく compiler identity とする。diagnostic 出力時だけ文字列化する。

## 実装計画

### Stage 0: 現状凍結と監査

目的: string-based trait reference authority が増えないようにする。

作業:

- `nodesrc/test_abstraction_static_verification_policy.js` を追加し、`format_trait_ref_name`、`TraitBound`、`ImplInfo`、`trait_lookup_cache` の current baseline を固定する。
- `parse_trait_ref_name` は Stage 1 の作業で 0 baseline になっており、再導入禁止にした。
- baseline は最終状態ではない。今後の stage で残りの表示名 field / string key の数を下げる。

完了状態:

- 2026-05-13: Stage 6 の完了により、baseline 凍結は final contract へ移行した。`parse_trait_ref_name` / `TraitBoundRef` / `ImplInfo` optional field / split HIR・Resource・monomorphize lookup field の再導入禁止は構造検査で固定する。

### Stage 1: typed TraitApplication 導入

目的: trait reference を表示文字列ではなく typed value として渡す。

作業:

- `TraitApplication` / `TraitId` を typecheck abstraction module に追加する。
- `TraitBoundRef` を `TraitBound` へ置換する。
- `format_trait_ref_name` は diagnostic/display helper へ移動し、静的検査の分岐から外す。
- `parse_trait_ref_name` を削除する。

進捗:

- 2026-05-12: `function_check.rs` の deferred trait bound check は、`TraitBoundRef.name` の表示文字列を `parse_trait_ref_name` で復元する経路から、`trait_base_name` / `trait_args` を直接比較する typed lookup へ移行した。
- 2026-05-12: `prefix_check.rs` の trait method self inference は、表示名を作って `infer_unique_type_param_for_trait` で parse し直す経路から、`infer_trait_application_args` の `TypeId` 列を直接 `infer_unique_type_param_for_trait_ref` へ渡す経路へ移行した。これにより `parse_trait_ref_name` は削除済みになった。
- 2026-05-12: `TraitBoundRef` を `TraitBound` に改名し、表示名 field を削除した。診断と verbose log は `TraitBound::display_name` で境界生成する。
- 2026-05-12: `TraitApplication` を追加し、`TraitBound` は `application: TraitApplication` と `trait_self_ty` を持つ形にした。type parameter bound の trait identity は split field ではなく typed value になった。
- 2026-05-12: typecheck 側の impl matching も `TraitApplication` helper へ寄せた。`function_check.rs` / `trait_check.rs` / `trait_call_apply.rs` は `imp.trait_base_name` / `imp.trait_args` を直接読まず、`ImplInfo::matches_trait_application` を使う。
- 2026-05-13: `ISS-20260512T185123437Z-TYPECHECK-TRAITAPPLICATION-STILL-STO-F6F9CDD1` を追加し、typecheck-side `TraitApplication` の trait identity を `TraitId` newtype へ移行した。`BoundEnv` / impl matching / trait method inference は raw `&str` ではなく `TraitId` を受け取り、文字列化は diagnostic/display 境界に限定する。

検証:

- generic trait argument regression。
- non-primitive trait argument regression。
- nested apply trait argument regression。
- unknown trait / wrong arity diagnostic。

### Stage 2: ImplKind 導入

目的: impl の状態を optional field combination ではなく enum にする。

作業:

- `ImplInfo` を `Impl { kind: ImplKind, target_ty, type_params, methods }` 系へ再設計する。
- duplicate impl、missing method、method signature mismatch は `match ImplKind` で分岐する。
- inherent impl unsupported も `ImplKind::Inherent` の明示的診断にする。

検証:

- duplicate impl。
- generic impl trait args。
- generic target rejection。
- capability trait impl validation。

進捗:

- 2026-05-12: `ISS-20260512T153756004Z-IMPLINFO-STILL-ENCODES-TRAIT-IMPL-ID-A4ECD77B` を追加し、verified にした。
- 2026-05-12: `ImplKind` を追加し、`ImplInfo` を `kind: ImplKind` と `target_ty` の形へ移行した。trait impl identity は `ImplKind::Trait { application: TraitApplication, self_ty: TypeId }` に集約し、optional string field の組み合わせからは切り離した。
- 2026-05-12: duplicate impl / deferred trait check / trait bound satisfaction / trait method application は、split field ではなく `ImplInfo` helper 経由で trait application を照合する。
- 残件: final `HirImpl` と `monomorphize.rs` は Stage 5 の typed monomorphize lookup で対応する。ここでは typecheck-side static verification authority の enum 化を完了対象にした。

### Stage 3: BoundEnv と type parameter identity

目的: bound map の key を不安定な TypeId / label fallback から分離する。

作業:

- type parameter declaration identity を `TypeParamId` として保持する。
- `BoundEnv` を追加し、`TypeId` resolve と declaration identity の対応を一箇所で管理する。
- label fallback を削除し、必要なら明示的な substitution map で処理する。

検証:

- same label but different scope の bound 混線が起きないこと。
- nested generic function unsupported diagnostic。
- generic function calls generic。

進捗:

- 2026-05-12: `ISS-20260512T160137255Z-TRAIT-BOUND-LOOKUP-DUPLICATES-TYPEID-1694BE55` を追加し、verified にした。
- 2026-05-12: `trait_check.rs` の `BlockChecker::type_param_has_bound_ref` が `traits.rs` の `type_param_has_trait_application_bound` と同じ TypeId / label fallback lookup を再実装していたため、duplicate 実装を削除し、BlockChecker 側は typed helper へ委譲する薄い method にした。
- 2026-05-12: `ISS-20260512T160950280Z-TRAIT-BOUND-LOOKUP-STILL-ACCEPTS-SAM-C03A85E0` を追加し、verified にした。
- 2026-05-12: `type_param_has_trait_application_bound` から `TypeKind::Var` label 文字列が一致する別 `TypeId` を同一 bound とみなす fallback を削除した。
- 2026-05-12: `ISS-20260512T172241782Z-TRAIT-TYPE-PARAMETER-BOUNDS-STILL-EX-09CE8755` を追加し、verified にした。
- 2026-05-12: `BoundEnv` を追加し、type parameter trait bounds の raw `BTreeMap<TypeId, Vec<TraitBound>>` を `BlockChecker` / `BindingKind` / `check_function` の境界から外した。bound lookup は `BoundEnv::has_trait_application_bound` に閉じ込めた。
- 2026-05-12: `ISS-20260512T173702516Z-BOUNDENV-STILL-KEYS-TYPE-PARAMETER-B-792D9BA4` を追加し、`TypeParamId` を導入した。`BoundEnv` は `BTreeMap<TypeParamId, Vec<TraitBound>>` を保持し、`insert` / `iter` の境界でも raw `TypeId` を受け取らない。
- 2026-05-12: nested function check と top-level driver の type parameter bound collection は `TypeParamId::new(*p_id)` を明示して `BoundEnv` へ渡す形にし、source policy で raw `TypeId` key の再導入を拒否する。

### Stage 4: trait method resolution の構造化

目的: trait method call result を raw `FuncRef::Trait` 生成に直接落とさず、resolution result enum を経由させる。

作業:

- `TraitMethodResolution` enum を追加する。
- receiver inference、expected self type、type argument inference、effect check を分類する。
- failure は `TypeDiagnosticCode` / `EffectDiagnosticCode` の typed diagnostic にする。

検証:

- trait method type args unsupported。
- trait method not found。
- pure calls impure trait method。
- receiver / expected self type なしの診断。

進捗:

- 2026-05-12: `ISS-20260512T161908521Z-TRAIT-METHOD-RESOLUTION-STILL-RETURN-21525B05` を追加し、verified にした。
- 2026-05-12: `TraitMethodResolution` enum と `TraitMethodCall` model を追加し、selected callable / unbound trait method call の receiver inference と trait application inference を共通 resolver に集約した。
- 2026-05-12: `infer_selected_trait_method_callee -> Option<FuncRef>` は削除した。selected callable 側は `TraitMethodResolution` を match し、unbound 側は typed failure variant から diagnostic を生成する。
- 2026-05-13: `ISS-20260512T193917855Z-TRAIT-METHOD-RESOLUTION-STILL-CARRIE-0BFEEFA9` を追加し、`TraitMethodCall` と `TraitMethodResolution::UnsatisfiedBound` の payload も `TraitApplication` へ移行した。trait method resolution の途中で表示名を作る `infer_trait_application_name` は削除し、diagnostic 境界だけで `display_name` を呼ぶ。
- 2026-05-12: `ISS-20260512T163228542Z-PENDING-TRAIT-CHECKS-STILL-USE-POSIT-FB7F1082` を追加し、verified にした。
- 2026-05-12: `pending_trait_bound_checks: Vec<(TraitBound, TypeId, Span)>` を `Vec<PendingTraitCheck>` へ移行し、pending check の `bound` / `target_ty` / `span` を名前付き field として保持するようにした。

### Stage 5: monomorphize lookup key typed 化

目的: typecheck と monomorphize が同じ trait application identity を共有する。

作業:

- `TraitImplEntry` と `TraitImplResolution` を typed `TraitApplication` / `MonoTraitLookupKey` へ移行する。
- string-keyed `impl_map` / `impl_method_index` / `impl_entry_index` / `trait_lookup_cache` を typed key へ置換する。
- unresolved trait call は引き続き `MonomorphizeResult` で構造化して返す。

検証:

- `monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking`
- generic impl trait argument resolution。
- generated Drop trait call monomorphize。
- Resource IR drop elaboration origin preservation。

進捗:

- 2026-05-12: `ISS-20260512T164311083Z-MONOMORPHIZE-TRAIT-LOOKUP-KEYS-STILL-DA66AC14` を追加し、verified にした。
- 2026-05-12: `monomorphize.rs` の `impl_map` / `impl_method_index` / `trait_lookup_cache` を positional tuple key から `MonoTraitApplication` / `MonoTraitMethodKey` / `MonoTraitLookupKey` へ移行した。
- 2026-05-12: 旧 `impl_entry_index` は `impl_method_index` と同じ candidate set を二重に保持していたため削除し、trait base name + method の候補 index に統合した。
- 2026-05-12: `ISS-20260512T175900768Z-MONOMORPHIZE-TRAIT-LOOKUP-MODEL-EXCE-55B52B7E` を追加し、`MonoTraitApplication` / `MonoTraitMethodKey` / `MonoTraitLookupKey` / `TraitImplEntry` / `TraitImplResolution` を `monomorphize/trait_lookup.rs` へ分離した。typed identity model は維持しつつ、root `monomorphize.rs` の責務集中を戻した。
- 2026-05-12: `ISS-20260512T165852784Z-HIR-TRAIT-CALLS-STILL-SPLIT-TRAIT-AP-405A462B` を追加し、verified にした。
- 2026-05-12: `HirTraitApplication` を追加し、`FuncRef::Trait` と `HirImpl` の trait identity を split string fields ではなく typed application で保持する形へ移行した。
- 2026-05-12: `ISS-20260512T171317751Z-RESOURCE-IR-TRAIT-CALL-TARGET-STILL--6B70AE36` を追加し、verified にした。
- 2026-05-12: `ResourceTraitApplication` を追加し、`ResourceCallTarget::Trait` も split fields ではなく typed application を保持する形へ移行した。
- 2026-05-13: `ISS-20260512T190305376Z-HIR-AND-RESOURCE-TRAIT-APPLICATIONS--0B41B202` を追加し、HIR / Resource IR の trait application identity を `HirTraitId` / `ResourceTraitId` newtype へ移行した。monomorphize と Resource lowering は `as_str()` 境界でのみ文字列化する。
- 2026-05-13: `ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB` を verified にした。Stage 5 の trait/generic regression は、`PureCallsImpure` を弱めず、raw memory を直接呼ぶ generic helper を impure signature に直すことで復旧した。
- 2026-05-13: `ISS-20260512T182144401Z-MONOMORPHIZE-TRAIT-LOOKUP-METHOD-IDE-99EBBCAC` を追加し、`MonoTraitMethodId` newtype を導入した。`MonoTraitMethodKey` / `MonoTraitLookupKey` は method identity を raw `String` ではなく typed id として保持する。
- 2026-05-13: `ISS-20260512T183111826Z-MONOMORPHIZE-TRAIT-APPLICATION-STILL-835C27CF` を追加し、monomorphize trait identity も `MonoTraitId` newtype へ移行した。`MonoTraitApplication` / `MonoTraitMethodKey` は trait identity を raw `String` ではなく typed id として保持し、identity type は `monomorphize/trait_identity.rs` へ分離した。
- 2026-05-13: `ISS-20260512T191325765Z-HIR-AND-RESOURCE-TRAIT-METHOD-IDENTI-78952D7B` を追加し、HIR / Resource IR の trait method identity も `HirTraitMethodId` / `ResourceTraitMethodId` newtype へ移行した。`FuncRef::Trait` / `ResourceCallTarget::Trait` は raw `String` method field を持たず、文字列化は lowering / dump / diagnostic 境界の `as_str()` に限定する。Resource IR 側の trait identity type は `resource/trait_identity.rs` へ分離し、`resource/model.rs` の責務上限も維持する。
- 2026-05-13: `ISS-20260512T194845369Z-IMPL-METHOD-LOWERING-STILL-KEEPS-REN-E15DE8F5` を追加し、impl method lowering pass も `TraitApplication` payload を持つ形へ移行した。method symbol mangle の境界だけで `display_name` を作り、final `HirImpl` は typed application から HIR application へ変換する。
- 2026-05-13: `ISS-20260512T195620903Z-MONOMORPHIZE-TRAIT-RESOLUTION-STILL--0481CA39` を追加し、monomorphize trait resolver の入口も `HirTraitApplication` / `HirTraitMethodId` を直接受け取る形へ移行した。call site で trait name / method string へ分解する旧 `resolve_trait_impl_name` は削除した。
- 残件: Stage 5 の typed trait / method identity は typecheck / HIR / monomorphize / Resource IR call target まで到達した。次は policy baseline を整理し、BoundEnv / TypeParamId 残件と Resource IR 側の安全検査 issue に戻る。

### Stage 6: policy baseline を 0 に下げる

目的: Stage 0 の凍結線を最終設計へ移す。

作業:

- `parse_trait_ref_name` baseline は 0 済みであり、再導入禁止を維持する。
- `TraitBoundRef` 旧 model baseline は 0 済みであり、再導入禁止を維持する。
- `ImplInfo` optional field baseline は 2026-05-12 に typecheck-side `ImplKind` 導入で 0 済みであり、source policy は `ImplInfo` struct body だけを検査する。`TraitInfo.doc` の `Option<String>` は documentation data であり、impl identity policy の baseline には数えない。
- source policy を「増加禁止」から「再導入禁止」へ変更する。

進捗:

- 2026-05-12: `ISS-20260512T174845757Z-ABSTRACTION-POLICY-COUNTS-TRAITINFO--FE3DB746` を追加し、`ImplInfo` optional field の policy を `traits.rs` 全体の `Option<String>` 数ではなく `ImplInfo` struct body 直接検査へ変更した。これにより `ImplInfo` は optional field 0 件を baseline ではなく完了条件として監視する。
- 2026-05-13: `ISS-20260512T193917855Z-TRAIT-METHOD-RESOLUTION-STILL-CARRIE-0BFEEFA9` により、trait method resolution result の split `trait_name` / `trait_args` / `applied_trait_name` payload 再導入禁止を source policy に追加した。`format_trait_ref_name` baseline は 6 から 4 に下げた。
- 2026-05-13: `ISS-20260512T194845369Z-IMPL-METHOD-LOWERING-STILL-KEEPS-REN-E15DE8F5` により、impl lowering pass の direct `format_trait_ref_name` と `applied_trait_name` payload 再導入禁止を source policy に追加した。`format_trait_ref_name` baseline は 4 から 2 に下げた。
- 2026-05-13: `ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429` の最終確認で、source policy を baseline 増加抑止から final contract 検査へ変更した。`format_trait_ref_name` は `traits.rs` の display boundary に限定し、`trait_lookup_cache` は出現数ではなく `BTreeMap<MonoTraitLookupKey, Option<TraitImplResolution>>` という typed key model を直接検査する。

## 進捗状況

- `typecheck/traits.rs`: Stage 1/2 は進行済み。TraitCapability enum、TraitId、TraitApplication、TraitBound、ImplKind が存在する。function-level deferred check の string parsing と `TraitBoundRef.name` は除去済みで、type parameter bound は `TraitApplication` に集約済み。ImplInfo は `ImplKind` を持ち、optional string model ではない。
- `typecheck/trait_check.rs`: Stage 3/4 実装済み。trait application parse 依存、split impl field 参照、duplicate label fallback 実装は削除済み。
- `typecheck/trait_bound_apply.rs`: Stage 4 実装済み。pending check は `PendingTraitCheck` named model を使い、tuple state には戻さない。
- `typecheck/trait_call_apply.rs`: Stage 4/5 は進行済み。trait method resolution は `TraitMethodResolution` enum を match し、resolution payload は `TraitApplication` と `HirTraitMethodId` を保持する。診断表示名は failure diagnostic を作る境界だけで生成する。
- `typecheck/driver.rs`: Stage 5 は進行済み。impl collection と impl method lowering は `TraitApplication` を保持し、表示名は method symbol mangle の境界でのみ生成する。
- `hir.rs`: Stage 5 は進行済み。`HirTraitId` / `HirTraitApplication` / `HirTraitMethodId` を追加し、`FuncRef::Trait` / `HirImpl` は typed trait application と typed method identity を保持する。
- `monomorphize.rs`: Stage 5 実装済み。trait lookup cache / impl indexes は `MonoTraitApplication` / `MonoTraitMethodKey` / `MonoTraitLookupKey` へ移行済み。Resolver 入口は `HirTraitApplication` / `HirTraitMethodId` を直接受け取り、monomorphize phase 内の `MonoTraitId` / `MonoTraitMethodId` を含む resolved key へ変換する。
- `resource/model.rs`: Stage 5 は実装済み。`ResourceTraitId` / `ResourceTraitApplication` / `ResourceTraitMethodId` により Resource IR call target の trait identity と method identity を typed field として保持する。
- `resource/trait_identity.rs`: Stage 5 は実装済み。Resource IR の trait / method identity newtype と application payload をここに集約し、`resource/model.rs` の call target model から identity helper 実装を分離している。
- `monomorphize/trait_identity.rs`: Stage 5 は実装済み。monomorphize 内部の trait / method identity newtype をここに集約し、lookup key module から raw string identity field を排除している。
- `nodesrc/test_abstraction_static_verification_policy.js`: Stage 6 final contract policy として更新済み。typed model / enum model / named state / typed lookup key の存在と旧 string authority model の再導入禁止を直接検査する。
