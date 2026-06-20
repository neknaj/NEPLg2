---
id: ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7
title: "memoized function values need backend representation and identity-observation ban"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-21
target: "nepl-core/src/codegen; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/effect_check.rs"
---

# ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7: memoized function values need backend representation and identity-observation ban

## 概要

Existing backend function values are lowered as table indices or i32-like ids and do not carry private cache environment state, while memo_call returns a function value with hidden private cache storage.

## 対象

- `nepl-core/src/codegen; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/effect_check.rs`

## 根拠

- 未記入

## 問題

Existing backend function values are lowered as table indices or i32-like ids and do not carry private cache environment state, while memo_call returns a function value with hidden private cache storage.

## 影響

Without a backend representation and identity-observation ban, memoized function values can either be impossible to lower or can leak closure/cache allocation identity through equality, hash, raw store/load, cast, layout query, or debug output.

## 修正方針

Choose a Phase 1 representation for memoized functions, such as compiler-generated wrappers with hidden private cache regions or a closure object with sealed identity, and forbid pure public APIs that observe function address, closure allocation id, cache region id, equality, hash, or raw representation.

## 検証

Regression tests should accept calling a memoized pure named function and reject identity/hash/address/cast/raw-store observation, function-value key usage, public cache field exposure, and backend paths that require an unsealed closure id.

## 2026-06-01 checkpoint

HIR の `MemoizedFunctionValue` を Resource IR lowering で plain `FunctionValue` と同化しないようにした。`ResourceOp::FunctionValue` は `ResourceFunctionValueKind::{Plain, Memoized}` を持つ。

現時点の backend codegen は、sealed private cache backend が未実装であるため、`MemoizedFunctionValue` を既存の function table value と同じ可観測結果へ lower する。ただし Resource IR と body hash では memoized kind を保持するため、Resource proof cache と将来 backend 実装は plain `@func` と `memo_call @func` を区別できる。

検証:

- `cargo test -p nepl-core function_memo_call --test functions -- --nocapture`
- `cargo test -p nepl-core resource_function_body_hash_tracks_memoized_function_value_kind --lib -- --nocapture`

残件:

- memoized function value の sealed backend representation。
- function identity equality / hash / raw address / debug observation の禁止を backend と typecheck へ明示接続すること。
- `memo_call @pure_named_func` の呼び出し実行時に private cache を実際に利用すること。

## 2026-06-01 function alias kind checkpoint

`FunctionAliasTable` は `FunctionValueIdentity` だけでなく `ResourceFunctionValueKind` も
運ぶようになった。これにより、同じ underlying function identity を持つ plain function value
と memoized function value が、copy、aggregate field、branch merge、match merge、indirect call
候補伝播で同一候補として dedupe されない。

既存の indirect call summary consumer はまだ function value kind を解釈せず、underlying
function symbol で borrow / effect / owner / initialized / collection-slot summary を引く。
そのため、plain と memoized が同じ symbol を指す場合は、summary 適用前に symbol を重複排除する。
これは現行 backend が memoized value を plain function table value と同じ可観測結果へ lower する
段階の互換境界であり、memoized kind を捨てるものではない。

この checkpoint は sealed backend representation そのものではない。目的は、今後 private
cache region identity や sealed wrapper identity を function value alias に載せる前提として、
既存の Resource IR 解析が memoized kind を落とさない運搬面を固定することである。

検証:

- `cargo test -p nepl-core function_alias --lib -- --nocapture`

## 2026-06-01 sealed memo cache proof dependency

sealed backend representation は
`ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7` の proof を下流依存にする。

backend が private cache storage を実際に持つ前に、sealed region が public value、raw address、
function equality/hash/debug observation、cache stats/clear/ref API へ出ないことを Resource IR 側で
証明する。`MemoizedFunctionValue` を plain function table value と同じ可観測結果へ lower している
現 checkpoint は、sealed representation 完了ではなく fail-closed な足場として扱う。

## 2026-06-15 selfhost backend request manifest checkpoint

selfhost 側に `stdlib/neplg2/core/codegen/memo_call_backend_request.nepl` を追加し、HIR `MemoizedFunctionValue` を codegen / backend が読む typed request manifest へ変換する境界を作った。

この checkpoint は sealed private cache backend representation そのものではない。private cache allocation、hit / miss、cache region identity、Resource IR `PrivateCache` proof、prechecked artifact、Wasm / LLVM bytes は作らない。目的は、backend 入力で `FnValue` と `MemoizedFunctionValue` を同化しない typed boundary を先に固定することである。

