---
id: ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7
title: "MemoKey and MemoValue need structural purity rules"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-13
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

## 2026-06-12 selfhost public surface token item dispatch checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl` と `stdlib/neplg2/core/check/module/memo_trait_public_surface_token_gate.nepl` の item scan を、既存 `selfhost_module_item_kind_declaration` を使う二段階 dispatch へ寄せた。

この checkpoint では、AST-only seed scan と token-aware seed scan の本体が `SelfhostModuleItemKind` の長い全列挙を個別に持つ構造をやめ、non-declaration item policy と declaration item policy に分けた。trait declaration だけが marker / method-bearing trait signature normalization へ進み、public function / struct / enum / impl は同じ item pass で typed error にする。import / use / prelude / no-prelude は Phase 1 local memo trait pair surface を越えるため fail-closed に拒否する。

subagent review では、`memo_trait_public_surface_seed.nepl` から `memo_trait_public_surface_token_gate.nepl` を直接呼ぶ案は `seed -> token_gate -> seed` の循環 import を作るため不可と判断した。final dedup には、shared token-aware public surface seed scan core と専用 error enum を切り出し、seed/hash/token_gate が各自の error enum へ写像する設計が必要である。今回の変更はその前段 checkpoint であり、この issue を完了扱いにはしない。

source policy は `nodesrc/test_selfhost_memo_trait_public_surface_seed_contract.js` と `nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js` で更新し、seed 側と token_gate 側の二段階 dispatch、facade 非公開、trusted source identity 非生成、source/span/lexeme を accepted authority にしない境界を固定した。検証は focused contracts、seed/hash doctest、split/proof contract、Zenn review gate、source policy regression、issues check、`git diff --check` を通した。

## 2026-06-12 selfhost canonical key payload bytes codec checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload_codec.nepl` を追加し、`.neplproof` reader / serializer 全体の前段として serialized canonical key tree payload の bytes codec を分離した。

codec は header、fixed-width node table、argument table を decode し、stable nominal key material を `SelfhostMemoTraitStableNominalKeyTable` に照合して、現在の session 内の `SelfhostCanonicalTypeKeyArena` と root key を再構築する。serialized payload は payload schema、node kind、primitive stable code、stable nominal key material、argument order だけを authority とし、`SelfhostCanonicalTypeKeyId`、`SelfhostNamedTypeId`、`SelfhostTypeId`、proof store record index、source text、span、path、display name、diagnostic text、lexeme を永続 authority にしない。

decode error は `SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind` として typed enum にした。schema mismatch、unexpected end、trailing bytes、unknown tag/code、negative count、count limit、root / arg target out of range、invalid arg range、unsupported parameter / function node、missing / duplicate / invalid nominal key、word high-bit、allocation failure、hash projection failure を bool や表示文字列に潰さず返す。

`selfhost_memo_trait_canonical_key_payload_decode_and_hash_result` は convenience API として decoded arena/root から既存 `selfhost_memo_trait_canonical_key_payload_hash_result` を呼び、hash を再計算する。bytes 内や record key 内の hash を信用して acceptance する経路は持たない。preseed 側の authority は引き続き materialized arena/root と typed policy / proof payload の照合に置く。

source policy は `nodesrc/test_selfhost_memo_trait_canonical_key_payload_codec_contract.js` で固定した。facade re-export、`nodesrc/selfhost_ty_sources.js` 登録、DAG、forbidden authority、typed error、existing hash producer delegation、stage0 smoke、line count / doc comment length cap 禁止を確認する。

残件は、`.neplproof` record reader / serializer、decoded record から proof store へ append する preseed loop、persistent stable map / serialized index、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary である。

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

## 2026-06-12 selfhost public surface hash scan cost checkpoint

local public surface hash materializer の性能残件として残っていた unsupported public surface 専用 pre-scan と seed scanner 全体の再呼び出しを削除した。`selfhost_memo_trait_public_surface_hash_materialize_result` は module identity validation の後、hash module 内部の `selfhost_memo_trait_public_surface_hash_scan_module_result` で AST item stream を 1 回だけ走査し、unsupported public surface validation、marker trait seed accumulation、seed table complete check を同じ loop で行う。

token-aware path も同じ形にし、`selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result` は `selfhost_memo_trait_public_surface_token_gate_seed_table_with_tokens_result` を呼ばず、hash module 内部の `selfhost_memo_trait_public_surface_hash_scan_module_with_tokens_result` で module item stream を 1 回走査する。token-aware trait normalization は facade-external `memo_trait_public_surface_token_gate.nepl` の item helper だけを再利用する。`memo_trait_public_surface_token_gate_scan_item_result` は token gate module 内では `pub` だが、`module.nepl` facade は token gate module を re-export しない。

hash folding authority は引き続き seed table の typed field に限定する。source text、span、path suffix、display name、diagnostic text、lexeme は hash material に入れない。source slicing は `MemoKey` / `MemoValue` の trait name classification にだけ使う。proof store、`core/ty` source registry、`signature_available=true` source record、checker / HIR / Resource IR / backend への逆依存は追加していない。

source policy は `nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js` と `nodesrc/test_selfhost_memo_trait_public_surface_seed_contract.js` を更新し、hash-owned single-pass scan、whole-module seed scanner 呼び出し禁止、token gate item helper だけの再利用、unsupported public surface の typed error、authority boundary、line count / doc comment length cap 禁止を固定した。

subagent review では Euclid が、初回 Required として 1 pass helper を public surface hash 側へ置くこと、token-aware path も同じ shape にすること、source policy を hash-owned item loop 期待へ更新すること、計算量を `O(n + method tokens)` として明記することを求めた。実装後の re-review では Blocker / Required なし、Question なし、Approve と判断した。Non-blocker として registry convenience path の candidate scanner + materializer 2 traversal は残ると確認した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_seed_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl -o target/selfhost_hash_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl -o target/selfhost_seed_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/test_selfhost_module_checker_split_contract.js`
- pass: `node nodesrc/test_selfhost_proof_entry_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後も、registry convenience path は candidate scanner と materializer を順に呼ぶため 2 traversal が残る。候補 table と seed table の同時生成、facade-external token gate と seed module private token scan の重複整理、`trait_body_segmenter` の next-index recomputation 削減は後続 slice に残す。

## 2026-06-12 selfhost memo trait marker signature shape checkpoint

`stdlib/neplg2/core/check/module/memo_trait_signature_shape.nepl` を追加し、`SelfhostModuleAst` から得られる trait declaration header/body evidence が Phase 1 の public marker trait signature として扱えるかを typed `Result` で判定する境界を作った。

現 selfhost AST は `SelfhostModuleDeclarationBody` に body envelope と first expression の `SelfhostSyntaxRange` だけを持ち、trait method declaration list や method signature AST をまだ持たない。そのため、この checkpoint は method-bearing trait の signature を正規化しない。`SelfhostModuleItemKind::TraitDecl`、`SelfhostModuleDeclarationKind::Trait`、public visibility、空の type annotation、空の lambda header、空の body envelope、空の first expression がそろう場合だけ marker shape evidence を返し、それ以外は `SelfhostMemoTraitSignatureShapeErrorKind` で fail-closed にする。

accepted marker signature hash は source text / span / syntax range / lexeme / path suffix / diagnostic text を材料にせず、domain と `SelfhostMemoTraitSourceKind` だけから作る。public surface seed はこの shape evidence を通して `normalized_signature_hash` を取り出すようになり、seed module 内で header/body range を直接 semantic hash へ変換しない。

subagent review では「現ASTで method parser 風の source scan を作らない」「module 名と payload 名が full signature normalization 済みに見えないようにする」「private visibility と header/body nonempty を個別に fail-closed にする」ことが確認されたため、module 名を `memo_trait_signature_shape` にし、private visibility / header type annotation / header lambda / body envelope / body first expression / item kind mismatch を stage0 と source policy で固定した。

残件は、body range を method declaration list へ分割する body segmenter、method name / type annotation / default body の stable signature normalization、re-export / import graph を含む full public surface hash、stable trait definition key producer、Copy / Drop / Eq / Hash pure evidence の実計算を接続し、実 stdlib の method-bearing `MemoKey` / `MemoValue` definition から stable source identity を生成することである。

## 2026-06-12 selfhost trait body method segmenter checkpoint

`stdlib/neplg2/core/syntax/parser/trait_body_segmenter.nepl` を追加し、trait body envelope を top-level method declaration segment の列へ分解する parser utility を実装した。

この module は `KwFn ... : body` の method declaration だけを accepted segment とし、`header %SelfhostSyntaxRange` と `default_body %SelfhostSyntaxRange` を分けて返す。expression body 用の `body_segmenter` は `KwFn` を expression start として扱わないため、trait method declaration 用の token range evidence は別境界に分離した。

失敗は `SelfhostTraitBodyMethodSegmentErrorKind` と `Result` で返し、empty envelope、invalid envelope、token bounds、layout error、non-method item、colon 欠落、empty default body、allocation failure を区別する。accepted path は source text、lexeme、path suffix、diagnostic text、stable fingerprint、public surface hash、source identity を authority にしない。`core/ty`、proof store、memo trait source / policy はこの parser-level evidence へ逆依存しないことを source policy で固定した。

subagent review では、segment 分割を stable `MemoKey` / `MemoValue` source identity や fingerprint へ直結しないこと、method name / type annotation / default body normalization へ責務を飛ばさないこと、line count / doc comment length 制限を入れないことが確認された。実装では segmenter の doc comment と `nodesrc/test_selfhost_trait_body_segmenter_contract.js` で、現在の責務を method segment evidence までに限定し、signature normalization と stable identity 接続を後続 slice として残した。

runtime doctest は accepted 2 method、non-method top-level item、method body introducer 欠落、empty envelope を確認する。fixture 構築は `Result Vec SelfhostToken StdErrorKind` を使い、allocation failure を `OutOfMemory` に写して fail-closed に扱う。owner-backed aggregate field の解放は `field::get` を使い、直接 field access で compiler memory boundary を迂回しない。

残件は、method segment の header range から method name / type annotation / effect / default body shape を stable normalized signature evidence へ変換する normalizer、method-bearing `MemoKey` / `MemoValue` trait definition を public surface seed / stable source evidence へ接続する producer、re-export / import graph を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost trait body method segmenter scan cursor checkpoint

`trait_body_segmenter.nepl` の accepted method path から next-index recomputation を削除した。従来は `selfhost_trait_body_method_segment_from_method_line_result` が colon、body start、indented body 判定、nested body 終端を計算して segment を作った後、main loop が `selfhost_trait_body_method_segment_next_index` で同じ method line と default body をもう一度見て continuation index を計算していた。

この checkpoint では private `SelfhostTraitBodyMethodSegmentBuild` を追加し、method line parser が `segment` と `next_index` を同時に返すようにした。public `SelfhostTraitBodyMethodSegment` の shape は変えず、continuation cursor は parser 内部 payload に閉じる。`next_index <= start` の壊れた layout は `InvalidLayout` として typed error にし、`-1` sentinel や bool-only success にはしない。`selfhost_trait_body_method_segment_next_index` helper は削除し、main loop は build payload の `next_index` だけで次の top-level item へ進む。

source text、lexeme、path suffix、display name、diagnostic text、stable fingerprint、public surface hash、source identity は segmenter の accepted authority にしていない。`core/ty`、proof store、memo trait source / policy、checker / HIR / Resource IR / backend への逆依存も追加していない。

source policy は `nodesrc/test_selfhost_trait_body_segmenter_contract.js` を更新し、private build payload、forward continuation、main loop の payload consumption、next-index helper 禁止、line count / doc comment length cap 禁止を固定した。subagent review では Euclid が、private payload、typed non-forward error、loop の payload-only continuation、source policy 更新を Required として指摘し、実装に反映した。実装後の re-review では Blocker / Required なし、Non-blocker として `InvalidLayout` fail-closed、authority boundary、doc comment、line-count / comment-length 制限なしが確認され、Approve と判断された。

検証:

