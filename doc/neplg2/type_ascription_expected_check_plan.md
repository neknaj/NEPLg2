# NEPLg2 型注釈 expected-check 設計計画

作成日: 2026-05-20

関連 issue:

- [ISS-20260520T052151589Z-TYPE-ASCRIPTION-DOES-NOT-CONSISTENTL-BFF974A9](../../issues/items/ISS-20260520T052151589Z-TYPE-ASCRIPTION-DOES-NOT-CONSISTENTL-BFF974A9.md)
- [NEPLg2 abstraction static verification plan](./abstraction_static_verification_plan.md)
- [NEPLg2 静的検査の複雑化解消計画](./static_check_complexity_reduction_plan.md)

## 目的

NEPLg2 は任意の式に `<T>` 形式の型注釈を前置できる。注釈がない式では従来通り型推論を行う一方、型注釈がある式では「候補を広く推論して最後に unify する」のではなく、「期待型 `T` を制約として使い、必要十分な探索で式を検査する」経路へ入れるべきである。

この文書は、型注釈を compile-time performance の小手先の最適化ではなく、型検査の責務分割として再設計するための計画である。

## 設計制約

- 注釈がない式の型推論機能は維持する。
- 注釈がある式では、注釈を typed expected-type evidence として扱う。
- 期待型によって overload / generics / trait method の候補を早期に絞る。
- 注釈があっても generic type argument、trait application、receiver type、overload argument が不足して一意に決まらない場合は、正確性に必要な探索を行う。
- 探索は無制限な総当たりではなく、型制約から導かれる候補集合と obligation を明示し、候補が残る理由を診断できる構造にする。
- stdlib 関数名や module 名の allowlist によって「この関数ならこの性質を認める」という実装にしない。
- 診断 ID や検査分岐は文字列ではなく enum / typed struct で表す。
- `match` の網羅性検査が効く設計にし、検査プログラム自体の誤りを発見しやすくする。
- 暫定実装を入れる場合でも、最終設計と矛盾する暫定設計にはしない。

## 現状調査

### 構文と AST

`nepl-core/src/parser.rs` は `<...>` を `PrefixItem::TypeAnnotation` として AST に積む。`nepl-core/src/ast.rs` も `PrefixItem::TypeAnnotation(TypeExpr, Span)` を持つため、任意式の前置型注釈という言語仕様の入口は存在している。

### typecheck

`nepl-core/src/typecheck/prefix_check.rs` は `pending_ascription: Option<(TypeId, usize)>` を持ち、`TypeAnnotation` を読んだ地点の stack depth と target type を記録する。式が完了した時点で `apply_ascription` を呼び、完成した式の型と target type を照合している。

また、call reduction は `expected_ret` を `apply_function`、`select_overload_candidate`、trait call 解決へ渡しており、一部では期待戻り値により overload 候補を絞れている。

### 既存 regression

`tests/compiler/typeannot.n.md` は literal、block、if、while、pipe、generic call、function literal などへの前置型注釈を確認している。`tests/compiler/overload.n.md` は explicit result ascription による overload 選択と、期待型がない場合の ambiguity を確認している。

### 不足

現状は `pending_ascription` と `expected_ret` が複数箇所に分散しており、「この式は推論中なのか、期待型に対する検査中なのか」が型で表現されていない。結果として、注釈が十分な場合でも、候補探索や型変数 fresh 化を先に行い、最後に annotation mismatch として落とす経路が残りやすい。

## 目標設計

### check / infer の明示分離

式検査の入口を概念上次の 2 つへ分ける。

```rust
enum ExprCheckMode {
    Infer,
    Check(TypeExpectation),
}
```

`Infer` は従来通り、式から型を推論する。`Check` は期待型を先に持ち、式がその型を持つことを検査する。実装上すぐに public API をこの形へ全置換できない場合でも、内部状態はこの区別を失わないようにする。

### TypeExpectation

期待型は裸の `TypeId` ではなく、由来と適用範囲を持つ typed evidence にする。

```rust
struct TypeExpectation {
    ty: TypeId,
    span: Span,
    source: TypeExpectationSource,
    base_depth: usize,
}

enum TypeExpectationSource {
    ExplicitAscription,
    OuterConsumerArgument,
    CallResult,
    LetBindingAnnotation,
}
```

