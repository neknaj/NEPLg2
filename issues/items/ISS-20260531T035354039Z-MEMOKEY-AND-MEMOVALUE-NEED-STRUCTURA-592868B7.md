---
id: ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7
title: "MemoKey and MemoValue need structural purity rules"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-12
target: "nepl-core/src/types.rs; nepl-core/src/typecheck; stdlib/std; stdlib/neplg2/core/ty"
---

# ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7: MemoKey and MemoValue need structural purity rules

## 概要

Phase 1 memo_call can use Copy-like key/value types, but TypeCtx::is_copy alone is too broad because function values, references, raw pointers, owner tokens, public mutable state, and external handles must not become MemoKey or MemoValue.

## 対象

- `nepl-core/src/types.rs; nepl-core/src/typecheck; stdlib/std`

## 根拠

- `doc/neplg2/private_effect_memoization_purity_design.md` の Phase 1 方針に従う。
- `memo_call` は private cache backend が入るまでは `MemoKey&Copy` / `MemoValue&Copy` かつ Drop なしの構造値だけを受け入れ、identity や resource lifecycle が観測可能な値を拒否する必要がある。
- ordinary `Copy` は低レベル境界の軽量 handle にも付与されるため、`MemoKey` / `MemoValue` trait と Phase 1 structural predicate は function value、reference、raw memory view、owner token を追加で拒否する。

## 問題

Phase 1 memo_call can use Copy-like key/value types, but TypeCtx::is_copy alone is too broad because function values, references, raw pointers, owner tokens, public mutable state, and external handles must not become MemoKey or MemoValue.

## 影響

If MemoKey or MemoValue is treated as a simple Copy alias, memo_call can cache values whose identity or behavior is externally observable, breaking the Pure contract.

## 修正方針

Define structural MemoKey and MemoValue rules that require pure Eq/Hash/Clone/Drop where applicable and explicitly reject function values, references, raw pointers, owner tokens, public mutable state, external resources, unknown effect values, and non-Copy/Drop values in Phase 1.

## 2026-05-31 checkpoint

- `stdlib/core/traits/memo.nepl` に `MemoKey` / `MemoValue` trait を追加し、`memo_call` の public signature を `.K: MemoKey&Copy, .V: MemoValue&Copy` にした。
- `memo_call` Phase 1 predicate は `ctx.is_copy`、`ctx.has_drop`、compiler memory type check、`MemoKey` / `MemoValue` trait bound を組み合わせる。key 側は `unit`、`i32`、`u8`、`bool`、`char` と、それらだけで構成される recursive structural Copy aggregate を受け入れる。value 側は同じ範囲に加えて `f32` も受け入れる。
- compiler-known primitive gate が参照する `MemoKey` / `MemoValue` trait definition は `stdlib/core/traits/memo.nepl` の source identity を確認する。
- accepted regression として、user-defined `Pair` に `MemoKey` / `MemoValue` / `Clone` / `Copy` impl があり、field が `i32` だけで構成される場合に `memo_call @same_pair` が通ることを確認した。
- rejected regression として、Copy だが `MemoKey` がない struct、Copy かつ `MemoKey` だが `MemoValue` がない struct、non-Copy struct、`str`、`f32` key、`f32` field を持つ structural key、function value、reference、`MemPtr i32`、`RegionToken i32`、unresolved generic function value を追加した。
- `unit` keyword が trait impl method signature の一部経路で fresh type variable になっていたため、type expression lowering で intrinsic type name として扱うようにした。これにより `MemoKey for unit` / `MemoValue for unit` の標準 impl が signature mismatch なしで検査される。
- unit key/value の regression を追加し、`%fn (unit) i32` の grouped unary unit argument と `%fn unit i32` の zero-argument function marker を区別して固定した。
- この checkpoint では private cache backend がまだないため、cache algorithm correctness、official external handle marker、`MemoKey` / `MemoValue` impl の semantic validation は継続して設計する。

## 検証