- pass: `node nodesrc/test_selfhost_trait_body_segmenter_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/trait_body_segmenter.nepl -o target/selfhost_trait_body_segmenter_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/test_selfhost_memo_trait_method_signature_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_method_signature.nepl -o target/selfhost_memo_trait_method_signature_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

## 2026-06-12 selfhost memo trait method signature normalizer checkpoint

`stdlib/neplg2/core/check/module/memo_trait_method_signature.nepl` を追加し、`trait_body_segmenter` が返す method header / default body range から `MemoKey` / `MemoValue` の method-bearing trait signature evidence を作る checker-layer normalizer を実装した。

`MemoKey` は `memo_key_eq` / `memo_key_hash32` の 2 method を固定順序で要求し、`MemoValue` は `memo_value_mark` の 1 method を要求する。method count、header range、method name、type annotation、lambda header、default body、segmenter rejection、hash placeholder は `SelfhostMemoTraitMethodSignatureErrorKind` として fail-closed に返す。

source text は canonical surface spelling の分類に使う。method 名、`Self` / `bool` / `i32` の type atom、literal default body、binder 参照の照合は source span から spelling を読むが、accepted fingerprint は受理後の fixed role code だけから作る。source span、syntax range、lexeme、path suffix、diagnostic text は hash material にしない。`memo_value_mark` は固定文字列 `value` を見るだけではなく、lambda binder と default body identifier の spelling が一致する場合だけ受理する。

公開 API は `selfhost_memo_trait_method_signature_result` に限定した。この関数は envelope から segmenter を呼び、segment list owner を必ず閉じる。`selfhost_memo_trait_method_signature_result_with_segments` は private helper にし、fake segment aggregate で segmenter provenance を迂回する public bypass を作らない。`module.nepl` facade にはまだ re-export せず、stable public-surface gate がこの normalizer を消費するまでは直接 module import できる leaf checker module として扱う。

subagent review では、source text / lexeme / span を hash authority にしないこと、spelling classification と fixed role code hash の境界を文書化すること、`memo_value_mark` の binder 不一致を拒否すること、`with_segments` を public API にしないことが Required / Blocker として確認された。実装では module doc、stage0、source policy を更新し、focused contract と doctest で退行を固定した。

source policy は `nodesrc/test_selfhost_memo_trait_method_signature_contract.js` で固定した。検査内容は、facade premature export の禁止、目的 / 契約 / 現状 / 計算量 / doctest、typed role / evidence / error、segmenter provenance、private with-segments helper、binder default body、hash function の source/span/range/lexeme 非依存、trusted source identity / registry / `signature_available=true` record の直生成禁止、`core/ty` / proof store 逆依存禁止、行数制限 / doc comment 長制限禁止を含む。

この checkpoint 後も、method-bearing `MemoKey` / `MemoValue` trait definition は public surface seed / stable source evidence pipeline へ未接続である。残件は、この normalizer を public surface seed materializer と stable source evidence producer へ接続すること、re-export / import graph を含む full public surface hash、stable trait definition key producer、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait method public surface gate checkpoint

`memo_trait_method_signature.nepl` の method-bearing signature evidence を、public surface seed / hash の token-aware path へ接続した。AST-only API は token authority を持たないため marker trait だけを受理する互換経路として残し、method-bearing `MemoKey` / `MemoValue` は lexer/parser が作った token stream と `SelfhostModuleAst` を同時に受け取る経路でのみ受理する。

`memo_trait_public_surface_seed.nepl` は、marker normalizer が body presence だけで拒否した場合に限って method normalizer へ fallback する。method normalizer の失敗は `MemoKeyMethodSignatureRejected %SelfhostMemoTraitMethodSignatureErrorKind` / `MemoValueMethodSignatureRejected %SelfhostMemoTraitMethodSignatureErrorKind` として payload を保持し、bool や汎用 unsupported に潰さない。accepted signature seed は marker / method normalizer の `normalized_signature_hash` だけから取り出し、source text、span、syntax range、method name spelling、path suffix、diagnostic text を accepted fingerprint authority にしない。

`memo_trait_public_surface_hash.nepl` は schema domain code を marker / method-bearing 用に更新し、token-aware materializer では facade-external internal module `memo_trait_public_surface_token_gate.nepl` を通して seed table を取得する。token-aware materializer / registry helper は `pub fn` にせず、`stdlib/neplg2/core/check/module.nepl` の `pub #import ... as *` から安定 facade に漏れないようにした。Fermat の review で facade premature export が blocker として見つかり、`test_selfhost_module_checker_split_contract.js` と `test_selfhost_proof_entry_contract.js` がその漏れを再現したため、token-aware shared gate を facade 外へ分離して修正した。

この checkpoint でも full public surface materializer ではない。残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、stable trait definition key producer、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。性能残件として、public surface seed の scanner / seed scan 二重走査、facade-external token gate と seed module private token scan の重複、`trait_body_segmenter` の next-index recomputation を後続 slice へ残す。

## 2026-06-12 selfhost memo trait stable definition key checkpoint

`stdlib/neplg2/core/check/module/memo_trait_definition_key.nepl` を追加し、`MemoKey` / `MemoValue` trait definition の stable definition key を作る checker-layer producer を実装した。

この producer は full public surface hash や proof store stored key の完成形ではない。入力は trait kind、caller が用意した module fingerprint、declaration ordinal に限定し、それらから schema version 付きの `SelfhostMemoTraitStableDefinitionKey` を返す。`schema_version`、`kind`、`module_fingerprint`、`definition_key_hash` は named field として保持し、equality でもすべて比較する。これにより、将来 key format が変わった場合に古い proof artifact と混同しない。

`module_fingerprint == 0`、declaration ordinal 欠落、declaration ordinal placeholder、derived definition key placeholder は `SelfhostMemoTraitStableDefinitionKeyErrorKind` として fail-closed に返す。source text、span、syntax range、file path、display name、diagnostic text は accepted key authority にしない。`memo_trait_definition_key.nepl` は `core/ty`、source identity record、proof store へ依存せず、`module.nepl` facade からも re-export しない。

`memo_trait_source_evidence_producer.nepl` は direct import で stable definition key producer を使うようになった。seed から stable evidence を作る accepted path では、raw `module_hash + kind + declaration_ordinal` fold ではなく `selfhost_memo_trait_stable_definition_key_result` を通し、`definition_key.definition_key_hash` を source symbol fingerprint として使う。stable definition key error は既存 seed error surface へ payload 付きで写す。

source policy は `nodesrc/test_selfhost_memo_trait_definition_key_contract.js` と `nodesrc/test_selfhost_memo_trait_source_evidence_producer_contract.js` で固定した。検査内容は、目的 / 契約 / 現状 / 計算量 / doctest、schema version、source kind code、placeholder rejection、facade 非公開、source spelling 非 authority、registry / source identity / `signature_available=true` record の直生成禁止、proof store 直結禁止、行数制限 / doc comment 長制限禁止を含む。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、stable nominal key / serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。性能面では、module fingerprint 計算が `.neplmeta` public interface artifact と checker-layer seed producer の間で重複しないように、次 slice で shared boundary を決める必要がある。

## 2026-06-12 selfhost memo trait canonical key checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_canonical_key.nepl` を追加し、`MemoKey` / `MemoValue` proof artifact 用の stable nominal key table と canonical type fingerprint sidecar projection を実装した。

この module は checker-layer の `memo_trait_definition_key.nepl` を import しない。`SelfhostNamedTypeId` は同じ session 内の constructor table index としてのみ扱い、永続 proof artifact の authority にはしない。caller が用意した module fingerprint、definition fingerprint、constructor ordinal、type arity から `SelfhostMemoTraitStableNominalKey` を作り、schema version と derived nominal key hash を含む named field payload として保持する。欠落、`0` placeholder、負の arity、derived hash `0` は `SelfhostMemoTraitStableNominalKeyErrorKind` で fail-closed に返す。

canonical type fingerprint は `SelfhostMemoTraitStableNominalKeyTable` を通して `Named` / `Applied` node を stable nominal key へ写す。table に record がない場合は `MissingNominalKey`、同じ `SelfhostNamedTypeId` に複数の stable key がある場合は `DuplicateNominalKey` として拒否し、first-wins にはしない。primitive は stable code へ畳み、generic parameter と function type はこの proof artifact fingerprint では `TypeParameterUnsupported` / `FunctionTypeUnsupported` として拒否する。argument range の破損、missing node、missing argument、derived fingerprint placeholder、壊れた arena による traversal fuel exhaustion も typed enum error に分ける。

accepted path は source text、span、syntax range、file path、display name、diagnostic text、lexeme を authority にしない。public wrapper は canonical key arena の node 数と argument 数から traversal fuel を作るため、正常な arena では key tree size に比例して終わり、破損した arena では無制限再帰ではなく `TraversalFuelExhausted` へ閉じる。stage0 smoke の失敗経路では type arena、stable nominal key table、projection 後の canonical key arena owner を明示的に解放する補助関数を持ち、テスト用コードでも owner boundary を曖昧にしない。

`memo_trait_proof_store.nepl` の doc comment は、この canonical key projection が proof store の sidecar stable projection であることを記載した。ただし proof store の stored proof lookup key 自体はまだ既存 canonical key arena を使っており、serialized canonical type fingerprint を stored proof input に混ぜる接続は次 slice に残す。

source policy は `nodesrc/test_selfhost_memo_trait_canonical_key_contract.js` で固定した。検査内容は、目的 / 契約 / 現状 / 計算量 / doctest、stable nominal key payload、typed error enum、missing / duplicate nominal key、Named / Applied の table 経由解決、argument range / traversal fuel boundary、checker-layer producer 非依存、source / span / path / display / diagnostic / lexeme 非 authority、proof store doc の sidecar projection 記述、行数制限 / doc comment 長制限禁止を含む。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、serialized canonical key fingerprint を proof store の stored proof 入力へ接続することである。

## 2026-06-12 selfhost memo trait proof store stable fingerprint checkpoint

`memo_trait_proof_store.nepl` の record に `stable_fingerprint %Option SelfhostMemoTraitCanonicalTypeFingerprint` を追加し、永続 artifact 由来の proof 入力を stable nominal key table 経由の canonical type fingerprint で fail-closed に検査する経路を接続した。

既存の `selfhost_memo_trait_proof_store_push` / `selfhost_memo_trait_proof_store_lookup_record` は session-local compatibility path として残し、record には `stable_fingerprint = none` を保存する。これにより、現在の selfhost stage0 や既存 session 内 lookup の互換性は維持しつつ、serialized proof artifact として再利用できる record と legacy record を型付きに区別できる。

新しい `selfhost_memo_trait_proof_store_push_stable_key` / `selfhost_memo_trait_proof_store_push_with_kind_stable_key` は、caller が外から fingerprint を差し込む API ではない。`SelfhostMemoTraitStableNominalKeyTable` と store 内 canonical key arena から `SelfhostMemoTraitCanonicalTypeFingerprint` を計算し、成功した場合だけ `some(fingerprint)` を record に保存する。fingerprint projection が失敗した場合は、records owner と projection 済み key arena owner を閉じて、`StableFingerprintProjectionRejected` を typed payload 付きで返す。

新しい `selfhost_memo_trait_proof_store_lookup_record_stable_key` は、lookup 側でも stable nominal key table から expected fingerprint を計算する。探索ではまず既存の cross-arena canonical equality を満たす record だけを候補にし、policy が一致した後に `stable_fingerprint` を検査する。legacy `none` record は `RecordStableFingerprintMissing` として fail-closed にし、`some` があっても fingerprint が違う record は `StableFingerprintMismatch` として拒否する。fingerprint が一致した場合でも、proof kind と existing producer gate を必ず通すため、fingerprint 単体を acceptance authority にはしない。

stage0 smoke では、legacy record の stable lookup が `RecordStableFingerprintMissing` になること、stable push した record が stable lookup で成功すること、同じ session-local canonical key でも stable nominal key table の definition fingerprint が違う場合に `StableFingerprintMismatch` になることを実行で固定した。source policy は proof store が source text、span、path suffix、display name、diagnostic text、lexeme、checker-layer definition key producer を accepted proof authority にしないこと、stable fingerprint path でも canonical equality / policy / producer gate を維持することを確認する。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer と proof store preseed、proof store の stable map / index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost memo trait proof store stable sidecar index checkpoint

`memo_trait_proof_store.nepl` の stable fingerprint 付き lookup に `SelfhostMemoTraitProofStoreStableIndexEntry` と `stable_index %Vec SelfhostMemoTraitProofStoreStableIndexEntry` を追加した。index entry は `stable_fingerprint` と `record_index` だけを保持し、proof、policy、canonical key は複製しない。これは accepted path の候補削減であり、proof reuse の authority ではない。

`selfhost_memo_trait_proof_store_push_with_kind_stable_key` は stable record を `records` に追加した後、同じ owner transition 内で `stable_index` へ entry を追加する。record push 失敗、index push 失敗、fingerprint projection 失敗、type projection 失敗の各経路で `records`、`stable_index`、`next_key_arena` の owner を閉じる。index push だけが失敗した場合は、追加済み `next_records` と index push error 側の vector owner の両方を閉じてから typed push error に戻す。

stable lookup はまず `selfhost_memo_trait_proof_store_find_projected_stable_index_loop` で fingerprint candidate を狭める。index hit 後も、record 側の stable fingerprint、cross-arena canonical equality、policy equality、proof kind、producer gate を必ず確認する。fingerprint 一致だけで `Ok` を返す経路は作っていない。index entry の `record_index` が壊れている、または entry が指す record に stable fingerprint が無い場合は `StableIndexMissing` で fail-closed にする。

index fast path が accepted proof を返せなかった場合は、既存の full stable scan を失敗分類用 fallback として使う。legacy record の `RecordStableFingerprintMissing`、stable fingerprint mismatch、policy mismatch の診断優先順位は維持した。一方で、full stable scan が `Ok` になるのに index lookup が `Ok` にならなかった場合は、index invariant 破損として `StableIndexMissing` に変換し、silent fallback accept で不整合を隠さない。

subagent review では Fermat と Goodall が、`HashMap` / `BTreeMap` へ進まず `Vec` sidecar index に閉じること、index は authority ではなく候補 narrowing に限定すること、producer gate まで維持すること、line count / doc comment length 制限を入れないことを Required とした。Fermat は index 欠落時の silent fallback accept を避ける typed error を Question / Required として挙げたため、`StableIndexMissing` を追加して fail-closed にした。