accepted input は `SelfhostHirExprPayload::MemoizedFunctionValue(identity)` だけである。`FnValue` は `FnValueUnsupported`、`Call` は `CallUnsupported`、その他 non-memo payload は enum error で fail-closed にする。identity は `DefId` あり、monomorphic、`SelfhostEffectKind::Pure` でなければならず、`SelfhostHirExpr.ty` と identity の `function_ty` も一致しなければならない。

request は `request_kind`、`source_function_def_id`、`function_ty`、`source_effect`、`type_arg_count` を authority field として保持する。`diagnostic_symbol` と `diagnostic_span` は診断用 metadata であり、accepted 判定、cache namespace、proof authority、永続 artifact key には使わない。`DefId` / `TypeId` / `Span` は session-local なので、`.neplobj` / `.neplproof` / `.neplmeta` の key にするには canonical type key、public surface hash、policy hash、stable source identity へ別途投影する必要がある。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_request_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_request.nepl --no-tree -j 1 --dist web/dist --assert-io --timeout-nonfatal -o tmp/selfhost-memo-call-backend-request.json`

残件:

- memoized function value の sealed backend representation。
- PrivateCache / PrivateState region と Resource no-escape proof。
- function identity equality / hash / raw address / debug observation の禁止。
- `MemoKey` / `MemoValue` aggregate proof と backend request の接続。
- `.neplobj` / prechecked artifact 用 stable request key への投影。

## 2026-06-15 selfhost backend request table checkpoint

selfhost 側に `stdlib/neplg2/core/codegen/memo_call_backend_request_table.nepl` を追加し、borrowed HIR module の root expression subtree から memoized backend request manifest を owner table へ集める境界を作った。

この checkpoint も sealed private cache backend representation そのものではない。private cache allocation、cache hit / miss、cache region identity、Resource IR `PrivateCache` proof、prechecked artifact、Wasm / LLVM bytes は作らない。目的は、backend 入力で `MemoizedFunctionValue` leaf を見落とさず、同じ `DefId` を指す複数 occurrence を後続 materialization が区別できる typed stream にすることである。

table entry は `memoized_expr_id` と `SelfhostMemoCallBackendRequest` を保持する。`memoized_expr_id` は session-local occurrence metadata であり、永続 artifact key や diagnostic span の代替ではない。collector は `MemoizedFunctionValue` branch だけで request builder を呼び、通常の non-memo leaf は無視する。HIR `Error` payload、root expression 欠落、child range 不正、child id 欠落、child expression 欠落、memo request rejection、fuel exhaustion は typed enum error で fail-closed にする。

child range は iteration 前に `selfhost_hir_child_range_new_bounded_result` で module child table 長に対して検証する。traversal fuel は深さではなく訪問 expression 総数の予算として sibling 間で thread する。push は private API とし、push 失敗では `Vec` が返した owner を閉じて `RequestPushFailed` に `StdErrorKind` を残す。

subagent review では、request table entry に occurrence identity が必要であること、HIR `Error` を通常 non-memo として無視してはいけないこと、child range を事前検証すること、fuel を global traversal budget とすること、push を外部公開しないこと、失敗時の partial owner cleanup を source policy で固定することが blocker / required として指摘された。実装では、`SelfhostMemoCallBackendRequestTableEntry`、`SelfhostMemoCallBackendRequestTraversalState`、`SelfhostMemoCallBackendRequestRejection`、bounded child range validation、root / child missing の別 error、private push、source policy regression を追加して対応した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_request_table_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_request_table.nepl --no-tree -j 1 --dist web/dist --assert-io --timeout-nonfatal -o tmp/selfhost-memo-call-backend-request-table.json`

残件:

- sealed memoized backend representation。
- PrivateCache / PrivateState effect masking と Resource no-escape proof。
- function identity equality / hash / raw address / debug observation の禁止。
- `MemoKey` / `MemoValue` aggregate proof と request stream の接続。
- `.neplobj` / prechecked artifact 用 stable request key への投影。
- request table の sorted index 化、stream compaction、explicit stack traversal、stage0 fixture 分割。

## 2026-06-15 selfhost backend preflight checkpoint

selfhost 側に `stdlib/neplg2/core/codegen/memo_call_backend_preflight.nepl` を追加し、memoized backend request stream を backend accepted path へ進める前に、HIR root から再収集・再照合する fail-closed preflight 境界を作った。

この checkpoint は sealed private cache backend representation そのものではない。目的は、`SelfhostMemoCallBackendRequestTable` が public struct であることを踏まえ、caller supplied table を authority にする API を作らず、borrowed `SelfhostHirModule` と root `SelfhostHirExprId` から内部で request table を構築して検査することである。