Accepted tests should cover primitive scalar/unit/structural Copy values; rejected tests should cover function keys, impure Eq/Hash/Clone/Drop, references, raw pointers, owner tokens, mutable/public state, external handles, and non-Copy values.

Current Phase 1 regression is covered by `cargo test -p nepl-core function_memo_call --test functions -- --nocapture`.

## 2026-06-12 selfhost predicate checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait.nepl` を追加し、selfhost compiler 側でも `MemoKey` / `MemoValue` の Phase 1 predicate を持つようにした。主 API は `Result unit SelfhostMemoTraitRejectKind` であり、`bool` helper はこの typed result から派生する補助に留めた。

現 selfhost predicate は `unit`、`bool`、`i32`、`u8`、`char` を `MemoKey` と `MemoValue` の両方で受理し、`f32` は `MemoValue` だけで受理する。`f32` key、`I64`、`F64`、`str`、`never`、`error`、function type、missing TypeId、generic parameter は enum reason 付きで拒否する。

Rust 実装の Phase 1 は structural Copy aggregate acceptance まで持つが、selfhost の現行 `SelfhostTypeArena` は named / applied type の field layout、trait impl evidence、Drop / Copy proof を持たない。そのため、この checkpoint では `NamedLayoutUnknown` / `AppliedLayoutUnknown` として fail-closed にする。aggregate acceptance は、type constructor layout evidence と trait solver が入った後にこの issue の後続 slice として接続する。

## 2026-06-12 selfhost aggregate evidence consumer checkpoint

`SelfhostMemoTraitEvidenceTable` と `selfhost_memo_key_type_result_with_evidence` / `selfhost_memo_value_type_result_with_evidence` を追加した。これは structural aggregate acceptance の solver ではなく、後続の layout / trait solver が作る `Result unit SelfhostMemoTraitRejectKind` payload を `Named` / `Applied` predicate が消費するための境界である。

証拠付き入口でも primitive、function、generic parameter、missing type record は証拠で上書きできない。`Named` / `Applied` は evidence record がある場合だけその Result payload を返し、証拠が無い場合は従来どおり `NamedLayoutUnknown` / `AppliedLayoutUnknown` に fail-closed する。table は session-local `SelfhostTypeId` を使うため、永続 artifact では canonical type key と solver policy hash が別途必要である。

runtime smoke では、Named と Applied の no-evidence reject / accepted evidence / rejected evidence を分けて確認し、さらに primitive `f32` key と missing `TypeId` が fake evidence で受理されないことも固定した。stage0 helper は Named 系と Applied 系に分割し、単一の巨大な prefix expression が selfhost compiler の探索範囲を増やさない形にした。

残件は、type constructor layout evidence、MemoKey / MemoValue trait source identity、Copy / Drop / Eq / Hash の pure evidence、recursive aggregate / cycle boundary、canonical type key indexed solver output を実装し、この evidence table の producer として接続することである。

## 2026-06-12 selfhost aggregate evidence producer gate checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl` を追加し、後続 solver が作る trusted aggregate proof summary を `SelfhostMemoTraitEvidenceRecord` へ変換する producer gate を分離した。

`SelfhostMemoTraitAggregateProof` は `type_id`、field layout summary、Copy / Drop / Eq / Hash proof status、cache escape hazard classification、`key_result`、`value_result` を持つ。`key_result` / `value_result` は `Result unit SelfhostMemoTraitRejectKind` のまま保持するため、key と value で異なる拒否理由を持つ aggregate proof を bool に潰さず consumer table へ渡せる。

`selfhost_memo_trait_aggregate_proof_to_record` は `Named` / `Applied` record だけを accepted record の候補とし、primitive、function、generic parameter、missing type record は `SelfhostMemoTraitEvidenceProduceRejectKind` で拒否する。さらに `Named` / `Applied` であっても、field layout missing、invalid field range、generic argument unsubstituted、cycle limit reached、operation proof missing / impure / unknown、cache reference escape、external handle、owner token、public mutable state、unknown hazard は producer 側で typed reject になり、consumer record へ進まない。