`TypeExpectationSource` は診断と debugging のためだけでなく、pipe や assignment marker 周辺で「どの式にかかる期待型か」を取り違えないための責務境界になる。

### 期待型制約と探索量の定義

型注釈は探索を全面禁止する印ではない。型注釈は expected type constraint であり、探索空間を制限する根拠である。

「十分な注釈」とは、少なくとも次のいずれかを満たし、追加探索なしに一意の解を得られる状態である。

- 式全体の期待戻り値が `Check(TypeExpectation)` として与えられている。
- callee の explicit type args と引数型と期待戻り値から、type parameter substitution が一意に決まる。
- overload 候補のうち、引数型と期待戻り値に一致する候補が 1 件に定まる。
- trait method について、receiver / explicit trait args / expected result から trait application と self type が一意に定まる。

一方、注釈があっても次の情報が不足している場合は、必要十分な探索を行う。

- generic function の type parameter が戻り値だけでは決まらず、引数型または trait bound から解く必要がある。
- trait method の receiver が後続引数または explicit type args からしか決まらない。
- overload 候補が expected result で絞られても複数残り、引数型や effect context を使ってさらに絞る必要がある。
- literal や constructor が expected type だけでは具体型を確定できず、周辺の consumer constraint を待つ必要がある。

この探索は、候補集合を typed value として保持する constraint solving として行う。候補を展開した理由、候補を捨てた理由、最後に曖昧性が残った理由を `enum` / `struct` で表せる設計にする。候補を組み合わせ爆発させてから後段で annotation mismatch に潰す経路は避ける。

## 実装計画

### Stage A: 設計凍結と issue 連携

目的: 型注釈最適化を局所的な高速化 patch にしない。

作業:

- この文書を追加する。
- issue に現状調査、設計制約、完了条件を記録する。
- `abstraction_static_verification_plan.md` と静的検査大規模修正計画から参照できるようにする。

完了条件:

- 実装前に `infer` / `check` の責務分割が文書化されている。
- expected type を文字列 allowlist や stdlib 個別認可として扱わないことが明記されている。

### Stage B: TypeExpectation model

目的: `pending_ascription: Option<(TypeId, usize)>` の意味を typed model へ移す。

作業:

- `TypeExpectation` / `TypeExpectationSource` を typecheck module に追加する。
- `pending_ascription`、`expected_last_ty`、`expected_ret` の受け渡しを段階的に `TypeExpectation` へ寄せる。
- pipe / assignment / block で期待型が別の式へ漏れないことを regression で固定する。

完了条件:

- 期待型の由来と適用範囲が tuple ではなく named typed state で表される。
- 既存の type annotation regression が通る。

### Stage C: overload expected-check pruning

目的: 期待戻り値がある overload call で、不要な候補探索を先に落としつつ、期待型だけでは不足する場合の必要探索を保つ。

作業:

- `select_overload_candidate` の引数を `Option<TypeExpectation>` へ寄せる。
- 引数個数、明示 type args、期待戻り値の形で候補を分類してから、必要な候補だけを instantiate / unify する。
- expected result で一意化できない候補は、argument constraints、effect context、specificity の順に typed rule として処理する。
- 候補が複数残る場合は specificity に逃げる前に、なぜ一意でないかを診断できる構造にする。

完了条件:

- explicit result ascription が overload 選択の主要根拠として使われる。
- no-context overload ambiguity は維持される。
- annotate すれば通る、annotate しなければ ambiguous、という対の regression を追加する。

実装状況:

- 2026-05-20: `select_overload_candidate`、`apply_function`、selected call、indirect call、trait call の expected result 境界を `Option<TypeExpectation>` に移した。call reduction で得た期待型の由来を call 適用層まで落とさずに保持できる。
- 2026-05-20: overload selection は binding の declared function shape を先に読み、関数でない候補、明示 type args 数不一致、capture 数不整合、arity 不一致を checkpoint / instantiate 前に除外するようにした。
- 2026-05-20: 明示 type args がない multiple overload では、declared result と expected target を `type_pattern_matches` で照合し、明らかに結果型が合わない候補を checkpoint / instantiate 前に除外するようにした。
- 2026-05-20: overload 候補 rejection reason を `OverloadCandidateRejection` enum として構造化し、candidate materialization count と pre-materialized rejection count の不変条件を `OverloadCandidateStats` で監視するようにした。
- 2026-05-20: overload ambiguity は `OverloadCandidateNarrowingStage` と `OverloadAmbiguityReason` payload から診断 message を生成し、どの narrowing stage の後に候補が残ったかを保持するようにした。
- 未完了: generic function 側の不足 constraint object 化。現時点では typed evidence の伝播、result constraint の早期 commit、declared-shape pruning、expected-result shape pruning、candidate count guard、ambiguity reason payload を固定している。