preflight は各 entry の `memoized_expr_id` から HIR expression を引き直し、`selfhost_memo_call_backend_request_from_hir_expr_result` を再実行する。再構築 request と table entry は `request_kind`、`source_function_def_id`、`function_ty`、`source_effect`、`type_arg_count` で照合する。`diagnostic_symbol`、`diagnostic_span`、関数名、`"memo_call"` 文字列は authority にしない。

request が 0 件なら backend materialization は不要として `SelfhostMemoCallBackendPreflightSummary` を返す。request が 1 件以上ある場合、PrivateCache / PrivateState effect masking、Resource no-escape proof、stable artifact key projection が未接続であるため、`PrivateCacheProofUnavailable(expr_id)` で fail-closed にする。subagent review で指摘された `ProofDeferred` を実行可能 plan に混ぜる危険を避けるため、現段階では accepted backend plan table を作らない。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_preflight_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_preflight.nepl --no-tree -j 1 --dist web/dist --assert-io --timeout-nonfatal -o tmp/selfhost-memo-call-backend-preflight.json`

残件:

- sealed memoized backend representation。
- PrivateCache / PrivateState effect masking と Resource no-escape proof。
- function identity equality / hash / raw address / debug observation の禁止。
- `MemoKey` / `MemoValue` aggregate proof と request stream / preflight の接続。
- `.neplobj` / prechecked artifact 用 stable request key への投影。

## 2026-06-15 selfhost backend private-cache request-evidence gate checkpoint

selfhost 側に `stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` を追加し、memoized backend request と request occurrence evidence を照合する gate を作った。

この checkpoint は sealed private cache backend representation そのものではない。private cache allocation、cache lookup / insert、cache region identity、PrivateCache / PrivateState surface fold、Resource no-escape proof producer、Wasm / LLVM bytes、永続 `.neplobj` / `.neplproof` artifact は作らない。目的は、backend accepted path へ進む前に、現在の HIR root で実際に収集された memoized request occurrence と proof record が一致していることだけを fail-closed に確認することである。

accepted gate 本体は module-private である。内部 entrypoint は borrowed `SelfhostHirModule`、root `SelfhostHirExprId`、traversal fuel、`body_module_fingerprint`、borrowed proof table だけを受け取る。caller supplied request table は受け取らず、内部で request table を構築する。構築後、各 entry の `memoized_expr_id` から HIR expression を引き直し、request builder を再実行して、`request_kind`、`source_function_def_id`、`function_ty`、`source_effect`、`type_arg_count` を再照合する。

proof key は `memoized_expr_id`、`source_function_def_id`、`function_ty`、`root_expr_id`、`body_module_fingerprint`、`request_kind`、`source_effect`、`type_arg_count`、`proof_kind`、`proof_schema_version` を持つ。`proof_kind` は現段階では `RequestOccurrenceGateEvidence` だけであり、後続の Resource no-escape proof や identity observation ban proof と混同しないための field である。`RequestEvidenceProven` は request occurrence evidence が一致したことだけを表し、PrivateCache proof 全体の完了を意味しない。

proof record、proof table、proof table push、accepted gate 本体は module-private にした。NEPL の public struct は constructor / field payload の直組み可能性を持つため、push だけを private にしても producer-owned 契約にはならない。missing proof、`RequestEvidenceRefuted`、duplicate proof、current root の request に対応しない orphan proof は enum error として fail-closed にする。成功時も executable backend plan は作らず、non-executable summary だけを返す。

subagent review では、status 名が広すぎると PrivateCache proof 完了と誤読されること、public push と public proof table / record constructor / table field が trust bypass になること、proof key に request kind / source effect / type arg count / proof kind / schema version が必要なこと、orphan proof を拒否すべきことが blocker / required として指摘された。実装では `RequestEvidenceProven` / `RequestEvidenceRefuted`、private proof record、private proof table、private writer、private accepted gate、orphan rejection、source policy regression を追加して対応した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp_memo_gate_tests.json`

残件:

- sealed memoized backend representation。
- producer-owned Resource proof boundary。
- PrivateCache / PrivateState effect masking と Resource no-escape proof。
- function identity equality / hash / raw address / debug observation の禁止。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- proof lookup の sorted index 化、root / fingerprint bucket 化、stage0 fixture 分割。

## 2026-06-20 selfhost memo_call backend Resource observation producer stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、Resource observation 由来の private-cache proof status を既存 request-evidence proof gate へ流す module-private producer stage0 を追加した。