この checkpoint は field layout solver や trait solver そのものではない。`SelfhostTypeId` は session-local であり、永続 artifact では canonical type key と solver policy hash で索引した proof store から現在の arena へ投影する必要がある。残件は、type constructor layout evidence、MemoKey / MemoValue trait source identity、Copy / Drop / Eq / Hash の pure evidence、recursive aggregate / cycle boundary の実計算、canonical type key indexed proof store を実装し、この producer gate の入力側へ接続することである。

## 2026-06-12 selfhost canonical proof store checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl` を追加し、`SelfhostTypeId` を永続 proof key にしない proof store 境界を作った。

store は `SelfhostCanonicalTypeKeyArena` と `SelfhostMemoTraitProofStoreRecord` を所有する。record は canonical root key、`SelfhostMemoTraitProofStorePolicy`、proof kind、TypeId を含まない `SelfhostMemoTraitStoredAggregateProof` だけを保持する。lookup は現在の `SelfhostTypeArena` の TypeId を一時 canonical key arena へ再投影し、cross-arena canonical equality と policy identity が一致した場合だけ stored proof を現在 TypeId の `SelfhostMemoTraitAggregateProof` へ戻す。canonical key が一致しても policy が違う record は stale proof として記録し、後続に期待 policy の record がないか探索を続ける。最後まで期待 policy が見つからず stale proof だけが存在した場合に `PolicyMismatch` を返す。その後、既存の `selfhost_memo_trait_aggregate_proof_to_record` を必ず通すため、primitive / function / parameter / missing record は store 上の fake proof でも accepted record にならない。

`SelfhostMemoTraitProofStoreLookupErrorKind` は projection failure、missing proof、policy mismatch、proof kind mismatch、producer rejection を enum として返す。`ProducerRejected` は外側 variant だけでなく `SelfhostMemoTraitEvidenceProduceRejectKind` payload まで比較し、primitive fake proof と missing layout などを同一視しない。stage0 smoke では accepted lookup、stale policy rejection、missing key rejection、primitive fake proof rejection、unsupported proof kind rejection を実行で確認した。

この checkpoint でも named type の canonical key はまだ `SelfhostNamedTypeId` であり、module path / public surface hash / stable constructor identity を含む永続 nominal key ではない。残件は、type constructor layout evidence、MemoKey / MemoValue trait source identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary の実計算、stable nominal key / serialized canonical key fingerprint を proof store の入力へ接続することである。

## 2026-06-12 selfhost typed proof store policy identity checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_policy.nepl` を追加し、proof store policy identity を source identity と rule identity の typed payload に分離した。

`SelfhostMemoTraitSourceKind` は `MemoKeyTrait` と `MemoValueTrait` を enum として分け、`SelfhostMemoTraitSourceIdentity` は module / symbol / signature fingerprint を source identity として意味づける。`SelfhostMemoTraitSourceIdentitySet` は MemoKey と MemoValue の両方を保持するため、片方の trusted source だけが一致した proof を再利用しない。`SelfhostMemoTraitRuleIdentity` は store schema、solver version、primitive rule、aggregate rule、hazard rule を別 field として持つ。

`SelfhostMemoTraitProofStorePolicy` は `sources %SelfhostMemoTraitSourceIdentitySet` と `rules %SelfhostMemoTraitRuleIdentity` の組になり、`memo_trait_proof_store.nepl` から raw `trait_source_hash` / `rule_hash` field と raw-i32 policy constructor を削除した。policy equality は source kind、module hash、symbol hash、signature hash、schema version、solver version、primitive rule hash、aggregate rule hash、hazard rule hash をすべて比較する。