source policy は `nodesrc/test_selfhost_memo_trait_proof_store_contract.js` を更新し、store が stable sidecar index を所有すること、stable push が record index entry を追加すること、index lookup が fingerprint だけで受理しないこと、full scan accepted result を `StableIndexMissing` へ変換すること、source text / span / path suffix / display name / diagnostic text / lexeme / checker-layer definition key producer を authority にしないことを固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_store_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl --no-tree -j 1 --assert-io -o tmp/selfhost-memo-trait-proof-store-stable-index-focused.json`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`
- warning_checked: 初回 `node nodesrc/run_source_policy_regressions.js --warn-only` は実装前の `note.n.md` checkpoint 未記録を warning として検出した。`note.n.md` の selfhost checkpoint 追加後に再実行し、今回差分由来 warning が解消したことを確認した。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer と proof store preseed、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-13 selfhost memo trait operation proof table checkpoint

`memo_trait_operation_proof.nepl` を追加し、Copy / Drop / Eq / Hash の operation proof status を session-local table から aggregate proof producer へ渡す境界を固定した。

`SelfhostMemoTraitOperationProofRecord` は `SelfhostTypeId` と 4 種類の `SelfhostMemoTraitAggregateProofStatus` を保持する。`SelfhostMemoTraitOperationProofTable` は `Vec SelfhostMemoTraitOperationProofRecord` を所有し、lookup は `selfhost_type_id_eq` だけで行う。source text、span、path suffix、display name、diagnostic text、lexeme は proof authority にしない。

同じ TypeId の record が複数ある場合は、first-wins や後勝ちで続行しない。`selfhost_memo_trait_operation_proof_find_loop` は 2 件目の一致を見た時点で `DuplicateRecord` を返し、`selfhost_memo_trait_operation_proof_record_for_type_or_missing_result` は `RecordMissing` だけを all-missing record に畳む。`DuplicateRecord`、`MissingTypeRecord`、`RecordPushFailed` は Missing status に潰さず typed error のまま返す。

`selfhost_memo_trait_aggregate_proof_from_operation_table_result` は、まず `SelfhostTypeArena` に type record が存在するかを確認する。存在しない場合は `MissingTypeRecord` で拒否するため、fake TypeId の operation proof record だけで aggregate proof は作れない。table に record が無い場合は accepted proof にせず、Copy / Drop / Eq / Hash がすべて `Missing` の record として producer gate に渡す。

この checkpoint は operation proof status の運搬と producer 接続であり、trait impl table、method body purity、Drop なし proof、Eq / Hash の純粋性検査、recursive aggregate traversal、cycle boundary はまだ実装しない。既存 producer gate の rejection taxonomy を再実装せず、`SelfhostMemoTraitAggregateProof` を作った後は `selfhost_memo_trait_aggregate_proof_to_record` を通す。

source policy は `nodesrc/test_selfhost_memo_trait_operation_proof_contract.js` で、facade re-export、source list 登録、session-local table の契約、missing record fail-closed、duplicate record rejection、fake TypeId rejection、typed operation status、typed error equality、push failure owner cleanup、checker / HIR / Resource IR / backend への逆依存禁止、proof store / artifact / codec への依存禁止、line count / doc comment length cap 禁止を固定する。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_operation_proof_contract.js`
- pass: `node nodesrc/test_selfhost_ty_split_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-operation-proof.json`

subagent review では Bohr が Blocker として duplicate TypeId record の first-wins 問題を指摘し、Required として `RecordMissing` 以外を all-missing に畳まないこと、fake TypeId all-proven record の regression を追加することを求めた。対応として `DuplicateRecord`、明示的な `RecordMissing` 専用 missing conversion、duplicate / fake TypeId stage0 checks、source policy を追加した。再レビューでは Blocker / Required なしとなり、duplicate record、non-missing error preservation、fake TypeId regression が意図どおり fail-closed になっていることを確認した。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer と proof store preseed、永続 artifact 用 stable map / serialized index、generic instantiation identity の HIR / monomorphize / artifact 接続である。

## 2026-06-13 selfhost generic type argument identity checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_type_argument_identity.nepl` を追加し、generic instantiation の型引数列を `SelfhostTypeId` vector のまま永続 key material にしない stable identity boundary として分離した。

`selfhost_memo_trait_stable_type_argument_identity_result` は、caller が渡した `&SelfhostTypeArena`、`&SelfhostMemoTraitStableNominalKeyTable`、`&Vec SelfhostTypeId` を受け取り、substitution 済み concrete type argument を `selfhost_canonical_type_key_project_from_arena`、`selfhost_memo_trait_canonical_type_fingerprint_result`、`selfhost_memo_trait_canonical_key_payload_hash_result` へ通す。entry は ordinal、canonical fingerprint、canonical payload hash のみを保持し、aggregate hash は schema version と一緒に比較する。aggregate hash は compact lookup key であり、最終的な同一性 authority ではない。

この identity は proof acceptance ではない。MemoKey / MemoValue proof の受理、policy equality、producer gate、proof store lookup は後続に残す。source text、span、path、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId` は accepted authority にしない。

失敗は `SelfhostMemoTraitStableTypeArgumentIdentityErrorKind` で typed enum 化した。projection failure、fingerprint rejection、payload rejection、entry push failure、type argument missing、hash placeholder を bool や表示文字列に潰さず fail-closed に返す。

stage0 smoke は empty accepted identity、single accepted identity、ordered two-argument accepted identity、argument order sensitivity、missing nominal key、duplicate nominal key、type parameter unsupported、function type unsupported を public producer 経由で確認する。source policy は `nodesrc/test_selfhost_memo_trait_type_argument_identity_contract.js` で facade re-export、source list 登録、canonical producer delegation、typed error、forbidden authority、proof acceptance 非該当、line count / doc comment length cap 禁止を固定した。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary である。

## 2026-06-13 selfhost memo trait `.neplproof` persistent stable map checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_stable_map.nepl` を追加し、decoded `.neplproof` record vector から persistent stable map entry vector を作る producer と、canonical fingerprint + canonical payload hash による candidate range lookup boundary を実装した。

stable map entry は `canonical_fingerprint`、`canonical_payload_hash`、`policy`、`record_ordinal`、`record_payload_hash` を持つ。primary sort key は `(canonical_fingerprint.schema_version, canonical_fingerprint.root_hash, canonical_payload_hash)` で、同じ key の entry は `record_ordinal` 昇順で連続する。`policy` は後続の policy equality 用 payload として保持するが、sort key や proof acceptance authority にはしない。

lookup は sorted order validation 後に half-open lower-bound binary search と collision group count を行い、探索範囲を `O(log m + c)` に閉じる。producer は record key と record body を artifact validator へ再投入し、不正 record や placeholder hash を typed enum error として fail-closed に返す。Phase 1 producer は insertion sort 相当の O(n^2) だが、public contract は後続の persistent binary writer / stable map codec へ置換できる形にした。

この stable map は proof acceptance ではない。map hit、fingerprint hit、canonical payload hash hit、record payload hash hit だけで proof を受理する経路は持たない。canonical payload decode、payload hash 再計算、policy equality、proof kind、proof store preseed / lookup、producer gate は後続 boundary が再検査する。`SelfhostCanonicalTypeKeyId`、`SelfhostTypeId`、`SelfhostNamedTypeId`、source text、span、path suffix、display name、diagnostic text、lexeme は stable map authority にしない。

source policy は `nodesrc/test_selfhost_memo_trait_proof_stable_map_contract.js` を追加し、facade re-export、`nodesrc/selfhost_ty_sources.js` 登録、typed entry / range / error enum、record validator 委譲、lower-bound lookup、producer output order validation、proof store / decoded / reader / serializer への逆依存禁止、session-local id / source authority 禁止、line count / doc comment length cap 禁止を固定した。

subagent review では Anscombe が Blocker なしと評価した。Required として、stable map が proof store / preseed / producer gate を直接呼ばず candidate narrowing に限定されること、永続 authority に store-local / session-local id を入れないこと、source policy を専用に追加すること、module doc に目的・契約・現状・計算量・doctest を丁寧に書くこと、行数やコメント長を制限しないことを確認した。同じ slice 内で stable map を serializer と preseed の間に置く source order へ調整し、source policy と doctest で境界を固定した。

検証は、stable map contract、ty split contract、Zenn review gate、stable map doctest、artifact / index / preseed contract、decoded / reader / payload reader / serializer order contract、source policy regression、issue check、diff whitespace check を通過した。source policy regression は既存 documentation sample gaps と Node WASI ExperimentalWarning を表示するが exit 0 であり、今回追加した stable map 境界の failure ではない。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-13 selfhost memo trait `.neplproof` serialized index codec checkpoint

`.neplproof` の serialized sidecar index table を reader / serializer / payload reader の artifact codec path へ接続した。

この checkpoint では、record-only reader API は互換境界として残し、新しい indexed reader API を追加した。indexed API は header と fixed-width record table に続いて `index_count` 件の serialized sidecar index entry を読み、`selfhost_memo_trait_neplproof_decoded_artifact_from_record_and_index_tables` へ渡す。decoded artifact constructor は header、record/index table relation、sorted order を再検査し、失敗時には record owner と index owner を閉じる。

serialized index entry は 4 word 固定で、canonical fingerprint schema、canonical fingerprint root hash、record ordinal、record payload hash を持つ。reader は typed fingerprint を直接組み立てず、`memo_trait_proof_artifact.nepl` の `selfhost_memo_trait_neplproof_index_entry_from_parts_result` へ委譲する。これにより binary reader hot path が canonical key producer module へ直接依存せず、artifact schema boundary が serialized scalar fields の validator になる。

serializer は header、record table、serialized index table、payload section の順に bytes を作る。`selfhost_memo_trait_neplproof_serializer_index_entry_result` は index entry を artifact validator へ通してから fixed-width word へ投影し、invalid ordinal / stale fingerprint schema / placeholder payload hash を typed error として返す。stage0 smoke は invalid index entry と payload reader round-trip を追加し、writer と reader の layout drift を実行で検出する。

payload reader は payload section offset を indexed prefix byte count から計算するようにした。この byte count は payload reader 側の unchecked arithmetic ではなく、reader 側の `selfhost_memo_trait_neplproof_reader_indexed_prefix_byte_count_result` を通して取得する。これにより、過大な `record_count` / `index_count` は prefix owner の確保や copy に進む前に fail-closed になる。prefix は `selfhost_memo_trait_neplproof_reader_decoded_artifact_from_indexed_record_bytes_result` へ渡し、payload reader 自身は record tag decode、stored proof decode、index entry validation の別 authority を作らない。

今回の index hit は proof acceptance authority ではない。canonical payload hash、canonical fingerprint、policy、proof kind、stored proof payload、record payload hash、proof store relation、producer gate は後続の payload reader / preseed / proof store が再検査する。source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId`、fingerprint hit 単独、record payload hash 単独、index hit 単独は authority にしない。

subagent review では、最初の設計レビューが persistent stable map まで同時に進めることを避け、serialized index codec を独立境界にするよう求めた。実装後レビューでは、reader / serializer / payload reader が serialized index を proof acceptance authority にしていないことと、主要失敗経路の owner cleanup が成立していることを確認した。最終レビューの Required として、payload reader が prefix owner を copy してから reader 側の `IndexCountLimitExceeded` に到達する経路があり、巨大 `index_count` を allocation 前に拒否する reader doc comment 境界とずれると指摘された。同じ slice 内で reader 側の checked prefix byte count boundary を追加し、payload reader はこの Result を通ってから prefix copy と payload offset scan へ進む形へ修正した。Non-blocker として decoded artifact constructor の doc comment が index table を `authority` と表現しており、proof acceptance authority と誤読され得るため、`lookup source` として読み、index hit は proof acceptance authority ではないと明記する形へ修正した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_reader_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_payload_reader_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_serializer_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_index_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-artifact-index-parts.json`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-decoded-index.json`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_reader.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-reader-index.json`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-payload-index.json`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-serializer-index.json`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、永続 artifact 用 stable map、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-13 selfhost memo trait `.neplproof` serializer checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl` を追加し、`.neplproof` Phase 1 writer boundary を接続した。

この serializer は header word、fixed-width 31 word record entry、canonical payload bytes section entry を書く。byte/word 処理は `memo_trait_artifact_word_codec.nepl` の共有 31-bit word writer に委譲し、serializer 内では private endian writer や bit operation を持たない。header は `selfhost_memo_trait_neplproof_header_result`、record key/body は `selfhost_memo_trait_neplproof_record_key_result` と `selfhost_memo_trait_neplproof_record_result` に必ず戻すため、writer path でも既存 artifact schema validator が authority になる。さらに `Known(range)` field evidence は reader と同じ range validator を通し、reader が拒否する proof payload を writer が bytes 化しないようにする。

Phase 1 では serialized index table をまだ持たないため、header validation 後に `index_count == record_count` だけを受理し、それ以外は `IndexCountUnsupported` で fail-closed にする。payload hash、record payload hash、fingerprint、index hit は serializer の proof acceptance authority ではない。reader / payload reader / preseed / proof store / producer gate が、canonical payload decode、payload hash 再計算、policy、proof kind、store relation を後続で再検査する。

source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId`、proof store stable identity は serialized authority にしない。proof store boundary は stored proof payload enum/type definitions のためだけに import し、lookup / push / preseed / materialized acceptance API は呼ばない。producer boundary も aggregate proof evidence enum/type definitions のためだけに import し、producer gate は呼ばない。