この checkpoint は sealed private cache backend representation そのものではない。actual Resource IR graph walker、PrivateCache / PrivateState surface fold、cache allocation、cache lookup / insert、cache region identity、Wasm / LLVM bytes、永続 `.neplobj` / `.neplproof` artifact はまだ作らない。目的は、Resource 側の observation を request-evidence gate へ接続する直前の status fold を typed enum / Result で固定し、未証明や未判定を成功 path に混ぜないことである。

Resource status は `PrivateCacheNoEscapeProven`、`PrivateCacheMayEscape`、`PrivateCacheMissing`、`PrivateCacheUnknown` に分けた。`PrivateCacheNoEscapeProven` だけを `RequestEvidenceProven` へ変換し、`PrivateCacheMayEscape` は `RequestEvidenceRefuted` へ変換して既存 gate の `ProofRefuted` として止める。`PrivateCacheMissing` と `PrivateCacheUnknown` は `ResourceProofMissing` / `ResourceProofUnknown` として producer error にし、request-evidence proof table には保存しない。

Resource proof record、Resource proof table、Resource proof table writer、Resource proof gate は module-private にした。外部 module が forged Resource observation table を public API に渡して accepted path を作る経路は公開していない。producer は Resource observation table を module-private request-evidence proof table へ変換してから既存 gate を呼び、既存 gate は引き続き HIR root から request table を内部再構築して request entry を HIR payload と再照合する。

subagent review では、Resource proof producer が public key / public request table を authority として受け取ると forged evidence になること、request occurrence evidence と Resource no-escape proof completion を混同しないこと、Missing / Unknown / Refuted を `Ok` にしないこと、known function 名だけではなく private table / record / status type が public signature に漏れること自体を source policy で禁止することが required として指摘された。実装では private Resource proof table / record / status、fail-closed status fold、private request-evidence table conversion、public private-type exposure ban、stage0 Resource producer smoke を追加して対応した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-call-backend-private-cache-resource-proof-producer.json`

残件:

- actual Resource IR graph walker から `SelfhostMemoCallBackendPrivateCacheResourceProofTable` を作る境界。
- fresh private cache region / non-escape proof と PrivateCache / PrivateState effect masking。
- cache hit / miss / size / clear / raw identity observation ban。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate / Resource producer の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- Resource proof table lookup の sorted index 化、root / fingerprint bucket 化、stage0 fixture 分割。

## 2026-06-20 selfhost memo_call backend Resource graph producer stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、memo_call backend 用の Resource graph producer stage0 を追加した。typed body / place / edge input を module-private graph input owner に閉じ、preflight で構造を検査したうえで private-cache Resource observation へ畳む。

この checkpoint は actual Resource IR graph walker 本体ではない。目的は、walker が将来返す graph input contract を固定し、closed graph 以外の情報や endpoint が欠けた edge を no-escape proof に使わないことである。`SelfhostMemoCallBackendPrivateCacheResourceGraphInput`、body / place / edge record、graph id、place id、place kind、edge kind、fold summary は module-private であり、public function signature や public struct / enum に出さない。

preflight は、body module fingerprint placeholder、invalid graph id、invalid place id、invalid operation ordinal、duplicate body、duplicate place、duplicate edge、missing body、non-closed graph event、edge endpoint missing を enum error として拒否する。fold は `ClosedForPrivateCacheBoundary` の graph だけを使い、private storage / entry / owned clone は `PrivateCacheNoEscapeProven`、reference return / public store / external handle は `PrivateCacheMayEscape`、unsupported place / unsupported call boundary は `PrivateCacheUnknown` に畳む。closed graph でも private-cache place が 1 つもない場合は `PrivateCacheUnknown` とし、空 graph から proof を合成しない。

graph producer は graph input を module-private Resource proof table へ変換し、その後で既存 Resource observation producer / request-evidence gate を呼ぶ。既存 gate は引き続き HIR root から request table を内部再構築して request entry を HIR payload と再照合するため、Resource graph input は caller supplied request table の代替 authority にならない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。graph input payload / owner / fold type が public signature に露出しないこと、GraphInput owner が Clone / Copy にならないこと、preflight が body / place / edge を検査すること、closed empty graph を Unknown にすること、Missing / Unknown / MayEscape を accepted path に混ぜないこと、行数や doc comment 量の制限を追加しないことを固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-call-backend-private-cache-resource-graph-producer.json`

残件:

- actual Resource IR graph walker から body / place / edge event を生成する境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- cache hit / miss / size / clear / raw identity observation ban。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate / Resource producer / graph producer の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- graph lookup index 化、walker event operation ordinal index 化、stage0 fixture 分割。