この checkpoint は proof store が要求する policy identity の型を固定した段階であり、trait definition table から `MemoKey` / `MemoValue` source identity を実生成する registry はまだ未接続である。残件は、type constructor layout evidence、MemoKey / MemoValue trait definition source identity の生成、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost trusted memo trait source registry checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_source.nepl` を追加し、proof store が `MemoKey` / `MemoValue` source identity を手作業で組み立てないための registry 境界を作った。

`SelfhostMemoTraitTrustedSourceRegistry` は `memo_key %SelfhostMemoTraitSourceIdentity` と `memo_value %SelfhostMemoTraitSourceIdentity` を持ち、`selfhost_memo_trait_trusted_source_registry_current_result` が current compiler の prepared source identity registry を `Result` で返す。registry constructor は private helper にしており、外部 caller が同じ型の `MemoKey` / `MemoValue` identity を入れ替えた trusted registry を作れないようにした。将来 artifact snapshot を外部入力として読む段階では、constructor を公開するのではなく、kind mismatch を `Result` で拒否する validator を追加する。`selfhost_memo_trait_trusted_source_registry_sources` は registry を borrow して `SelfhostMemoTraitSourceIdentitySet` へ投影するため、registry owner を消費しない。`selfhost_memo_trait_trusted_source_registry_is_current` は typed source identity equality で snapshot と current source set を照合し、current source construction が壊れた場合は `false` を返す。

`memo_trait_proof_store.nepl` の stage0 は `selfhost_memo_trait_trusted_source_identity_set_current_result` を使うようになり、proof store 内で `SelfhostMemoTraitSourceKind::MemoKeyTrait` / `MemoValueTrait` や `selfhost_memo_trait_source_identity_new` を直接呼ばない。source policy はこの退行を検出する。

この checkpoint は prepared fingerprint registry であり、trait definition table から source text、public surface hash、signature hash を実際に materialize する実装ではない。残件は、type constructor layout evidence、trait definition table から prepared fingerprint ではない stable source identity を生成する materializer、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait source materializer checkpoint

`memo_trait_source.nepl` に `SelfhostMemoTraitDefinitionFingerprint`、`SelfhostMemoTraitDefinitionSourceRecord`、`SelfhostMemoTraitSourceMaterializeErrorKind`、`SelfhostMemoTraitTrustedSourceRegistryErrorKind` を追加し、prepared i32 fingerprint を直接 source identity constructor へ渡すのではなく、typed definition source record から `Result SelfhostMemoTraitSourceIdentity SelfhostMemoTraitSourceMaterializeErrorKind` へ materialize する境界を作った。

materializer は expected kind と record kind が一致しない場合に `KindMismatch`、signature fingerprint が trusted source identity として未確定の場合に `SignatureMissing` を返す。current registry には `selfhost_memo_trait_trusted_source_registry_current_result` と `selfhost_memo_trait_trusted_source_identity_set_current_result` を追加し、MemoKey 側と MemoValue 側の materialization error を `MemoKeySourceRejected` / `MemoValueSourceRejected` として区別する。非 Result wrapper は撤廃し、current 判定と stage0 は Result API を match して失敗時に `false` / summary false へ落とす。

`memo_trait_proof_store.nepl` の stage0 は Result 版の current source set から policy を作るようになり、registry materialization が将来失敗した場合は owner cleanup 付き abort path へ fail-closed に進む。source policy は typed definition record、materializer Result API、kind mismatch / signature missing、current Result API、proof store の Result registry 使用、registry/proof_store での raw constructor 迂回禁止を固定する。

この checkpoint もまだ full trait definition table scanner ではない。残件は、actual trait definition table から prepared i32 ではない stable trait source record を生成する producer、public surface hash / stable trait definition key、type constructor layout evidence、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait source table checkpoint

`memo_trait_source.nepl` に `SelfhostMemoTraitDefinitionSourceTable` を追加し、current trusted source registry を fixed table から materialize する経路へ変更した。table は `MemoKey` / `MemoValue` の `Option SelfhostMemoTraitDefinitionSourceRecord` と duplicate flag を持ち、`selfhost_memo_trait_definition_source_table_add_record` が source kind によって record を分類する。

`selfhost_memo_trait_trusted_source_registry_from_definition_table` は、duplicate、missing、source materialization failure を `SelfhostMemoTraitTrustedSourceRegistryErrorKind` で区別して返す。stage0 は missing key、missing value、duplicate key、duplicate value、key source rejected、value source rejected をすべて public validator 経由で作り、手書きの `Result::Err` で検査経路を迂回しない。per-kind current source identity helper は private smoke helper に戻し、proof store policy 用 source set は table-backed `selfhost_memo_trait_trusted_source_identity_set_current_result` からだけ取得する。

この checkpoint でも current table producer は prepared i32 fingerprint 2 件の Phase 1 である。残件は、actual trait definition table scanner、public surface hash と stable trait definition key 由来の source record producer、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost type constructor layout evidence checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl` を追加し、`MemoKey` / `MemoValue` aggregate proof の field layout 入力を session-local typed evidence table として分離した。

