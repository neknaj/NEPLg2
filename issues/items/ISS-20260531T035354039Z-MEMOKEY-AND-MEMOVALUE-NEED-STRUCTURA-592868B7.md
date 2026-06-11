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