## 2026-06-21 selfhost memo_call backend Resource walker input scanner stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、future actual Resource IR walker が返す typed event stream を module-private graph input へ正規化する scanner stage0 を追加した。

この checkpoint は actual Resource IR walker 本体ではない。body event、place event、edge event、unsupported event を別 table として受け、scanner が body / place / edge / unsupported の構造を検査したうえで既存 `SelfhostMemoCallBackendPrivateCacheResourceGraphInput` に写す境界である。walker input owner と event payload は module-private のままにし、public API から forged event stream を accepted path へ渡す経路は作らない。

scanner preflight は body module fingerprint placeholder、invalid graph id、invalid place id、invalid operation ordinal、duplicate operation ordinal、missing body event、non-closed graph event を typed enum error として拒否する。operation ordinal は place / edge / unsupported の cross-kind で重複を拒否する。unsupported event は `SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason` の typed reason を保持し、対応 body を `TraversalUnsupported` graph body へ変換する。unsupported body に属する place / edge event は GraphInput へ渡さない。

stage0 smoke は accepted、MayEscape、missing、unsupported、duplicate ordinal、missing body event、placeholder fingerprint を確認する。accepted stream は既存 request-evidence gate の non-executable summary まで到達する。MayEscape / missing / unsupported は既存 graph producer / Resource producer / request-evidence gate の typed fail-closed path へ流れ、duplicate ordinal / missing body / placeholder は scanner error として止まる。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。walker event payload / owner が public signature に露出しないこと、WalkerInput owner が Clone / Copy にならないこと、unsupported reason が typed enum であること、scanner が body / place / edge / unsupported を検査すること、unsupported override と place / edge skip、stage0 の7分岐、行数や doc comment 量の制限を追加しないことを固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-call-backend-private-cache-walker-scanner.json`

残件:

- actual Resource IR graph walker 本体から typed body / place / edge / unsupported event stream を生成する境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- cache hit / miss / size / clear / raw identity observation ban。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate / Resource producer / graph producer / scanner の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- graph lookup index 化、walker event operation ordinal index 化、stage0 fixture 分割。

## 2026-06-21 selfhost memo_call backend Resource walker producer bridge stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、HIR root から request table を内部再構築し、producer-owned な private walker event stream を作って既存 scanner / graph gate へ渡す bridge stage0 を追加した。

この checkpoint は actual Resource IR walker 本体ではない。目的は、stage0 fixture ではなく HIR root 由来の request authority から private walker input を作る経路を固定し、public API から forged walker event stream や GraphInput を accepted path へ渡せない状態を維持することである。request table は `selfhost_memo_call_backend_request_table_from_hir_root_result` で内部収集し、各 entry は既存 proof gate と同じ HIR payload recheck と proof key construction を通す。

現段階では actual Resource traversal が private-cache place / edge / identity observation をまだ生成できない。そのため bridge は `PrivateCacheNoEscapeProven` や accepted `PrivateCacheStorage` / `CloneOutOwnedValue` graph を合成せず、request occurrence ごとに closed body header と `UnknownResourceOperation` の typed unsupported event を作る。scanner はこの unsupported event を `TraversalUnsupported` graph body へ写し、graph producer / Resource producer / request-evidence gate は最終的に `ResourceProofUnknown` として fail-closed に拒否する。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。producer bridge error taxonomy、error code helper の wildcard 禁止、HIR root からの request table 内部構築、request recheck、proof key construction、scanner 経由、GraphInput cleanup、unsupported traversal の Unknown rejection、placeholder fingerprint rejection、bridge internals の public API 化禁止を固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`

残件:

- actual Resource IR graph walker 本体から typed body / place / edge / unsupported event stream を生成する境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- actual Resource IR walker が cache hit / miss / size / clear / debug、function identity、raw identity observation を検出して typed observation stream へ出す境界。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate / Resource producer / graph producer / scanner / producer bridge の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- graph lookup index 化、walker event operation ordinal index 化、producer bridge request/key bucket 化、stage0 fixture 分割。

## 2026-06-21 selfhost memo_call backend Resource observation ban classifier stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、memoized function の pure 化を壊す可観測操作を typed observation kind として分類し、Resource walker の unsupported event へ畳む observation ban stage0 を追加した。

この checkpoint は actual Resource IR walker 本体ではない。目的は、actual walker が将来検出する cache observation / function identity observation / raw identity observation を、NoEscape proof とは別の fail-closed stream として扱う境界を固定することである。`NoObservationDetected` は 1 record の中立状態であり、body 全体の無観測証明や `PrivateCacheNoEscapeProven` には変換しない。`ObservationDetected(kind)` だけが typed unsupported event を作る。