`SelfhostMemoTraitLayoutEvidenceTable` は `layouts %Vec SelfhostMemoTraitLayoutRecord` と `fields %Vec SelfhostMemoTraitLayoutFieldRecord` を所有する。layout record は `SelfhostNamedTypeId`、constructor arity、`SelfhostMemoTraitAggregateFieldRange` を持ち、field record は owner constructor identity、field index、substitution 済み field type の `SelfhostTypeId` を持つ。source spelling、display name、diagnostic string、module path suffix は accepted path の authority にしない。

`selfhost_memo_trait_layout_evidence_for_type_result` は `Named` と `Applied` を分けて検査する。`Named` は arity 0 の constructor layout だけを受理し、type parameter を持つ constructor layout は `NamedConstructorHasTypeParameters` として拒否する。`Applied` は constructor identity と applied argument count が layout arity と一致し、型引数が未解決 parameter でないことを確認する。field range が table 外、field owner mismatch、field index mismatch、missing field type、unsubstituted generic field は `SelfhostMemoTraitLayoutEvidenceErrorKind` として fail-closed に返す。成功した場合だけ `SelfhostMemoTraitAggregateFieldEvidence::Known(range)` を返す。

この checkpoint は layout evidence の生成境界であり、aggregate acceptance の完了ではない。Copy / Drop / Eq / Hash pure evidence、cache hazard proof、recursive aggregate / cycle boundary、enum/sum layout、stable nominal key / serialized canonical key fingerprint はまだ後続 slice に残る。runtime doctest は compiler 探索空間を広げない小さな typed Result smoke にし、validator 本体の contract は `nodesrc/test_selfhost_memo_trait_layout_contract.js` で固定した。

## 2026-06-12 selfhost memo trait definition source scanner checkpoint

`stdlib/neplg2/core/check/module/memo_trait_source_scan.nepl` を追加し、`SelfhostModuleAst` の `TraitDecl` から `MemoKey` / `MemoValue` definition source table candidate を作る checker-layer scanner を実装した。

この scanner は `core/ty` へ syntax AST 依存を持ち込まず、`core/check/module` で `SelfhostModuleAst` と `SelfhostMemoTraitDefinitionSourceTable` を接続する。`TraitDecl` だけを候補にし、`FunctionDecl "MemoKey"` のような同名 non-trait item は table slot を埋めない。`TraitDecl` が declaration header / head を欠く場合や header kind が壊れている場合は、bool や文字列ではなく `SelfhostMemoTraitDefinitionScanErrorKind` で fail-visible に返す。

scanner が作る `SelfhostMemoTraitDefinitionSourceRecord` は `signature_available = false` である。source slice と `"MemoKey"` / `"MemoValue"` の文字列比較は候補分類にだけ使い、accepted source identity の authority にはしない。したがって `selfhost_memo_trait_trusted_source_registry_scan_module_result` は、両方の trait が見つかっても既存の table-backed registry validator を通して `MemoKeySourceRejected(SignatureMissing)` に fail-closed する。current trusted registry は引き続き prepared source table を使い、scanner output を current authority へ昇格しない。