owner contract は output `Vec u8` の消費に閉じる。validation 失敗や internal ordinal failure では serializer が partial output owner を閉じる。borrowed payload bytes は caller 所有であり、serializer は payload owner を閉じない。payload byte copy は `v::get` の impossible short read を `PayloadByteMissing` として扱い、`v::push` failure は `PayloadBytePushInvalid(StdErrorKind)` として `VecPushError` owner を閉じたうえで返す。

stage0 smoke は public header / record / payload entry writer を経由して bytes を作り、payload reader に渡して round-trip する。`accepted_writer`、`invalid_header`、`invalid_record`、`invalid_proof_payload`、`payload_reader_roundtrip` を `SelfhostMemoTraitNeplProofSerializerStage0Summary` として返し、doctest で成功 writer、index-count unsupported、record payload hash placeholder rejection、reader-inverse proof payload rejection、payload reader round-trip を確認する。

source policy は `nodesrc/test_selfhost_memo_trait_proof_serializer_contract.js` を追加し、facade re-export、`nodesrc/selfhost_ty_sources.js` の reader -> payload_reader -> serializer -> preseed 順序、shared word codec、artifact validator、Phase 1 `index_count == record_count`、fixed 31 word layout、artifact word projection、payload length prefix、reader-inverse field range validation、owner cleanup、payload reader round-trip、proof-store lookup / push / preseed / producer gate 禁止、source-derived authority 禁止、line count / doc comment length cap 禁止を固定した。既存 reader / payload reader / decoded source-order contract と `nodesrc/run_source_policy_regressions.js` にも serializer policy を登録した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_serializer_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_payload_reader_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_reader_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`
- pass: `node nodesrc/test_selfhost_ty_split_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-serializer.json`
- pass: `node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl -o tmp/selfhost_serializer_doctest_review.json --max-cases 1`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` fixed-width record-only reader checkpoint

`memo_trait_proof_reader.nepl` に、header reader の次段として fixed-width record table を typed record vector へ decode し、decoded artifact owner を作る public reader boundary を追加した。

この reader は artifact schema version 1 の record body を 31 word として扱う。word layout は reader doc comment に offset 単位で固定し、canonical payload schema / fingerprint schema / fingerprint root hash / canonical payload hash、`MemoKey` / `MemoValue` source identity、solver policy、stored proof kind、field evidence、Copy / Drop / Eq / Hash proof status、hazard evidence、key/value result、record payload hash を順に読む。

reader は `header.record_count` を record 件数の唯一の authority とし、bytes length から件数を推測しない。`record_count` は外部入力なので `RecordCountLimitExceeded` で上限を持たせ、現 slice の record-only API では `index_count != record_count` を `IndexCountUnsupported` として拒否する。record table 後に余分な word がある場合は `TrailingBytes` として拒否する。serialized index table と canonical payload bytes section は後続 slice の別 API と schema 境界で扱う。

tag 復元は reader 内の helper へ分けた。source kind、stored proof kind、field evidence、proof status、hazard、`Result unit SelfhostMemoTraitRejectKind` は raw `i32` のまま上位へ流さず、未知 tag、field range 不整合、`Ok` result に残った reject payload を `SelfhostMemoTraitNeplProofReaderErrorKind` の typed variant として fail-closed に返す。

record key は `memo_trait_proof_artifact.nepl` に追加した `selfhost_memo_trait_neplproof_record_key_from_parts_result` へ委譲する。これにより、binary reader は full canonical key producer を直接 import せず、serialized fingerprint parts を artifact schema boundary で検査してから既存 record key validator へ渡す。

decoded artifact owner は `selfhost_memo_trait_neplproof_decoded_artifact_from_records` で作る。この reader は proof acceptance authority ではない。index hit、record payload hash、fingerprint hit のいずれかだけで proof を受理しない。canonical payload decode、payload hash 再計算、materialized key existence、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続境界で再検査する。reader は typed proof payload constructors のために proof store module を import するが、lookup / push / preseed / stable materialization API は呼ばない。

性能残件への対応として、record word decode と stage0 record fixture writer は深い nested match chain ではなく、fixed-width word vector loop と field ordinal projection loop にした。record decode は record 数 n、fixed width w=31 に対して O(n * w) であり、schema version 1 では O(n) である。これにより、selfhost compiler の prefix/typecheck 探索空間を広げる巨大分岐式を避ける。

`record_word_push` は push failure 時に `VecPushError` 内の owner を閉じてから `RecordPushInvalid` を返す。source policy は `nodesrc/test_selfhost_memo_trait_proof_reader_contract.js` と artifact / decoded contract で固定した。検査内容は、decoded-before-reader source order、reader が full canonical key producer を import しないこと、31 word layout、record_count 上限、tag decoder 全体の unknown tag fail-closed、nonzero Ok payload rejection、record-only bounds、trailing bytes rejection、decoded artifact boundary delegation、proof-store typed payload constructor 限定、proof-store acceptance/preseed API 禁止、行数制限 / doc comment 長制限禁止を含む。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` serializer、canonical payload bytes section reader、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-13 payload section reader checkpoint

`.neplproof` fixed-width record-only reader の後段として、`stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl` を追加した。この reader は record table 直後の canonical payload bytes section を record ordinal 順に読み、decoded artifact owner、共有 materialized canonical key arena owner、record ordinal 対応 materialized key id vector owner を束ねて返す。

payload section は各 record につき `payload_byte_len` word と、その byte 数ぶんの canonical payload codec bytes を持つ。record-only prefix は既存 `selfhost_memo_trait_neplproof_reader_decoded_artifact_from_record_bytes_result` に渡して検査し、payload bytes は `selfhost_memo_trait_canonical_key_payload_decode_result` で decode する。decode ごとの arena-local root は `selfhost_canonical_type_key_copy_from_arena` により、共有 materialized arena へ複製する。

この checkpoint は proof acceptance ではない。payload reader は proof store、preseed、producer を import せず、proof store lookup / append / preseed API を呼ばない。payload hash、canonical fingerprint、policy、proof kind、store relation、producer gate は後続 preseed / proof store が再検査する。source text、span、path suffix、display name、diagnostic text、lexeme、record payload hash 単独、fingerprint hit 単独、index hit 単独、store-local id、session-local `SelfhostTypeId` は reader の accepted authority にしない。

失敗時には、この module が作った decoded artifact owner、materialized key arena owner、materialized key id vector owner、payload slice owner、decoded payload owner を閉じる。error は `HeaderReadInvalid`、`RecordDecodeInvalid`、`PayloadLengthWordInvalid`、`PayloadBytesUnexpectedEnd`、`PayloadDecodeInvalid`、`MaterializedKeyCopyInvalid`、`TrailingBytes` などの typed enum variant として返す。nested error equality も word codec、payload codec、reader、canonical key copy error の比較へ委譲する。

source policy は `nodesrc/test_selfhost_memo_trait_proof_payload_reader_contract.js` を追加し、facade re-export、`reader -> payload_reader -> preseed` source order、proof_store / preseed / producer import 禁止、record reader / payload codec / canonical key copy への委譲、source-derived authority 禁止、owner cleanup、stage0 smoke、行数制限 / doc comment 長制限禁止を固定した。既存 reader / decoded policy は payload reader 挿入後の source order へ更新した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_payload_reader_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_reader_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-payload-reader.json`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl -n 1`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` header reader codec checkpoint

`.neplproof` record reader / serializer の前段として、`stdlib/neplg2/core/ty/ty/memo_trait_artifact_word_codec.nepl` と `stdlib/neplg2/core/ty/ty/memo_trait_proof_reader.nepl` を追加した。

shared word codec は 31-bit nonnegative little-endian word の decode / encode だけを担当する。short input は `UnexpectedEnd`、4 byte 目 high bit は `WordHighBitUnsupported`、負の write は `NegativeWordUnsupported`、vector push failure は `PushFailed(StdErrorKind)` として分け、payload schema や proof acceptance へ混ぜない。`memo_trait_canonical_key_payload_codec.nepl` は byte offset / word index reader と word writer をこの shared codec へ委譲し、canonical key tree payload の復元責務に集中する。

`.neplproof` header reader は先頭 magic word `792013` と 5 個の header word を読み、既存 `selfhost_memo_trait_neplproof_header_result` に schema / count validation を委譲する。reader error は `MagicMismatch`、`WordReadInvalid(SelfhostMemoTraitArtifactWordReadErrorKind)`、`HeaderInvalid(SelfhostMemoTraitNeplProofArtifactErrorKind)` に分ける。

この checkpoint は `.neplproof` 全体の受理ではない。reader は trailing bytes、record body、sidecar index body、canonical payload bytes table、proof store preseed、artifact file I/O を扱わない。source text、span、path suffix、display name、diagnostic text、lexeme、store-local id、session-local `TypeId` は accepted authority にしない。

source policy は shared codec re-export、dependency order、reader error taxonomy、proof artifact header validator への委譲、proof store / preseed / decoded artifact への逆依存禁止、source-derived authority 禁止、行数制限や doc comment 長制限の禁止を固定する。

この checkpoint 後の残件は、`.neplproof` record body reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash を接続することである。

## 2026-06-12 selfhost public surface registry single-pass checkpoint

`memo_trait_public_surface_hash.nepl` の registry convenience path で残っていた candidate scanner と public surface hash materializer の module item stream 二重走査を、combined registry materializer へ移した。

過去 checkpoint に記録されている「registry convenience path は 2 traversal が残る」という残件は、この checkpoint で解消済みとして扱う。履歴上の古い記述は当時の scope と残件を示すものであり、最新の実装境界では registry convenience path も single-pass path である。

`SelfhostMemoTraitPublicSurfaceHashRegistryScanState` は candidate source table と public surface seed table を同じ item pass で更新する。ただし、candidate table は source spelling から `MemoKey` / `MemoValue` trait 候補を分類するためだけに使い、public surface hash の authority は seed table の typed field に限定する。成功 payload も `SelfhostMemoTraitPublicSurfaceHashRegistryMaterialization` の `candidates` と `materialization` に分け、candidate classification と accepted hash materialization を混同しない。

AST-only registry API は `selfhost_memo_trait_public_surface_hash_registry_materialize_result` を使い、token-aware registry API は `selfhost_memo_trait_public_surface_hash_registry_materialize_with_tokens_result` を使う。どちらも module identity validation の後、module item stream を 1 回だけ走査する。standalone `selfhost_memo_trait_definition_source_table_scan_module_result`、`selfhost_memo_trait_public_surface_hash_materialize_result`、`selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result` は、互換境界と単体検査用 API として残した。

error taxonomy は combined path でも collapse しない。candidate 分類の AST 不整合は `CandidateScanRejected(SelfhostMemoTraitDefinitionScanErrorKind)`、public surface seed / hash 側の拒否は `PublicSurfaceHashRejected(SelfhostMemoTraitPublicSurfaceHashErrorKind)`、stable evidence / registry validator の拒否は `SeedRegistryRejected(SelfhostMemoTraitStableSourceSeedRegistryErrorKind)` として分ける。同じ trait item で candidate 分類と seed 分類の両方が失敗し得る場合は candidate 分類を先に評価し、malformed trait header は hash error ではなく `CandidateScanRejected` へ寄せる契約を doc comment と source policy で固定した。

source policy は `nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js` を更新し、combined registry materializer、candidate / seed の同一 item pass、既存 standalone API の維持、registry API から旧 scanner + materializer sequence へ戻らないこと、candidate-first error precedence、line count / doc comment length cap 禁止を確認する。