分類は enum と exhaustive match で行う。cache hit / miss / size / stats / clear / debug / cache region identity は `CacheObservationUnsupported`、function equality / hash / debug / closure allocation identity は `FunctionIdentityObservationUnsupported`、raw identity / raw representation は `RawIdentityObservationUnsupported`、unsupported observation は `UnknownResourceOperation` へ畳む。wildcard fallback は使わない。

observation kind / status / record / table は module-private である。public API には private observation table を受け取る accepted path を作らず、stage0 summary と typed result helper だけを公開する。gate は HIR root から request table を内部再構築し、HIR payload recheck と proof key construction を通したうえで observation record を scanner / graph gate へ渡す。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io --shard 5/6 -o tmp/selfhost-memo-call-backend-private-cache-observation-ban-shard5.json`

残件:

- actual Resource IR walker 本体から typed body / place / edge / unsupported event stream を生成する境界。
- actual Resource IR walker が cache hit / miss / size / stats / clear / debug、function identity、raw identity observation を検出して observation table へ出す境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate / Resource producer / graph producer / scanner / producer bridge / observation ban gate の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- observation table request/key bucket 化、graph lookup index 化、walker event operation ordinal index 化、stage0 fixture 分割。

## 2026-06-21 selfhost memo_call backend actual walker event normalizer stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、future actual Resource IR walker が返す単一の unified event stream を既存の graph-side `ResourceWalkerInput` と observation-side `ObservationBanTable` へ分配する normalizer stage0 を追加した。

この checkpoint は actual Resource IR walker 本体ではない。目的は、actual traversal が将来 body / place / edge / unsupported / observation を 1 本の typed stream として返したときに、それを GraphInput や proof table へ直接渡さず、既存 scanner / graph gate / observation ban gate へ必ず通す境界を固定することである。

normalizer は body / place / edge / unsupported payload を `SelfhostMemoCallBackendPrivateCacheResourceWalkerInput` へ写し、detected observation だけを `SelfhostMemoCallBackendPrivateCacheObservationBanTable` へ写す。`NoObservationDetected` は body 全体の無観測証明ではないため table に追加せず、成功 proof や accepted graph へ変換しない。detected observation が 1 件でもある場合は graph path より observation ban gate を優先し、観測禁止操作が graph proof 成功で隠れないようにした。

unified event payload / table / split output は module-private である。public API には private unified stream や split output を受け取る accepted path を作らず、stage0 summary と typed result helper だけを公開した。split の失敗時は、walker input push が失敗した場合には observation table だけを閉じ、observation table push が失敗した場合には walker input だけを閉じるように、owner cleanup の責務を push 関数の消費契約に合わせて分けた。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。unified event payload / table / split output が public signature に露出しないこと、owner table と split output が Clone / Copy にならないこと、split loop と error helper が wildcard fallback を使わないこと、graph-side event は既存 walker input push へ、observation event は既存 observation ban table / gate へ流すこと、normalizer が `PrivateCacheNoEscapeProven` / `PrivateCacheStorage` / `CloneOutOwnedValue` / GraphInput / proof table record を合成しないことを固定した。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-call-backend-private-cache-actual-walker-normalizer-full.json`

残件:

- actual Resource IR walker 本体が unified event stream を生成する境界。
- actual Resource IR walker が cache hit / miss / size / stats / clear / debug、function identity、raw identity observation を検出して unified stream へ出す境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と request stream / proof gate / Resource producer / graph producer / scanner / producer bridge / observation ban gate / unified normalizer の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- observation table request/key bucket 化、graph lookup index 化、walker event operation ordinal index 化、unified event stream index 化、stage0 fixture 分割。

## 2026-06-21 selfhost memo_call backend actual walker event producer bridge stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、HIR root から request authority を内部再収集し、producer-owned unified event stream を作って既存 actual walker event normalizer へ渡す producer bridge stage0 を追加した。

この checkpoint は actual Resource IR walker 本体ではない。目的は、actual traversal が未接続の段階でも、caller supplied request table、forged unified event table、direct GraphInput を authority にせず、HIR root 由来 request を recheck / proof key construction へ通してから unified stream へ写す境界を固定することである。