source policy は `nodesrc/test_selfhost_memo_trait_source_scan_contract.js` で固定した。検査内容は、scanner が syntax AST と memo trait source table の接続層に留まること、`core/ty` が syntax/check を import しないこと、scanner が `selfhost_memo_trait_source_identity_new` を呼ばないこと、`signature_available=true` を作らないこと、proof store が scanner へ直接依存しないこと、行数制限や doc comment 長制限を追加しないことを含む。

この checkpoint の残件は、stable public surface hash、stable trait definition key、trait signature normalization、module identity を持つ stable source record producer である。その producer が入るまでは、actual scanner は table shape と fail-closed 経路を確認する前段として扱う。

## 2026-06-12 selfhost memo trait stable source evidence checkpoint

`stdlib/neplg2/core/check/module/memo_trait_source_fingerprint.nepl` を追加し、scanner candidate table と typed public surface materializer 由来の stable fingerprint evidence を突き合わせる producer gate を作った。

`SelfhostMemoTraitStableSourceFingerprintEvidence` は module identity、stable trait definition key、normalized public signature の fingerprint availability を型付き payload として持つ。`SelfhostMemoTraitStableSourceEvidenceRecord` は `MemoKey` / `MemoValue` の source kind と、その fingerprint evidence を分けて保持する。`selfhost_memo_trait_definition_source_table_from_stable_evidence_result` は scanner candidate table に `MemoKey` / `MemoValue` 候補が 1 件ずつ存在すること、candidate / evidence の duplicate flag が立っていないこと、evidence の 3 fingerprint がすべて `some` であること、さらに fingerprint が scanner placeholder と同じ `0` ではないことを確認した場合だけ、`signature_available = true` の `SelfhostMemoTraitDefinitionSourceRecord` を作る。

この producer は scanner candidate の fingerprint を trusted identity payload として使わない。candidate table は presence / duplicate evidence としてだけ扱い、accepted payload は stable evidence table から作る。最終的な registry 化も既存の `selfhost_memo_trait_trusted_source_registry_from_definition_table` を通すため、stable source producer が `selfhost_memo_trait_source_identity_new` を直接呼ぶ経路は持たない。

source policy は `nodesrc/test_selfhost_memo_trait_source_fingerprint_contract.js` で固定した。検査内容は、producer が checker-layer module であり `core/ty` へ逆依存しないこと、proof store が producer output へ直接依存しないこと、source text / display name / path suffix / diagnostic text を accepted authority にしないこと、候補欠落・候補重複・evidence 欠落・evidence 重複・fingerprint 未確定・placeholder fingerprint を typed enum error として保持すること、行数制限や doc comment 長制限を追加しないことを含む。

この checkpoint も public surface hash の計算本体ではない。残件は、actual typed public surface materializer から module identity / stable trait definition key / normalized trait signature fingerprint を生成する実装、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait stable source seed producer checkpoint

`stdlib/neplg2/core/check/module/memo_trait_source_evidence_producer.nepl` を追加し、typed module / public surface / trait-local signature seed から `SelfhostMemoTraitStableSourceEvidenceTable` を作る Phase 1 producer を実装した。

入力は `SelfhostMemoTraitStableSourceModuleSeed` と `SelfhostMemoTraitStableSourceTraitSeed` であり、module identity hash、public surface hash、trait kind、visibility、declaration ordinal、normalized signature hash を named field として保持する。raw `i32` の tuple や source spelling / span / display name / path suffix は accepted source identity の authority にしない。seed の欠落、`0` placeholder、private visibility、`MemoKey` / `MemoValue` の重複、malformed table による kind mismatch は `SelfhostMemoTraitStableSourceSeedErrorKind` で fail-closed に返す。さらに、seed 自体が nonzero でも deterministic folding の結果が `0` になった場合は、public evidence table へ出さず derived placeholder error として拒否する。