subagent review では Hume が Blocker なしと判断した。Required として、candidate/source table failure、seed/hash failure、registry validation failure の error taxonomy 分離、既存 scanner / materializer API の維持、source policy の combined path 期待への更新、combined payload で candidate table と materialization を分けること、doc comment に DAG と O(n+k) の探索範囲を明記することを求めた。今回の実装ではすべて同じ slice 内で対応した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_source_scan_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_seed_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_token_seed_scan_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl -o target/selfhost_hash_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/test_selfhost_documentation_contract.js`
- pass_with_existing_gaps: `node nodesrc/run_source_policy_regressions.js --warn-only` exit=0。既存の stdlib / selfhost documentation gap sample と Node WASI ExperimentalWarning が表示されたが、この slice の source policy 退行はない。
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-13 recursive aggregate traversal checkpoint

`memo_trait_recursive_aggregate.nepl` を追加し、`MemoKey` / `MemoValue` aggregate proof の前段として aggregate field closure を cycle-safe に走査する境界を接続した。

この checkpoint は proof acceptance ではない。出力 summary は root、到達 aggregate 数、field edge 数、最大 depth だけを持ち、Copy / Drop / Eq / Hash proof status、hazard proof、accepted evidence record、proof store record は作らない。失敗は `SelfhostMemoTraitRecursiveAggregateErrorKind` に閉じ、layout rejection と stack push failure の nested payload を保持する。

走査は `memo_trait_layout.nepl` の public layout evidence validator だけを通る。primitive field は leaf、Named / Applied field は再帰、Parameter field は `LayoutRejected(GenericArgumentUnsubstituted)`、Function field は `UnsupportedFieldType` として扱う。cycle 判定は ancestry stack 上の再登場だけを拒否し、global visited first-wins で sibling branch を誤って拒否しない。depth は root depth 0 を含む fuel として明示し、上限を超えた場合だけ `DepthLimitReached` にする。

stage0 smoke は accepted root->primitive、nested accepted root->child->primitive、self-cycle、depth limit、primitive target、missing TypeId、parameter field rejection、function field rejection を public entry 経由で確認する。source policy は facade re-export、source list 登録、`layout < producer < recursive aggregate < operation proof` の順序、operation proof / proof artifact / HIR / Resource IR / backend / source text authority への逆依存禁止、typed error equality の wildcard 禁止、line count / doc comment length cap 禁止を固定した。

最適化は今必要なものと後から可能なものに分ける。今固定した traversal contract、typed error taxonomy、cycle / depth / unsupported field の fail-closed 境界は producer gate の意味に関わるため後回しにしない。一方で、layout lookup indexing、sibling branch summary memoization、persistent artifact 共有、branch 間再走査削減は public summary / error 契約を変えずに後から置換できるため、この checkpoint では未実装のまま次 stage に進める。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_recursive_aggregate_contract.js`
- pass: `node nodesrc/test_selfhost_ty_split_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_layout_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_operation_proof_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_recursive_aggregate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-recursive-aggregate.json`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/test_selfhost_documentation_contract.js`。既存 documentation gap sample は表示されるが baseline check は pass。
- pass: `node nodesrc/test_selfhost_memo_trait_predicate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_type_argument_identity_contract.js`
- pass_with_existing_gaps: `node nodesrc/run_source_policy_regressions.js --warn-only` exit=0。既存の stdlib / selfhost documentation gap sample と Node WASI ExperimentalWarning が表示されたが、この slice の source policy 退行はない。
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`。CRLF working-copy warning は表示されたが whitespace error はない。

この checkpoint 後の残件は、recursive aggregate summary を producer gate の実入力へ接続すること、Copy / Drop / Eq / Hash pure evidence の実計算、re-export / import graph / public non-trait declaration を含む full public surface hash、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation identity の HIR / monomorphize / artifact 接続である。

## 2026-06-12 selfhost token-aware public surface seed scan core checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_surface_token_seed_scan.nepl` を追加し、method-bearing `MemoKey` / `MemoValue` trait definition を扱う token-aware public surface seed scan を seed / hash / token_gate から独立した shared core に分離した。

shared core は `SelfhostModuleAst` と `SelfhostToken` stream を同じ parser input 由来の authority として受け取り、marker trait と method-bearing trait を同じ `SelfhostMemoTraitStableSourceSeedTable` へ投影する。accepted seed は memo trait kind、public visibility、固定 declaration ordinal、signature shape normalizer または method signature normalizer の `normalized_signature_hash` に閉じる。source text、span、syntax range、lexeme、path、display name、diagnostic text、source hash は seed / hash authority にしない。

`SelfhostMemoTraitPublicSurfaceTokenSeedScanErrorKind` を dedicated enum として追加した。候補欠落、重複、private visibility、signature placeholder、unsupported public surface、malformed item、method signature rejection を bool / string sentinel にせず variant で保持する。method signature rejection は `SelfhostMemoTraitMethodSignatureErrorKind` payload を持ち、seed / hash / token_gate 側の error mapping でも payload を落とさない。

`memo_trait_public_surface_seed.nepl` は旧 token-aware private scan helper を削除し、shared core の module result を seed error surface へ写してから既存 seed scan wrapper に戻す構造へ変更した。`memo_trait_public_surface_token_gate.nepl` は既存 caller 向け互換 wrapper として残し、scan 本体ではなく core error から seed error への mapping だけを持つ。`memo_trait_public_surface_hash.nepl` は token_gate wrapper を経由せず、shared core の item helper を直接使うことで、hash materializer の single-pass loop と同じ policy を共有する。

source policy は `nodesrc/test_selfhost_memo_trait_public_surface_token_seed_scan_contract.js` を追加し、shared core の lower-DAG 境界、seed/hash/token_gate 逆 import 禁止、facade re-export 禁止、dedicated error enum、method payload preservation、hash の direct core use、proof_store / HIR / Resource / backend / codegen 依存禁止、source identity / hash / span / path authority 禁止、line count / doc comment length cap 禁止を固定した。既存 seed/hash contracts も shared core delegation と token_gate thin wrapper を確認するよう更新した。

subagent review では Euclid が、shared core が seed / hash / token_gate に依存しないこと、dedicated error enum を持つこと、hash が token_gate wrapper ではなく core item helper を直接使うこと、facade が token_gate / core を re-export しないこと、source policy が proof_store / HIR / Resource / backend 依存や source identity / hash / span authority を拒否することを Required として確認した。実装はこの指摘に従った。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_token_seed_scan_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_seed_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_public_surface_hash_contract.js`
- pass: `node nodesrc/test_selfhost_module_checker_split_contract.js`
- pass: `node nodesrc/test_selfhost_proof_entry_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl -o target/selfhost_seed_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl -o target/selfhost_hash_doctest.json --no-tree -j 1 --dist web/dist --assert-io`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost decoded `.neplproof` batch preseed checkpoint

`memo_trait_proof_preseed.nepl` に `SelfhostMemoTraitNeplProofDecodedBatchRecord`、`SelfhostMemoTraitNeplProofPreseedBatchErrorKind`、`SelfhostMemoTraitNeplProofPreseedBatchError` を追加し、複数 decoded record を working proof store へ順に投入する batch preseed boundary を追加した。

batch input は materialized canonical key id、期待する proof store policy、typed artifact record を named field で持つ。`.neplproof` reader / serializer、record bytes decode、persistent stable map、serialized index はこの boundary の責務に含めず、既存の single-record materialized append boundary を record ごとに呼ぶ。fingerprint、payload hash、policy、record schema は single-record boundary 内で再検査されるため、batch record の key id だけを authority として受理しない。

`selfhost_memo_trait_neplproof_decoded_record_batch_append` は store owner を消費し、すべての record が `AcceptMissing` append または `ExistingMatching` skip で成功した場合だけ `Ok(store)` を返す。`RejectedConflict`、record validation、fingerprint、payload hash、policy、store append のいずれかで失敗した場合は、single-record append boundary が入力 store を閉じ、batch error は `record_ordinal` と nested append error を保持する。vector read が失敗した場合も `RecordMissing` として store を閉じ、partial seeded store を成功値として返さない。

stage0 smoke は empty batch、同一 record 2 件による existing match skip、2 件目 conflict による ordinal 1 の `RejectedConflict`、2 件目 invalid record による ordinal 1 の typed decision error を確認する。source policy は `nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js` で、batch record/error 型、nested equality、public batch API、入力順 loop、store cleanup、materialized fingerprint 再計算、stage0 summary、計算量 doc、line count / doc comment length cap 禁止を固定した。

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost decoded `.neplproof` artifact batch projector checkpoint

decoded artifact owner と decoded batch append boundary の間に、`SelfhostMemoTraitNeplProofDecodedArtifact` と materialized canonical key id vector から `SelfhostMemoTraitNeplProofDecodedBatchRecord` vector を作る projector boundary を追加した。

`selfhost_memo_trait_neplproof_decoded_batch_records_from_artifact_result` は artifact invariant を `selfhost_memo_trait_neplproof_decoded_artifact_validate_result` で再検査し、artifact header の `record_count` と materialized key id vector の長さを照合する。件数が一致しない場合は `MaterializedKeyCountMismatch` として拒否し、reader 側の record / canonical payload 対応付け漏れを batch append まで進めない。件数が一致しても、各 `SelfhostCanonicalTypeKeyId` が caller-owned materialized key arena で読めなければ `MaterializedKeyMissing(record_ordinal)` として拒否する。

projector loop は record ordinal ごとに materialized key id vector を読み、さらに caller-owned materialized key arena で key id の実在を確認してから、`selfhost_memo_trait_neplproof_decoded_artifact_record_at_result` で同じ ordinal の record を copy-out し、`SelfhostMemoTraitNeplProofDecodedBatchRecord` を作って output vector へ push する。record copy-out 失敗、key id vector 欠落、arena 内 key id 欠落、output vector allocation / push failure は `SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind` の typed variant として返す。部分構築した output vector はこの boundary 内で閉じるため、caller は失敗時に owner cleanup を追加で行わない。

この projector は `.neplproof` reader / serializer、record bytes decode、persistent stable map、serialized index の実体ではない。reader が typed record vector と materialized key id vector を作った後の接続点だけを固定する。artifact record key、materialized key id、expected policy を batch input へ束ねるだけであり、canonical payload hash、canonical fingerprint、policy、store relation、producer gate は後続の decoded batch append と proof store preseed decision が再検査する。

stage0 smoke は、1 record artifact と 1 key id vector から batch record vector を作る成功経路と、1 record artifact に空 key id vector を渡す `MaterializedKeyCountMismatch` を確認する。source policy は decoded module import、typed builder error、artifact validation、key id count check、record/key ordinal pairing、partial output cleanup、stage0 smoke、reader / serializer 未導入を固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-preseed-batch-builder.json`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost decoded `.neplproof` single-record append checkpoint

decoded canonical key payload bytes から proof store へ single record を投入する境界を接続した。

`key.nepl` には `SelfhostCanonicalTypeKeyCopyErrorKind` と `selfhost_canonical_type_key_copy_from_arena` を追加した。この API は `SelfhostTypeId` を持たない decoded canonical key tree を、source arena の key id 数値を再利用せず store-local arena へ複製する。source node / argument 欠落、argument range 破損、fuel exhaustion、allocation failure は typed enum で返し、失敗時は target arena owner を閉じる。新規 equality は wildcard arm ではなく、網羅 match で作る private code helper を通す。

`memo_trait_proof_store.nepl` には `selfhost_memo_trait_proof_store_push_materialized_key` を追加した。caller-owned materialized key arena と store-owned arena を cross-arena equality で比較し、duplicate / conflict 判定を append 前に行う。`SelfhostMemoTraitProofStorePushErrorKind::MaterializedKeyCopyRejected` は key copy の typed error payload を保持するため、copy failure を stable duplicate や bool へ潰さない。

`memo_trait_proof_preseed.nepl` には bytes-level decision API と append API を追加した。`selfhost_memo_trait_neplproof_record_preseed_decision_decoded_payload_bytes` は codec で decoded owner を作り、fingerprint と payload hash を materialized arena から再計算してから preseed decision を返す。`selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes` は `AcceptMissing` の場合だけ store materialized append を呼び、`ExistingMatching` は store を変更せず返し、`RejectedConflict`、decision error、decode error、fingerprint error、store append error は fail-closed に分類する。失敗時には入力 store owner を閉じ、partial seeded store を成功値として返さない。

`nodesrc/selfhost_ty_sources.js` に `memo_trait_proof_artifact.nepl` と `memo_trait_proof_preseed.nepl` を source-policy aggregate 対象として登録した。この登録で顕在化した documentation baseline 退行は、module doc の先頭位置と新規宣言コメントを直して baseline 以内へ戻した。`impl` 内の doc comment は現行 parser が受理しないため置かず、型 / module / public API の doc comment に contract を集約する。

subagent review では Hume が、decoded payload append は `TypeArena` / `TypeId` authority を復元するのではなく、store-local arena copy API を作って materialized key を duplicate check 前に投入する方針を要求した。さらに `ExistingMatching` は skip、`AcceptMissing` は append、`RejectedConflict` は fail-closed とし、batch preseed loop では partial seeded store を compile に使わない atomicity が必要と指摘した。今回の slice では single-record append boundary までを実装し、batch reader / preseed loop と atomic working store は次の残件に残した。

最終 review では Hume が Blocker / Required なしと判定した。stage0 は decoded payload bytes decision、empty-store append、same-record skip、conflict append を public decoded append API 経由で踏む。`missing_key` だけは decoded bytes では root 不正が decode 時点で拒否されるため、materialized decision API の欠落 key smoke として残す。

source policy は `nodesrc/test_selfhost_type_key_contract.js`、`nodesrc/test_selfhost_memo_trait_proof_store_contract.js`、`nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js` を更新した。canonical key copy boundary、store-local append、decoded bytes decision / append、typed append error、owner cleanup、codec import、aggregate registration、documentation baseline を固定している。

検証:

- pass: `node nodesrc/test_selfhost_type_key_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_canonical_key_payload_codec_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_store_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/test_selfhost_documentation_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl -n 1`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl --no-tree -o tmp/selfhost-memo-trait-proof-preseed-decoded-append.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl --no-tree -o tmp/selfhost-memo-trait-proof-store-materialized-append.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、`.neplproof` reader / serializer、複数 decoded record を atomic working store へ投入する batch preseed loop、persistent stable map / serialized index、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash を接続することである。

## 2026-06-12 selfhost memo trait canonical key payload hash checkpoint

`memo_trait_canonical_key_payload.nepl` を追加し、`.neplproof` record key 用の canonical payload hash を materialized canonical key arena から再計算する境界を作った。