### Stage D: generics / trait expected-check

目的: generics と trait method でも期待型を探索削減と正確性の両方に使い、注釈不足時の必要探索を明示的に扱う。

作業:

- generic function の type parameter substitution を、引数型だけでなく期待戻り値からも解く。
- trait method resolution で expected result を typed `TypeExpectation` として扱い、trait application inference の根拠を明示する。
- 解けない場合は fallback 推論を無制限に広げず、未解決の type parameter / trait application / receiver constraint を保持して、解決可能な追加制約が来る範囲だけ探索を続ける。
- 制約が閉じても一意に決まらない場合は、どの type parameter または trait application が不足しているかを診断する。

完了条件:

- 型注釈付き generic / trait call は必要十分な候補検査で決まる。
- annotation mismatch と overload ambiguity と trait bound unsatisfied が混線しない。

実装状況:

- 2026-05-20: selected generic call では call result と expected result を type args 確定前に unify するようにした。これにより `<T>` による結果期待型が generic instantiation の根拠として残る。
- 2026-05-20: trait method resolution は expected result を `TypeExpectation` として受け取り、trait application inference には target type を渡す。診断 span は `TypeExpectation` が管理する。
- 2026-05-20: trait application inference は `TypeParamInferenceSource` / `TypeParamInferenceConstraint` により、argument 由来制約と expected result 由来制約を typed object として保持するようにした。`infer_trait_application_args` は `Option<TypeExpectation>` を直接受け、呼び出し側で target `TypeId` へ落とさない。
- 2026-05-20: selected generic function call も `GenericCallConstraintSource` / `GenericCallConstraint` で argument 由来制約と expected result 由来制約を typed object として保持するようにした。期待戻り値制約は引数制約より先に適用するため、`id<T>(T)->T` のような関数で `<u8>` の期待型が `T` を拘束し、その後の char literal 引数検査にも context が伝わる。
- 未完了: constraint が閉じても一意に決まらない場合の診断 payload 化。

### Stage E: performance guard

目的: コンパイル時間改善を時間計測だけに依存させない。

作業:

- source policy または focused test で、期待型がある経路から expected type を捨てる再帰探索が増えないことを監視する。
- 必要なら verbose log とは別に test-only counter を設け、対象 regression の候補検査数を固定する。
- full CI では wall-clock を見るが、local regression は候補数や typed path の有無を主に検査する。

実装状況:

- 2026-05-20: `OverloadCandidateStats` が considered / materialized / accepted / rejection reason count を保持し、`debug_assert!` で `materialized + pre_materialized_rejections <= considered` を検査する。source policy は rejection reason enum、exhaustive match、materialization guard、materialization count の位置を監視する。

完了条件:

- 既存の注釈 regression が維持される。
- 注釈付き overload/generic regression の候補探索が増えた場合に focused test で検出できる。

## 非目標

- 型注釈なしの式から推論機能を削除しない。
- stdlib の特定関数名を列挙して高速化しない。
- annotation mismatch を後段の文字列診断で取り繕わない。
- 一時的な compile-time shortcut を final design として残さない。

## セルフホストへの反映

self-host compiler では、Rust 実装の flat な `pending_ascription` tuple をそのまま移植しない。`neplg2/core/check` または `neplg2/core/ty` 側で `TypeExpectation` 相当の typed model を先に用意し、式 checking と inference を別 entry として設計する。

この方針はセルフホスト版の directory/file 分割にも影響する。`check/expr`、`check/expectation`、`check/overload`、`check/trait` のように、期待型の伝搬と候補選択を同じ巨大 file に閉じ込めない。