この producer は `SelfhostMemoTraitDefinitionSourceRecord(signature_available=true)` を直接作らない。成功時も stable evidence table までを返し、registry へ進む場合は既存の `selfhost_memo_trait_trusted_source_registry_from_stable_evidence_result` と `selfhost_memo_trait_trusted_source_registry_from_definition_table` を必ず通す。これにより scanner candidate、typed seed、stable fingerprint gate、trusted registry validator の責務境界を保つ。proof store は引き続き trusted source registry / source set Result API だけを使い、この producer へ直接依存しない。

source policy は `nodesrc/test_selfhost_memo_trait_source_evidence_producer_contract.js` で固定した。検査内容は、typed seed struct、seed table の欠落・重複表現、enum error、missing / placeholder fingerprint rejection、private visibility rejection、kind mismatch、既存 stable source gate 経由、`core/ty` の逆依存禁止、proof store 直結禁止、`selfhost_memo_trait_source_identity_new` と `signature_available=true` record の直接生成禁止、行数制限や doc comment 長制限の不在を含む。

この checkpoint も full public surface materializer ではない。残件は、re-export を含む actual public surface hash 生成、trait body / method signature normalization、stable trait definition key、stable nominal key / serialized canonical key fingerprint、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost public surface seed materializer checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl` を追加し、`SelfhostModuleAst` の public marker trait declaration から `SelfhostMemoTraitStableSourceSeedTable` を作る Phase 1 materializer を実装した。

この materializer は `SelfhostMemoTraitDefinitionSourceTable` scanner、public surface seed scan、stable evidence producer、fingerprint gate、trusted registry validator を順に接続する。module identity と public surface hash は caller が `SelfhostMemoTraitStableSourceModuleSeed` として渡し、module path、file path、display name、diagnostic text、source span、syntax range、source text slice から accepted fingerprint authority を作らない。source text slicing は `MemoKey` / `MemoValue` の候補分類だけに使う。

Phase 1 accepted path は public marker trait だけに限定した。private trait、duplicate、missing、malformed header、trait body / method signature normalization が必要な trait は `SelfhostMemoTraitPublicSurfaceSeedErrorKind` として fail-closed に返す。`core/ty` は checker-layer public surface seed module を import せず、proof store も seed output へ直接依存しない。source policy は raw source identity construction、`signature_available=true` record の直接生成、line count / doc comment length 制限の混入を禁止する。

この checkpoint でも実 stdlib の `MemoKey` / `MemoValue` trait body / method signature は normalized signature evidence へ未接続である。残件は、actual public surface hash / stable trait definition key producer、trait body / method signature normalization evidence、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait local public surface hash materializer checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl` を追加し、local `MemoKey` / `MemoValue` marker trait pair 用の Phase 1 public surface hash materializer を実装した。

この materializer は caller supplied module identity hash、`SelfhostModuleAst`、public surface seed materializer の seed table を受け、seed table の typed field だけを deterministic に fold して `SelfhostMemoTraitStableSourceModuleSeed.public_surface_hash` を作る。fold material は `MemoKey` / `MemoValue` の kind、visibility、declaration ordinal、normalized marker signature seed、schema domain code に限定し、source text、span、syntax range、lexeme、display name、path suffix、diagnostic text は accepted hash authority にしない。module identity hash の欠落、`0` placeholder、derived public surface hash `0` は enum error で fail-closed にする。

この checkpoint は full public surface materializer ではない。import / use / prelude / no-prelude は dependency public surface が未正規化なので `ImportSurfaceUnsupported` / `UseSurfaceUnsupported` / `PreludeSurfaceUnsupported` / `NoPreludeSurfaceUnsupported` として拒否する。public function / struct / enum / impl declaration も、この local marker trait pair hash では完全な module public surface に含められないため、それぞれ `PublicFunctionSurfaceUnsupported` / `PublicStructSurfaceUnsupported` / `PublicEnumSurfaceUnsupported` / `PublicImplSurfaceUnsupported` として拒否する。一方で、private non-trait declaration は public surface に出ないためこの slice では無視する。private `MemoKey` / `MemoValue` trait は前段 seed materializer で typed error として拒否する。

