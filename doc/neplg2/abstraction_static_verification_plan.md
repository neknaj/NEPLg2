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

### 不十分な点

現行実装は基本機能を持つが、設計としては十分ではない。

1. trait application identity がまだ一部文字列に寄っている。
   - `TraitBoundRef.name` は `format_trait_ref_name` で作った表示名を保持する。
   - 2026-05-12 時点で、表示文字列から trait argument を復元する `parse_trait_ref_name` は削除済みである。
   - ただし `TraitBoundRef.name` 自体は残っているため、diagnostic/display 用の文字列と compiler identity を構造上分離しきれていない。

2. impl model が optional string field に寄っている。
   - `ImplInfo` は `trait_name: Option<String>`、`trait_base_name: Option<String>`、`trait_args: Vec<TypeId>`、`trait_self_ty: Option<TypeId>` を持つ。
   - inherent impl、trait impl、trait application の有無が enum で分かれず、field combination の妥当性を compiler が保証しにくい。

3. type parameter bound identity が `TypeId` と label fallback に依存している。
   - `type_param_bounds: BTreeMap<TypeId, Vec<TraitBoundRef>>` は resolve 後 ID や label 同一性の fallback を必要としている。
   - これは type parameter の stable identity が境界として弱いことを示す。

4. pending trait check が tuple で保持されている。
   - `pending_trait_bound_checks: Vec<(TraitBoundRef, TypeId, Span)>` は field の意味が型名だけでは読み取りにくい。
   - 診断や deferred check の分類を enum / named struct にすべきである。

5. monomorphize trait lookup が string-keyed map へ寄っている。
   - `impl_map`、`impl_method_index`、`impl_entry_index`、`trait_lookup_cache` は trait name / method name の `String` を key にする。
   - display name と compiler identity が同じ層に混ざり、typecheck と monomorphize の agreement を型で保証できない。

6. source policy は abstraction contract を直接監視していなかった。
   - typecheck file split と monomorphize panic policy はあるが、trait reference string authority の拡大を止める専用 policy はなかった。

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

- `nodesrc/test_abstraction_static_verification_policy.js` を追加し、`format_trait_ref_name`、`TraitBoundRef`、`ImplInfo`、`trait_lookup_cache` の current baseline を固定する。
- `parse_trait_ref_name` は Stage 1 の作業で 0 baseline になっており、再導入禁止にした。
- baseline は最終状態ではない。今後の stage で残りの表示名 field / string key の数を下げる。

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

### Stage 6: policy baseline を 0 に下げる

目的: Stage 0 の凍結線を最終設計へ移す。

作業:

- `parse_trait_ref_name` baseline は 0 済みであり、再導入禁止を維持する。
- `TraitBoundRef` 旧 model baseline を 0 にする。
- `ImplInfo` optional string baseline を 0 にする。
- source policy を「増加禁止」から「再導入禁止」へ変更する。

## 進捗状況

- `typecheck/traits.rs`: 実装済みだが再設計対象。TraitCapability enum は良い。function-level deferred check の string parsing は除去済みだが、TraitBoundRef と一部 string formatting/parsing は残る。
- `typecheck/trait_check.rs`: 実装済みだが再設計対象。trait application parse 依存は削除済みだが、bound satisfaction の label fallback は残る。
- `typecheck/trait_bound_apply.rs`: 実装済みだが再設計対象。pending check と substituted bound を named typed model へ移す必要がある。
- `typecheck/trait_call_apply.rs`: 実装済みだが再設計対象。trait method resolution result enum が必要。
- `monomorphize.rs`: 実装済みだが再設計対象。trait lookup cache / impl indexes を typed key にする必要がある。
- `nodesrc/test_abstraction_static_verification_policy.js`: Stage 0 baseline policy として追加。