前回の decoded preseed bridge は、canonical payload hash と canonical fingerprint が decoded canonical key payload から作られていることを前提にしていた。この checkpoint では、stage0 の固定値 `3003` と public preseed API の caller supplied raw hash を廃止し、`SelfhostMemoTraitStableNominalKeyTable` と `SelfhostCanonicalTypeKeyArena` から `SelfhostMemoTraitCanonicalKeyPayloadHash` を作る producer を接続した。hash の入力は payload schema、node kind、primitive stable code、stable nominal key、argument order に限定し、source text、span、path、display name、diagnostic text、lexeme、store-local id、session-local `SelfhostTypeId` は artifact authority にしない。

payload schema version は `selfhost_memo_trait_canonical_key_payload_schema_version` として canonical fingerprint schema から分けた。Phase 1 では値は同じ `1` だが、`.neplproof` artifact validation は payload schema と canonical fingerprint schema の両方を確認する。これにより、後続の serializer が canonical key tree bytes の encoding を変えた場合でも、fingerprint schema だけに依存しない invalidation 境界を持てる。

`SelfhostMemoTraitCanonicalKeyPayloadErrorKind` は missing node、missing argument、invalid argument range、derived placeholder、traversal fuel exhaustion、missing / duplicate nominal key、type parameter unsupported、function type unsupported を typed enum として返す。stage0 smoke は named / applied accepted path と、missing / duplicate nominal、parameter、function、missing node、missing argument、invalid range、cyclic fuel exhaustion を実行で固定する。

`memo_trait_proof_preseed.nepl` の public bridge API は `&SelfhostMemoTraitStableNominalKeyTable`、`&SelfhostCanonicalTypeKeyArena`、`SelfhostCanonicalTypeKeyId` を受け取り、内部で payload hash producer を呼んで record key の canonical payload hash と照合する。stage0 は producer の値を record key に入れ、hash mismatch は record key 側だけを壊して、再計算値との不一致として確認する。fingerprint mismatch、policy mismatch、invalid record、seeded store の `ExistingMatching` / `RejectedConflict` も同じ producer 境界を通る。`memo_trait_proof_artifact.nepl` は payload schema boundary を import し、record key validation で payload schema / fingerprint schema / fingerprint payload schema の三者を fail-closed に照合する。

subagent review では Raman が、次 slice は serialized canonical key tree payload codec / hash producer が最適であり、reader / serializer へ進む前に caller-supplied hash を排除するべきだと指摘した。実装後 review では Kuhn が、初期実装で public preseed API がまだ `materialized_canonical_payload_hash %i32` を受け取っていることを High、contract test が public 境界を固定していないことを Medium、`Applied` node hash に payload schema が混ざっていないことを Low として指摘した。修正後は public preseed API から raw hash を削除し、`CanonicalPayloadMaterializationInvalid %SelfhostMemoTraitCanonicalKeyPayloadErrorKind` を追加して producer failure を typed error として持ち上げ、contract test で raw hash API の復活を禁止した。Required として、store-local id 非 authority、stable nominal key への正規化、source text / span / path / display / diagnostic / lexeme 非 authority、hash と fingerprint を同じ decoded payload から再計算すること、Parameter / Function の fail-closed、payload schema version の分離、pure core boundary、typed enum error、丁寧な doc comment、line count / doc comment length cap 禁止が挙げられたため、この slice 内で反映した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_canonical_key_payload_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_canonical_key_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload.nepl --no-tree -o tmp/selfhost-memo-trait-canonical-key-payload.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl --no-tree -o tmp/selfhost-memo-trait-proof-preseed-payload.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl --no-tree -o tmp/selfhost-memo-trait-proof-artifact-payload.json -j 1 --assert-io --dist web/dist`

この checkpoint 後の残件は、`.neplproof` reader / serializer、serialized canonical key tree bytes codec、decoded record から proof store へ append する preseed loop、persistent stable map / serialized index、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` decoded preseed materialization checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl` を追加し、decoded `.neplproof` record を materialized canonical key と照合してから proof store preseed decision へ進める bridge boundary を作った。

この checkpoint は `.neplproof` reader / serializer 本体ではない。binary / text codec、canonical key tree payload serialization、payload hash producer、disk / bundled artifact I/O は後続 slice とする。今回の責務は、reader が decoded record と decoded canonical key payload を得た後、artifact schema validation、materialized key existence、canonical payload hash、canonical fingerprint、policy を照合し、問題がない場合だけ proof store の materialized preseed API へ委譲することである。

`SelfhostMemoTraitNeplProofPreseedErrorKind` は `ArtifactRecordInvalid`、`MaterializedCanonicalKeyMissing`、`MaterializedFingerprintMismatch`、`MaterializedPolicyMismatch`、`CanonicalPayloadHashMismatch` を持つ。artifact schema error は nested typed error を保持し、payload hash / fingerprint / policy mismatch はそれぞれ別 variant として扱う。bool や表示文字列に潰さず、caller が artifact discard、通常検査 fallback、diagnostic を選べるようにした。

proof store 側には `selfhost_memo_trait_proof_store_preseed_decision_materialized_key` を追加した。この public API は store-local `SelfhostMemoTraitProofStoreStableIdentity` を外へ出さず、caller-owned canonical key arena と store-owned canonical key arena を cross-arena equality で比較する。identity 判定は stable record fingerprint、canonical key equality、policy equality、fingerprint equality をすべて要求し、fingerprint-only acceptance は作らない。

stage0 smoke は empty store に対する `AcceptMissing` に加えて、stable proof を seed した store に対する decoded bridge API 経由の `ExistingMatching` と `RejectedConflict` を確認する。`RejectedConflict` は同じ key / policy / fingerprint / payload hash でも proof kind が違う decoded record を使い、same stable identity の差分 payload を上書きや first-wins で隠さないことを固定した。

subagent review では Hilbert が、bridge 境界自体は妥当だが、bridge API 自身の stage0 が `ExistingMatching` / `RejectedConflict` を実行していないことを Required として指摘した。修正後は proof store stage0 に依存せず、`selfhost_memo_trait_neplproof_record_preseed_decision_materialized` 経由で seeded store の skip / conflict を実行するようにした。

source policy は `nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js` を追加し、facade re-export、artifact schema validation、proof store delegation、materialized key existence、payload hash / fingerprint / policy check order、bridge stage0 の `ExistingMatching` / `RejectedConflict`、owner cleanup、artifact schema への store-local ID 混入禁止、fingerprint-only acceptance 禁止、line count / doc comment length cap 禁止を固定した。`nodesrc/run_source_policy_regressions.js` にも登録した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_store_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_ty_split_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl --no-tree -o tmp/selfhost-memo-trait-proof-preseed.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/selfhost-ty-proof-preseed-facade.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl -i stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl --no-tree -o tmp/selfhost-memo-trait-proof-store-artifact.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`

この checkpoint 後の残件は、`.neplproof` reader / serializer、serialized canonical key tree payload codec、decoded record から proof store append への投入、persistent stable map / serialized index、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` artifact schema checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl` を追加し、`MemoKey` / `MemoValue` stored proof を `.neplproof` artifact へ載せる前段の typed schema boundary を固定した。

この checkpoint は `.neplproof` の binary / text codec ではない。file I/O、byte parser、serializer、canonical key tree payload encoding、persistent stable map は後続 slice とする。今回の責務は、reader / serializer が最初に構築すべき header、serialized record key、record payload、sidecar index entry と、それらの fail-closed validation を selfhost compiler core の型で表すことである。

`SelfhostMemoTraitNeplProofHeader` は artifact schema、canonical payload schema、policy schema、record count、index count を持つ。schema mismatch や負の件数は `SelfhostMemoTraitNeplProofArtifactErrorKind` の enum variant として返し、record payload を読まず fail-closed にできる。

`SelfhostMemoTraitNeplProofRecordKey` は canonical payload schema、canonical fingerprint、canonical payload hash、typed solver policy を持つ。store-local `SelfhostCanonicalTypeKeyId`、`SelfhostTypeId`、`SelfhostNamedTypeId`、record index、source text、span、path、display、diagnostic、lexeme は serialized key authority にしない。canonical payload hash は placeholder `0` を拒否し、将来の canonical key tree serialization と fingerprint が同じ入力から来たことを確認するための boundary とする。

`SelfhostMemoTraitNeplProofRecord` は record key、`SelfhostMemoTraitStoredProofKind`、`SelfhostMemoTraitStoredAggregateProof`、record payload hash をまとめる。proof kind は stable identity には入れないが、record payload hash の対象であり、proof store preseed / lookup gate と producer gate が expected proof kind と stored proof payload を検査する。artifact schema 層だけで `KeyOnlyUnsupported` などを受理しない。

`SelfhostMemoTraitNeplProofIndexEntry` は canonical fingerprint、record ordinal、record payload hash だけを持つ候補 narrowing payload である。index entry は proof acceptance authority ではなく、index hit 後も record 側の fingerprint と payload hash に一致しなければ `IndexFingerprintMismatch` / `IndexRecordHashMismatch` で fail-closed にする。

stage0 smoke は accepted header / key / record / index に加えて、artifact schema mismatch、canonical key schema mismatch、canonical payload hash placeholder、policy schema mismatch、record index out of range、index fingerprint mismatch、index record payload hash mismatch を確認する。source policy は `nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js` で、store-local id や source text authority、fingerprint-only acceptance、後段 layer import、line count / doc comment length cap の退行を禁止する。

subagent review では Dewey が Required として、store-local identity と serialized artifact semantics の分離、fingerprint-only 受理禁止、checker / HIR / Resource / backend 非依存、source text / span / path / display / diagnostic / lexeme 非 authority、schema / policy / payload mismatch の fail-closed、source policy 追加を求めた。Curie は実装後に Blocker / Required なしと判断し、Non-blocker として regex source policy の限界と canonical payload schema / fingerprint schema の Phase 1 同一扱いを挙げた。Question だった proof kind semantic rejection は proof store / producer gate の責務として doc comment に明記した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl --no-tree -j 1 --assert-io -o tmp/selfhost-memo-trait-proof-artifact-focused.json`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、`.neplproof` reader / serializer、artifact から proof store preseed への投入、serialized canonical key tree payload codec、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` decoded index table validation checkpoint

`memo_trait_proof_artifact.nepl` に decoded artifact 全体の index table validation boundary を追加した。

この checkpoint は `.neplproof` reader / serializer 本体ではない。reader が header、record vector、sidecar index vector を decoded した直後、proof store preseed へ進む前に通す fail-closed gate である。

`selfhost_memo_trait_neplproof_index_table_result` は header の `record_count` / `index_count` と decoded vector length を照合し、各 record と各 index entry を既存の単体 validator へ再投入する。その後、index entry が指す record の canonical fingerprint と record payload hash が一致すること、さらに sidecar index table が record ordinal を一対一に覆っていることを検査する。

同じ record ordinal へ複数 index entry が向く場合は `SelfhostMemoTraitNeplProofIndexValidationErrorKind::IndexRecordOrdinalDuplicate`、どの index entry からも覆われない record ordinal がある場合は `IndexRecordOrdinalMissing` として拒否する。index table は候補 narrowing 用の sidecar であり、fingerprint hit や stable index hit を proof acceptance authority にしない。

stage0 smoke は accepted table、record count mismatch、index count mismatch、invalid record、invalid index entry、index-record mismatch、duplicate ordinal、missing coverage を public aggregate validator 経由で確認する。safe `Vec` からは通常作れない defensive missing entry は、duplicate scan と coverage scan の `v::get None` を `IndexEntryMissing` に分類する形で contract に固定した。source policy は `nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js` で、typed enum error、header count check、record / index revalidation、coverage loop、duplicate rejection、missing coverage rejection、defensive entry missing、fingerprint-only / stable-index-only authority 禁止、line count / doc comment length cap 禁止を固定した。

この checkpoint 後の残件は、`.neplproof` reader / serializer、persistent stable map / serialized index、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` sorted index lookup contract checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl` を追加し、decoded `.neplproof` sidecar index の sorted order と candidate range lookup contract を artifact schema から分離した。

この checkpoint は `.neplproof` reader / serializer や persistent map の実装ではない。reader が header / record vector / index vector を構築した後、public lookup API が header validation、decoded table validation、sorted order check を順に通し、canonical fingerprint に一致する index entry 群の `SelfhostMemoTraitNeplProofIndexCandidateRange` だけを返す。

candidate range は index vector 内の開始 index と件数だけを保持し、proof payload、policy、canonical payload hash を返さない。fingerprint hit は proof acceptance authority ではなく、後続の canonical payload decode、payload hash 再計算、policy、proof kind、decoded batch preseed、proof store lookup、producer gate に委譲する。

sorted order は `(canonical_fingerprint.schema_version, canonical_fingerprint.root_hash, record_ordinal)` の昇順で固定した。同じ fingerprint の複数 entry は collision candidate group として許可し、canonical equality 側で絞る。fingerprint が戻る場合は `FingerprintOrderInvalid`、同じ fingerprint group 内の record ordinal が狭義に増加しない場合は `RecordOrdinalOrderInvalid` で fail-closed にする。

subagent review では Euclid が Blocker なしと判断したうえで、同一 fingerprint の合法な collision group を runtime doctest / stage0 で確認することを Required とした。修正後は `accepted_collision_range` を追加し、同じ fingerprint の 2 entry が `candidate_count = 2` の accepted range になることを source policy と doctest の両方で固定した。