bridge は request ごとに closed body event と `UnknownResourceOperation` unsupported event だけを生成する。`PrivateCacheNoEscapeProven`、`PrivateCacheStorage`、`CloneOutOwnedValue`、GraphInput、proof table record は合成しない。stage0 observation fixture は module-private helper で detected observation を unified stream に混ぜ、normalizer の observation precedence が graph path を上書きしないことを確認する。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。producer bridge error taxonomy、wildcard なしの error helper、HIR-root request authority、request recheck、proof key construction、body / unsupported unified event のみの生成、normalizer 経由、normalizer bypass 禁止、accepted proof / graph payload / proof table record 合成禁止、private bridge internals の public API 化禁止を固定した。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io --shard 8/8 -o tmp/selfhost-memo-call-backend-private-cache-actual-walker-producer-bridge-shard8.json`
- timeout: `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --no-tree -j 1 --dist web/dist --assert-io -o tmp/selfhost-memo-call-backend-private-cache-actual-walker-producer-bridge-full.json` は 8 件中 7 件 pass 後、新規 doctest が compile timeout。shard 実行では pass しており、semantic failure ではなく大型 stage0 module の Resource 検査時間が支配している。

残件:

- actual Resource IR walker 本体が request / body から unified body / place / edge / unsupported / observation event stream を生成する境界。
- actual Resource IR walker が cache hit / miss / size / stats / clear / debug、function identity、raw identity observation を検出して unified stream へ出す境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- stage0 fixture 分割、initialized-state 探索削減、request/key bucket 化、graph lookup index 化、walker event operation ordinal index 化、unified event stream index 化。

## 2026-06-21 selfhost memo_call backend actual walker operation classifier stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、actual Resource IR walker が将来出す operation 相当の record を typed enum で分類し、既存 actual walker event normalizer へ渡す operation classifier stage0 を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。ここで固定したのは、operation kind / record / table の module-private owner boundary、HIR root 由来 request authority、request entry と proof key の再照合、operation record から unified event stream への typed mapping、既存 normalizer への委譲である。

分類は wildcard fallback を使わず、closed private cache storage / clone-out owned value を graph accepted 側の event、return cache reference / public store を MayEscape 側の event、未知 operation を `UnknownResourceOperation` unsupported event、cache hit / function identity / raw identity を observation event へ写す。operation table は public API に出さず Clone / Copy にしない。classifier は scanner / graph gate / observation gate を直接呼ばず、GraphInput、proof table record、sealed backend bytes、`PrivateCacheNoEscapeProven` を合成しない。

stage0 public summary は synthetic closed clone path、escape path、unknown operation path、observation path、placeholder proof key path を返す。accepted path は「operation classifier が既存 graph gate と normalizer に接続される」ことの smoke であり、fresh private cache region proof、PrivateCache / PrivateState effect masking、sealed memoized backend representation、artifact replay の完了ではない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。operation vocabulary、operation record authority、operation table owner exposure、wildcard-free classification、normalizer bypass 禁止、proof / backend 合成禁止を固定している。追加 doctest は巨大 module の compile timeout を悪化させたため、この checkpoint では source policy と既存 selfhost shard runner で contract を検査し、stage0 fixture 分割を後続最適化として残した。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist --shard 8/8 -o tmp/selfhost-memo-call-backend-private-cache-operation-classifier-selfhost-shard8of8.json`

残件:

- actual Resource IR traversal 本体が typed operation record または unified event stream を生成する境界。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と operation classifier / producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- operation table request/key bucket 化、graph id index 化、stage0 fixture 分割、initialized-state 探索削減。

## 2026-06-21 selfhost memo_call backend actual walker traversal source collector stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、既存 `SelfhostMemoCallBackendPrivateCacheResourceWalkerInput` と `SelfhostMemoCallBackendPrivateCacheObservationBanTable` から module-private traversal source table owner を作る collector stage0 を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。collector は borrowed walker input / observation table を検査し、place / edge / unsupported / observation を traversal source vocabulary へ写すだけである。GraphInput、proof table record、`PrivateCacheNoEscapeProven`、sealed backend bytes、Wasm / LLVM fragment、operation table は作らない。