registry convenience path は candidate scanner、public surface hash materializer、stable evidence producer、stable fingerprint gate、trusted registry validator の順に通す。`SelfhostMemoTraitDefinitionSourceRecord(signature_available=true)` や `SelfhostMemoTraitSourceIdentity` はこの module で直接作らない。`core/ty` と proof store はこの checker-layer hash materializer に逆依存しない。

subagent review では Fermat が、この producer を `actual public surface hash` と呼ぶと full module surface / re-export / trait body normalization まで実装済みと誤読されること、unsupported item の error 粒度を分けること、schema domain code を hash に含めることを Required として指摘した。Goodall は Blocker なしで、source / span / path / diagnostic の hash authority 禁止、module identity placeholder rejection、import / use fail-closed、既存 gate 経由を Required とした。実装では module doc と contract に local marker trait pair 限定を明記し、unsupported item error を import / use / prelude / public declaration kind ごとに分離し、schema domain code を fold へ入れた。

source policy は `nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js` で固定した。検査内容は、facade export、目的 / 契約 / 現状 / 計算量 / doctest、typed materialization struct、typed error enum、scanner -> materializer -> seed evidence -> registry gate の順序、source / span / name / path / diagnostic の hash fold 禁止、import / use / prelude fail-closed、public non-trait declaration fail-closed、`core/ty` 逆依存禁止、proof store 直結禁止、行数制限 / doc comment 長制限禁止を含む。

この checkpoint 後も、実 stdlib の `MemoKey` / `MemoValue` trait body / method signature は normalized signature evidence へ未接続である。残件は、re-export / import graph を含む full public surface hash、stable trait definition key producer、trait body / method signature normalization evidence、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait marker signature shape checkpoint

`stdlib/neplg2/core/check/module/memo_trait_signature_shape.nepl` を追加し、`SelfhostModuleAst` から得られる trait declaration header/body evidence が Phase 1 の public marker trait signature として扱えるかを typed `Result` で判定する境界を作った。

現 selfhost AST は `SelfhostModuleDeclarationBody` に body envelope と first expression の `SelfhostSyntaxRange` だけを持ち、trait method declaration list や method signature AST をまだ持たない。そのため、この checkpoint は method-bearing trait の signature を正規化しない。`SelfhostModuleItemKind::TraitDecl`、`SelfhostModuleDeclarationKind::Trait`、public visibility、空の type annotation、空の lambda header、空の body envelope、空の first expression がそろう場合だけ marker shape evidence を返し、それ以外は `SelfhostMemoTraitSignatureShapeErrorKind` で fail-closed にする。

accepted marker signature hash は source text / span / syntax range / lexeme / path suffix / diagnostic text を材料にせず、domain と `SelfhostMemoTraitSourceKind` だけから作る。public surface seed はこの shape evidence を通して `normalized_signature_hash` を取り出すようになり、seed module 内で header/body range を直接 semantic hash へ変換しない。

subagent review では「現ASTで method parser 風の source scan を作らない」「module 名と payload 名が full signature normalization 済みに見えないようにする」「private visibility と header/body nonempty を個別に fail-closed にする」ことが確認されたため、module 名を `memo_trait_signature_shape` にし、private visibility / header type annotation / header lambda / body envelope / body first expression / item kind mismatch を stage0 と source policy で固定した。

残件は、body range を method declaration list へ分割する body segmenter、method name / type annotation / default body の stable signature normalization、re-export / import graph を含む full public surface hash、stable trait definition key producer、Copy / Drop / Eq / Hash pure evidence の実計算を接続し、実 stdlib の method-bearing `MemoKey` / `MemoValue` definition から stable source identity を生成することである。