`SelfhostMemoTraitNeplProofSortedIndexErrorKind` は header rejection と decoded table rejection を nested typed payload として保持する。source text、span、path suffix、display name、diagnostic text、lexeme は lookup key、sort key、tie-break authority にしない。artifact serialized index と proof store stable sidecar index の authority も混ぜない。

source policy は `nodesrc/test_selfhost_memo_trait_proof_index_contract.js` で固定した。検査内容は facade re-export、artifact schema reuse、checker / HIR / Resource / backend 逆依存禁止、目的 / 契約 / 戻り値 / 現状 / 計算量 / doctest、candidate range payload、same-fingerprint collision accepted range、typed nested error、header/table/order/lookup の順序、fingerprint-only / index-only acceptance 禁止、source/span/path/diagnostic authority 禁止、line count / doc comment length cap 禁止を含む。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_index_contract.js`
- pass: `node nodesrc/test_selfhost_ty_split_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl --no-tree -o tmp/selfhost-memo-trait-proof-index.json -j 1 --assert-io --dist web/dist`

この checkpoint 後の残件は、`.neplproof` record reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` sorted index lower-bound lookup checkpoint

`memo_trait_proof_index.nepl` の public lookup contract は維持したまま、candidate group の開始位置探索を線形 scan から lower-bound binary search へ進めた。header validation、decoded table validation、sorted order check は引き続き public boundary 内で必ず実行し、その後に half-open range `[low, high)` で `target <= entry.canonical_fingerprint` となる最初の index を探す。

見つかった位置の fingerprint が target と一致する場合だけ、そこから連続する collision group を数えて `SelfhostMemoTraitNeplProofIndexCandidateRange` を返す。target が table 内に存在しない場合は、target が最初の entry より小さい場合でも最後の entry より大きい場合でも `CandidateMissing` に閉じる。defensive read failure は `IndexEntryMissing` のままで、bool や表示文字列には潰さない。

この checkpoint は proof acceptance ではない。binary search は artifact record ordinal 候補の探索範囲を O(log m + c) へ縮小するだけであり、canonical payload decode、payload hash 再計算、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続の authority として残す。c は同じ fingerprint の collision candidate 数である。

source policy は `nodesrc/test_selfhost_memo_trait_proof_index_contract.js` を更新し、lower-bound helper、overflow しにくい `low + (high - low) / 2` midpoint、candidate group count、計算量 doc、fingerprint-only / index-only acceptance 禁止、line count / doc comment length cap 禁止を固定した。

subagent review では Euclid が、validation 順序を `header_result`、`index_table_result`、`sorted_index_order_result`、lower-bound lookup のまま保つこと、lookup key は fingerprint だけにし record ordinal を tie-break authority にしないこと、midpoint を `low + (high - low) / 2` にすること、`CandidateMissing` を有効 table 上の候補不在だけに限定すること、doc comment の現状 / 計算量を更新することを Required とした。実装と source policy はこの指摘に合わせた。

## 2026-06-12 selfhost memo trait `.neplproof` sorted index producer checkpoint

`memo_trait_proof_index.nepl` に decoded record vector から sorted sidecar index vector を作る producer boundary を追加した。

`selfhost_memo_trait_neplproof_sorted_index_build_result` は `&Vec SelfhostMemoTraitNeplProofRecord` を借用し、成功時だけ owned `Vec SelfhostMemoTraitNeplProofIndexEntry` を返す。各 record は `selfhost_memo_trait_neplproof_record_key_result` と `selfhost_memo_trait_neplproof_record_result` で再検査し、作った entry も `selfhost_memo_trait_neplproof_index_entry_result` へ通す。不正 record から sidecar entry を作らない。

producer output は返却前に `selfhost_memo_trait_neplproof_header_result`、`selfhost_memo_trait_neplproof_index_table_result`、`selfhost_memo_trait_neplproof_sorted_index_order_result` を通す。これにより、producer 自身が「作るから正しい」と仮定せず、既存の decoded table aggregate validation と sorted order validation へ再投影する。

`SelfhostMemoTraitNeplProofIndexProducerErrorKind` は allocation / push failure、record missing、index slot missing、record invalid、index entry build rejection、produced table rejection、produced order rejection を typed enum として返す。record vector の defensive missing は `RecordEntryMissing`、bubble-back 中の produced index vector missing は `IndexEntryMissing` に分ける。nested payload は `StdErrorKind`、`SelfhostMemoTraitNeplProofArtifactErrorKind`、`SelfhostMemoTraitNeplProofIndexValidationErrorKind`、`SelfhostMemoTraitNeplProofSortedIndexErrorKind` のまま保持する。

現 stage の sort は bubble-back insertion sort で、record 数 n に対して O(n^2) である。これは Phase 1 の decoded artifact contract を固定するための実装であり、後続の binary writer / persistent stable map / serialized index では同じ Result contract を保ったまま O(n log n) または O(n) へ置き換える。

producer output は proof acceptance authority ではない。返す sidecar index は canonical fingerprint、record ordinal、record payload hash による candidate narrowing table であり、canonical key bytes decode、payload hash 再計算、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続に残る。source text、span、path suffix、display name、diagnostic text、lexeme、`SelfhostTypeId`、`SelfhostNamedTypeId`、`SelfhostCanonicalTypeKeyId`、proof store stable identity は producer の key や authority にしない。

producer implementation は proof store lookup / push / preseed、store-local stable identity、decoded batch append API を直接呼ばない。source policy はこの禁止境界も固定し、producer が proof-store API と結合して proof acceptance を迂回しないようにする。

stage0 smoke は unordered record vector から first / second fingerprint の candidate range を作れること、same fingerprint 2 件が collision group として `candidate_count = 2` になること、invalid record が `RecordInvalid(RecordPayloadHashPlaceholder)` として fail-closed に拒否されることを確認する。

source policy は `nodesrc/test_selfhost_memo_trait_proof_index_contract.js` を更新し、producer doc、O(n^2) 現状説明、typed producer error、public producer API、record / index revalidation、post-build table/order validation、bubble-back insertion、producer から proof-store / preseed / decoded-batch append API への直接呼び出し禁止、unordered stage0 smoke、invalid record rejection、source/span/path/diagnostic authority 禁止、line count / doc comment length cap 禁止を固定した。

subagent review では Euclid が、Phase 1 では同 module へ producer を置く判断を妥当とした。Required として、producer output を proof acceptance にしないこと、source text / span / path / display / diagnostic / lexeme と session-local id / proof store stable identity を authority にしないこと、record validation を省略しないこと、producer output を既存 table/order validator へ通すこと、O(n^2) 現状と将来置換可能性を doc comment に明記すること、source policy で退行を固定することを求めた。実装はこの指摘に従った。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_index_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl -n 1`

この checkpoint 後の残件は、`.neplproof` record reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` decoded artifact owner checkpoint

`stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl` を追加し、decoded record vector と sorted sidecar index vector を 1 つの owner boundary に束ねた。

`SelfhostMemoTraitNeplProofDecodedArtifact` は header、records `Vec` owner、indexes `Vec` owner を持つ。成功値を受け取った caller は `selfhost_memo_trait_neplproof_decoded_artifact_free` で両方の `Vec` owner を閉じる。現 stage の record / index entry は Copy payload だけを持つため deep free は不要だが、後続で canonical payload bytes などの owner payload を埋め込む場合は、この free boundary も同じ時点で拡張する。

`selfhost_memo_trait_neplproof_decoded_artifact_from_records` は入力 record owner を受け取り、`selfhost_memo_trait_neplproof_sorted_index_build_result` で sorted sidecar index を作り、header validation、decoded table validation、sorted order validation を通した場合だけ artifact owner を返す。index build 失敗時は records を閉じ、post-build validation 失敗時は records と indexes の両方を閉じる。caller は error payload だけを受け取り、失敗時 owner を回収しない。

lookup は proof acceptance ではなく candidate narrowing だけを返す。`SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::CandidateMissing` を追加し、有効な artifact に対象 fingerprint の candidate がない場合を `LookupInvalid` から分離した。`LookupInvalid` は header / table / sorted order の破損に由来する lookup 前提の拒否だけに使う。

decoded artifact owner は `memo_trait_proof_store` / `memo_trait_source` を direct import しない。stage0 smoke は `selfhost_memo_trait_neplproof_artifact_stage0` の accepted record を再利用し、proof store policy や source identity の構築詳細を decoded artifact 層へ持ち込まない。source policy は proof-store / source / preseed API、checker / HIR / Resource / backend 逆依存、source text / span / path / display / diagnostic / lexeme / session-local id authority、line count / doc comment length cap を禁止する。

Resource IR の initialized-state 検査により、stage0 の直接 field chain `first_record.key.canonical_fingerprint` は owner の部分移動として拒否された。そのため `field::get_ref` を使い、Copy payload を owner 非消費で読む形に修正した。これは検査の迂回ではなく、owner boundary を明示する根本修正である。

subagent review では Euclid が初回 Required として、cleanup contract、`CandidateMissing` と corruption の分離、proof_store/source direct import 禁止、source text / span / path / diagnostic / lexeme / session-local id authority 禁止、Copy payload の free boundary doc を求めた。再レビューでは Blocker / Required なしとし、上記が反映済みであることを確認した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl --no-tree -o tmp/selfhost-memo-trait-proof-decoded-focused.json -j 1 --assert-io --dist web/dist`
- pass: `node nodesrc/test_selfhost_ty_split_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

focused doctest の timing では `compile_ms=13572`、`resource_static_check=12416.146ms`、`resource_static_initialized_moves=11173.328ms`、`resource_static_owner_obligations=818.486ms` だった。RPN cold base 高速化と同じく、cache だけでなく Resource checker の探索範囲削減は継続課題である。

この checkpoint 後の残件は、`.neplproof` reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash を接続することである。

## 2026-06-12 selfhost memo trait `.neplproof` decoded artifact candidate record checkpoint

`memo_trait_proof_decoded.nepl` に、sorted index lookup が返した candidate range から index entry と decoded record を copy-out する public boundary を追加した。

`selfhost_memo_trait_neplproof_decoded_artifact_candidate_record_at_result` は `&SelfhostMemoTraitNeplProofDecodedArtifact`、lookup に使った `SelfhostMemoTraitCanonicalTypeFingerprint`、`SelfhostMemoTraitNeplProofIndexCandidateRange`、range 内の相対 offset を受け取る。artifact invariant を再検査し、range / offset、overflow defensive guard、target fingerprint、index entry と record の対応を確認してから `SelfhostMemoTraitNeplProofDecodedCandidateRecord` を返す。

`SelfhostMemoTraitNeplProofDecodedCandidateRecord` は `index_entry` と `record` の Copy payload pair であり、artifact owner 内部の storage location や reference は外へ出さない。これは proof acceptance ではなく、後続の canonical payload decode、policy、proof kind、decoded batch preseed、proof store lookup、producer gate へ渡す候補 projection である。

error は `SelfhostMemoTraitNeplProofDecodedArtifactErrorKind::CandidateAccessInvalid(SelfhostMemoTraitNeplProofDecodedCandidateErrorKind)` として分けた。`CandidateRangeInvalid`、`CandidateOffsetOutOfRange`、`CandidateIndexEntryMissing`、`CandidateRecordEntryMissing`、`CandidateTargetFingerprintMismatch`、`CandidateRecordFingerprintMismatch`、`CandidateRecordHashMismatch`、`CandidateRecordValidationUnexpected(SelfhostMemoTraitNeplProofArtifactErrorKind)` を typed variant として保持し、`CandidateMissing`、`LookupInvalid`、単体 accessor の `RecordEntryMissing` / `IndexEntryMissing` と混ぜない。equality helper も wildcard を使わず、nested payload まで比較する。