`ResourceIrTraversalUnavailable` は actual traversal 未接続専用に戻した。known unsupported traversal / observation は `UnsupportedTraversalSource` / `UnsupportedObservationSource` として保持する。`ExternalHandle`、`UnsupportedPlace`、`CallBoundaryUnsupported` は unavailable に落とさず、`Owns` / `BorrowView` は `CloneOutOwnedValue` に偽装しない。`NoObservationDetected` は source record を生成しない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。place / edge / observation helper の wildcard fallback 禁止、ExternalHandle / Unsupported / UnsupportedObservation の unavailable 化禁止、Owns / BorrowView の clone-out 偽装禁止、NoObservationDetected の source なし、collector helper 非公開、GraphInput / proof / backend / operation table 合成禁止、line count / doc comment amount limiting checks の禁止を固定した。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist --shard 8/8 -o tmp/selfhost-memo-call-backend-private-cache-traversal-source-collector-selfhost-shard8of8.json`

残件:

- actual Resource IR traversal 本体が real Resource IR / HIR lowering result から typed walker input または traversal source table を生成する境界。
- actual traversal source と fresh private cache region proof の接続。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。

## 2026-06-21 selfhost memo_call backend actual walker traversal source projection stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、actual walker traversal source vocabulary を accepted / escaping / observation / unavailable の typed source variant へ広げ、source-to-operation projection 経由で既存 operation classifier / unified event normalizer へ渡す projection stage0 を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。目的は、actual traversal が将来返す source classification tag を operation table へ入れる前段の module-private vocabulary として固定し、operation classifier vocabulary への変換を wildcard-free match で閉じることである。`ResourceIrTraversalUnavailable` は引き続き `UnknownResourceOperation` へだけ写し、accepted proof にはしない。accepted / escaping / observation source は operation record へ写るだけであり、GraphInput、proof table record、`PrivateCacheNoEscapeProven`、sealed backend bytes、Wasm / LLVM fragment は合成しない。

stage0 public summary は private fixture source table を使い、accepted graph path、MayEscape path、unknown unsupported path、observation path、placeholder proof key path を確認する。これは source-to-operation projection と既存 classifier / normalizer の接続 smoke であり、operation producer bridge の HIR-root path が accepted source を生成したことを意味しない。operation producer bridge は引き続き request ごとに `ResourceIrTraversalUnavailable` source だけを emit し、actual traversal 未接続のまま no-escape proof を観測したように見せない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。traversal source vocabulary、source record authority、source table owner exposure 禁止、source table Clone / Copy 禁止、source-to-operation projection の wildcard fallback 禁止、projection fixture の owner cleanup、operation classifier 経由、producer bridge が accepted source を emit しないこと、proof / GraphInput / backend 合成禁止、line count / doc comment amount limiting checks の禁止を固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`

残件:

- actual Resource IR traversal 本体が real Resource IR / HIR lowering result から traversal source table へ accepted / escaping / observation / unavailable source を生成する境界。
- actual traversal source と fresh private cache region proof の接続。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- source / operation table request-key bucket 化、graph id index 化、stage0 fixture 分割、initialized-state 探索削減。

## 2026-06-21 selfhost memo_call backend actual walker operation producer bridge stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、HIR root 由来の request authority から producer-owned traversal source table を作り、それを operation table へ投影して既存 operation classifier / unified event normalizer へ渡す operation producer bridge stage0 を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。目的は、public caller が traversal source table や operation table を渡す accepted path を作らず、HIR root から request table を内部再構築し、request entry と proof key を再照合したうえで module-private traversal source table owner を作る境界を固定することである。

stage0 producer は request ごとに `ResourceIrTraversalUnavailable` source record だけを作る。source-to-operation projection はこの source を `UnknownResourceOperation` record にだけ変換する。`PrivateCacheStoragePlace`、`ReturnedOwnedClonePlace`、`CloneOutOwnedValueEdge` などの accepted 側 operation は出さない。accepted operation の smoke は既存 operation classifier stage0 の synthetic fixture に留め、producer bridge が actual traversal / fresh private region / clone-out safety を観測したように見せない。

producer bridge は operation table を作った後、必ず `selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_from_hir_root_result` を通す。scanner、graph gate、observation ban gate、unified normalizer を producer bridge から直接呼ばず、GraphInput、proof table record、`PrivateCacheNoEscapeProven`、sealed backend bytes、Wasm / LLVM fragment も合成しない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。HIR-root request authority、traversal source table / operation table owner cleanup、`ResourceIrTraversalUnavailable -> UnknownResourceOperation` だけの projection、classifier 経由、lower gate bypass 禁止、accepted proof / accepted operation / GraphInput / proof table / backend bytes 合成禁止、producer internals の public API 化禁止、line count / doc comment amount limiting checks の禁止を固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist --shard 8/8 -o tmp/selfhost-memo-call-backend-private-cache-operation-producer-bridge-selfhost-shard8of8.json`

残件:

- actual Resource IR traversal 本体が real Resource IR / HIR lowering result から typed operation record または unified event stream を生成する境界。
- actual traversal source table に real Resource IR traversal 由来の accepted / escaping / observation source を流す境界。
- actual traversal 由来の closed private cache storage / clone-out owned value / return reference / public store / observation operation の分類。
- fresh private cache region proof、PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と operation classifier / producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- operation table request/key bucket 化、graph id index 化、stage0 fixture 分割、initialized-state 探索削減。