stage0 smoke は accepted lookup range から candidate record を取り出せること、invalid range が `CandidateRangeInvalid` になること、`candidate_offset = range.candidate_count` が `CandidateOffsetOutOfRange` になること、別 fingerprint target が `CandidateTargetFingerprintMismatch` になること、同じ fingerprint の collision group で `offset=1` の record を取り出せることを追加で確認する。source policy は candidate record struct、candidate error enum、artifact validation、range / offset guard、target fingerprint check、index-entry / record match、typed error equality、proof-store / preseed / source / checker-HIR-resource-backend direct dependency 禁止を固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-decoded-candidate-record.json`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_index_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass_with_existing_gaps: `node nodesrc/run_source_policy_regressions.js --warn-only` exit=0。既存の stdlib / selfhost documentation gap sample と Node WASI ExperimentalWarning が表示されたが、この slice の source policy 退行はない。
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後も、`.neplproof` reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

## 2026-06-12 selfhost memo trait `.neplproof` candidate batch record projector checkpoint

`memo_trait_proof_preseed.nepl` に、sorted index lookup が返した candidate range から decoded batch append 用の `SelfhostMemoTraitNeplProofDecodedBatchRecord` を 1 件作る public boundary を追加した。

`selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result` は `&SelfhostMemoTraitNeplProofDecodedArtifact`、`&SelfhostCanonicalTypeKeyArena`、`&Vec SelfhostCanonicalTypeKeyId`、期待する `SelfhostMemoTraitProofStorePolicy`、lookup target fingerprint、`SelfhostMemoTraitNeplProofIndexCandidateRange`、candidate range 内 offset を受け取る。内部では `selfhost_memo_trait_neplproof_decoded_artifact_candidate_record_at_result` を必ず通し、artifact invariant、range / offset、target fingerprint、index entry と record の対応を decoded artifact 側で再検査する。

candidate range 内 offset は record ordinal でも materialized key id ordinal でもない。batch record projector は candidate accessor が返した `index_entry.record_ordinal` を取り出し、その ordinal で materialized key id vector を読む。件数不一致は `MaterializedKeyCountMismatch`、arena 内 key id 欠落は `MaterializedKeyMissing(record_ordinal)`、candidate accessor の拒否は `CandidateInvalid(SelfhostMemoTraitNeplProofDecodedBatchCandidateError { candidate_offset, kind })` として fail-closed にする。これにより、candidate narrowing の offset と decoded record の stable ordinal を混同しない。

candidate record の nested payload は `field::get_ref` で `index_entry` と `record` を一段ずつ copy-out してから batch record へ束ねる。直接 `candidate_record.index_entry.record_ordinal` のように深く読む形にはしない。これは Resource IR 上で partial field move と未初期化 cell を作らず、Copy payload pair の owner lifecycle を明確に保つためである。

この projector は proof acceptance ではない。candidate accessor が target fingerprint と record payload hash の整合を確認しても、canonical payload hash、canonical fingerprint、policy、store relation、producer gate は後続の decoded batch append と proof store preseed decision が再検査する。stage0 smoke は candidate success と candidate offset out-of-range を追加し、source policy は candidate accessor 経由、key id count check、`record_ordinal` による materialized key lookup、arena existence check、typed `CandidateInvalid` wrapping、`candidate_offset` を key id ordinal として使わないことを固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-preseed-candidate-batch.json`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_index_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass_with_existing_gaps: `node nodesrc/run_source_policy_regressions.js --warn-only` exit=0。既存の stdlib / selfhost documentation gap sample と Node WASI ExperimentalWarning が表示されたが、この slice の source policy 退行はない。
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後も、`.neplproof` reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

## 2026-06-12 selfhost memo trait `.neplproof` candidate range preseed checkpoint

`memo_trait_proof_preseed.nepl` に、sorted sidecar index lookup が返した candidate range を working proof store へ投入する public preseed boundary を追加した。

`selfhost_memo_trait_neplproof_decoded_candidate_range_preseed` は proof store owner を消費し、stable nominal key table、materialized canonical key arena、decoded artifact、materialized key id vector、期待 policy、lookup target fingerprint、candidate range を受け取る。各 candidate は range-local `candidate_offset` で走査し、既存の `selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result` で batch record へ変換した後、`selfhost_memo_trait_neplproof_record_append_materialized_checked` へ渡す。

この API は proof acceptance authority ではない。candidate range は探索範囲を狭めるだけであり、canonical payload hash、canonical fingerprint、policy、store relation、proof store append は既存 append boundary が再検査する。`candidate_offset` は error payload の range-local offset であり、materialized key id vector ordinal、artifact record ordinal、sorted index absolute position として使わない。key id lookup は single-candidate projector が candidate accessor から得た `index_entry.record_ordinal` だけで行う。`expected_policy` は caller supplied の reuse boundary として渡し、record key から導出しない。

失敗は `SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedError` と `SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedErrorKind` に閉じた。candidate build failure では append へ進む前なので range preseed helper が入力 store owner を閉じて `BuildInvalid` を返す。append failure では既存 append boundary が store owner を閉じるため、range preseed helper は二重 close せず `AppendInvalid` を返す。どちらも failing candidate offset と nested typed error を保持し、bool や diagnostic string へ潰さない。

既存の `SelfhostMemoTraitNeplProofPreseedStage0Summary` はすでに大きいため、candidate range smoke は summary field として増やさず、`selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_smoke` で成功系と invalid range typed rejection を実行する形にした。source policy は summary block へ candidate range field / error 型が混入しないこと、smoke が既存 stage0 内で実行されること、candidate range public API が append boundary を迂回しないことを固定する。

subagent review では Bacon が初回 Required として、summary 拡張禁止 regex が弱いこと、candidate range smoke helper が未到達なことを指摘した。同じ slice 内で summary block 単位の negative source policy と実行される smoke helper を追加した。再レビューでは Blocker / Required なしで approve され、proof acceptance authority、owner cleanup、candidate offset の扱い、doc comment、summary 非拡張方針に追加修正要求なしと確認された。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_preseed_contract.js`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-trait-proof-preseed-candidate-range.json`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_decoded_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_index_contract.js`
- pass: `node nodesrc/test_selfhost_memo_trait_proof_artifact_contract.js`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass_with_existing_gaps: `node nodesrc/run_source_policy_regressions.js --warn-only` exit=0。既存の stdlib / selfhost documentation gap sample、Node WASI ExperimentalWarning、Git の LF/CRLF working-copy warning が表示されたが、この slice の source policy 退行ではない。
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後も、`.neplproof` reader / serializer、persistent stable map / serialized index の実体、generic instantiation 用 stable type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

## 2026-06-12 selfhost memo trait proof store preseed decision checkpoint

`memo_trait_proof_store.nepl` に `SelfhostMemoTraitProofStorePreseedDecision` を追加し、`.neplproof` reader / serializer が store へ proof record を投入する前の store-local preseed 判定を typed enum として固定した。

`AcceptMissing` は同じ stable identity を持つ record が無く、candidate を追加してよい状態を表す。`ExistingMatching` は同じ stable identity と同じ proof payload が既にあるため、reader が再追加せず skip してよい状態を表す。`RejectedConflict` は同じ stable identity に異なる proof payload が見つかった状態であり、上書き、後勝ち、first-wins、warning-only で続行してはいけない。

stable identity は既存 checkpoint と同じく `SelfhostCanonicalTypeKeyId`、`SelfhostMemoTraitProofStorePolicy`、`SelfhostMemoTraitCanonicalTypeFingerprint` の組である。`proof_kind` は stable identity に含めず、payload equality で比較する。これにより、same key / policy / fingerprint の `KeyAndValue` と `KeyOnlyUnsupported` が共存して lookup の挿入順に意味が依存する退行を避ける。

stored proof payload equality は、`fields`、`copy_proof`、`drop_proof`、`eq_proof`、`hash_proof`、`hazard`、`key_result`、`value_result` をすべて比較する。`Known(range)` は range payload まで、`Result::Err(kind)` は `SelfhostMemoTraitRejectKind` まで比較し、bool や表示文字列には潰さない。legacy `stable_fingerprint = none` record は preseed 判定の stable identity 対象から除外する。

stage0 smoke では、stable push 済み record に対して同じ proof kind / payload を候補にした場合に `ExistingMatching` になること、同じ identity でも proof kind が違う場合に `RejectedConflict` になることを実行で固定した。preseed smoke の失敗経路では、`store3` と stable nominal table 2 つを閉じる abort helper を追加し、テスト用の失敗経路でも owner boundary を曖昧にしないようにした。

subagent review では Beauvoir が、この slice は `.neplproof` reader / serializer と proof store preseed の自然な前段であり、same stable identity の再投入を常に duplicate error にせず existing matching と conflict に分ける方針を妥当と評価した。Required として、conflict fail-closed、existing matching skip、proof kind の payload equality 側比較、stored proof 全 field 比較、fingerprint-only / stable_index-only / source text authority / line count cap 禁止の source policy 固定を求めたため、同じ slice 内で反映した。

実装後 review では Confucius が Blocker / Required なしで approve した。typed enum で conflict を隠していないこと、proof kind が identity ではなく payload equality 側で比較されていること、payload 比較漏れ、owner cleanup、source policy 固定不足がないことを確認した。Non-blocker として、stage0 の `RejectedConflict` 実行例は proof kind 差分で踏んでおり、将来さらに強めるなら fields / proof status / hazard / key_result / value_result 各 field 差分の behavioral doctest を追加できると指摘した。

source policy は `nodesrc/test_selfhost_memo_trait_proof_store_contract.js` を更新し、preseed enum、preseed decision doc、stored proof kind equality、stored aggregate proof equality、Result payload equality、record payload matching、preseed scan、stage0 existing / conflict regression、line count / doc comment length cap 禁止を固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_store_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl --no-tree -j 1 --assert-io -o tmp/selfhost-memo-trait-proof-store-preseed-focused.json`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `git diff --check`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost memo trait proof store stable identity checkpoint

`memo_trait_proof_store.nepl` に `SelfhostMemoTraitProofStoreStableIdentity` を追加し、stable duplicate 判定と将来の stable map / serialized index preseed が共有する store-local identity boundary を名前付き struct にした。

identity fields は `SelfhostCanonicalTypeKeyId`、`SelfhostMemoTraitProofStorePolicy`、`SelfhostMemoTraitCanonicalTypeFingerprint` に限定する。`SelfhostCanonicalTypeKeyId` は store の `SelfhostCanonicalTypeKeyArena` と対でだけ意味を持つため、この struct をそのまま `.neplproof` の serialized key へ書き出してはいけない。永続 artifact では canonical key の stable serialization と policy identity を別 payload として保存する。

proof kind は stable identity に含めない。同じ canonical key / policy / stable fingerprint の record が proof kind だけ違って複数存在すると、lookup の first-wins により片方が隠れ、永続 artifact の意味が挿入順に依存するためである。proof kind は lookup / producer gate の互換性確認で扱い、same stable identity の duplicate を許す理由にはしない。

`selfhost_memo_trait_proof_store_stable_identity_new`、`selfhost_memo_trait_proof_store_stable_identity_eq`、`selfhost_memo_trait_proof_store_record_stable_identity_matches` を追加し、duplicate scan は loose tuple ではなく typed identity helper を通る。identity equality は canonical equality、policy equality、stable fingerprint equality をすべて要求し、fingerprint-only acceptance を作らない。

stage0 smoke は、1回目を `KeyAndValue` stable push、2回目を `KeyOnlyUnsupported` stable push にして、proof kind が異なっても same stable identity なら `StableDuplicate` になることを実行で固定した。source policy は `nodesrc/test_selfhost_memo_trait_proof_store_contract.js` で、store-local identity struct、serialized `.neplproof` key との分離、proof kind 除外、typed identity equality、record matcher、typed duplicate scan、異なる proof kind の duplicate regression、line count / doc comment length cap 禁止を確認する。

subagent review では Pasteur が Required として、store-local identity と serialized artifact semantics の分離、identity field の限定、proof kind 除外、same key / policy / fingerprint で proof kind が違っても `StableDuplicate` になる source-policy case、古い duplicate helper signature から typed identity helper shape への source policy 更新を求めた。Archimedes は実装 / test を approve し、Required として note / issue の durable checkpoint 追記を求め、Non-blocker として identity struct に `proof_kind` / `record_index` / TypeId 系 field が混入する退行を scoped negative regex で落とす案を挙げた。今回の実装と記録更新はすべて同じ slice 内で反映した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_store_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl --no-tree -j 1 --assert-io -o tmp/selfhost-memo-trait-proof-store-stable-identity-focused.json`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer と proof store preseed、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。

## 2026-06-12 selfhost memo trait proof store stable duplicate checkpoint

`memo_trait_proof_store.nepl` の stable proof push に duplicate rejection を追加した。

stable proof artifact 由来の record は、session-local first-wins ではなく、永続入力として同じ proof identity が重複した時点で fail-closed にする。identity は stable fingerprint 単独ではなく、cross-arena canonical key equality、solver policy equality、record stable fingerprint equality の組で判定する。proof kind は duplicate identity に含めない。同じ key / policy / fingerprint に key-only proof と key-and-value proof が共存すると、lookup の first-wins により片方が隠れ、永続 artifact の意味が挿入順に依存するためである。

`SelfhostMemoTraitProofStorePushErrorKind::StableDuplicate` を追加し、stage0 smoke は同じ stable proof を2回 push した時に2回目が `StableDuplicate` になることを確認する。duplicate rejection は record / stable index append の前に実行し、失敗経路では `records`、`stable_index`、projection 済み `next_key_arena` を閉じる。session-only compatibility path は `stable_fingerprint = none` のままにして、既存の store 内 lookup 互換性を維持した。

subagent review では Mencius が Blocker / Required なしで approve した。Mencius は、duplicate 判定が fingerprint-only ではなく canonical equality / policy / stable fingerprint を通すこと、proof kind を identity から外す判断が first-wins ambiguity を避けること、duplicate path の owner cleanup が complete であること、source policy が typed `StableDuplicate`、append 前拒否、cleanup、public stable push exercise を固定していることを確認した。

source policy は `nodesrc/test_selfhost_memo_trait_proof_store_contract.js` を更新し、typed push error、stable duplicate helper の判定条件、stable push の append 前拒否、owner cleanup、stage0 duplicate exercise、line count / doc comment length cap の禁止を固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_trait_proof_store_contract.js`
- pass: `node nodesrc/run_doctest.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl -n 1`
- pass: `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl --no-tree -j 1 --assert-io -o tmp/selfhost-memo-trait-proof-store-stable-duplicate-focused.json`
- pass: `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- pass: `node nodesrc/run_source_policy_regressions.js --warn-only`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`

この checkpoint 後の残件は、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer と proof store preseed、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続することである。
