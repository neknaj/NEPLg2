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

## 2026-06-21 selfhost memo_call backend collector-owned traversal bundle stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、既存 collector が作る traversal source table と matching fresh witness table を同じ bundle lifecycle に載せる collector-owned traversal bundle stage0 を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。actual Resource IR body、HIR lowering result、cache lookup / insert operation、effect operation はまだ読まない。accepted path は private `ResourceWalkerInput + ObservationBanTable` fixture から collector を経由して作る。production HIR-root path は引き続き `ResourceIrTraversalUnavailable` だけを emit し、accepted source や fresh witness を actual traversal 未接続のまま生成しない。

collector-owned helper は、walker input と observation table を owner として受け取り、既存 `selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result` で source table を作る。source collection の成功・失敗の両方で input / observation owner を閉じる。source collection が失敗した場合は witness table を作らず、`Stage0SourceRejected` で fail-closed にする。source table 作成後の witness 生成失敗では、既存 `actual_traversal_bundle_stage0_with_sources_result` が source owner を閉じる。

accepted fixture は `PrivateCacheStoragePlace` root ordinal `0` と `CloneOutOwnedValueEdge` support ordinal `1` を collector output から作り、matching witness ordinal `0/1` と照合する。unsupported source + matching witness、observation source + matching witness はどちらも fail-closed にする。

bundle gate は既存の `region_no_escape_candidate_from_table_result -> region_fresh_witness_request_evidence_gate_result` 経由に留める。direct ResourceProofTable push、request proof table push、GraphInput 合成、PrivateCache / PrivateState effect mask、sealed backend bytes、Wasm / LLVM fragment、`.neplobj` / `.neplproof` artifact key は作らない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。collector-owned helper の cleanup order、input fixture failure の `Stage0SourceRejected` 化、accepted fixture の root/support ordinal と witness ordinal の一致、unsupported / observation source の fail-closed、lower proof / backend / effect / artifact 合成禁止、public summary の Result payload 限定を固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-collector-owned-bundle-doctest-current.json`

残件:

- actual Resource IR traversal 本体が real Resource IR / HIR lowering result から typed traversal source table または walker input table を生成する境界。
- actual traversal 由来の fresh witness table を生成し、collector fixture ではなく producer-owned actual traversal bundle として request-evidence bridge へ接続する境界。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- operation table request/key bucket 化、graph id index 化、stage0 fixture 分割、initialized-state 探索削減。

## 2026-06-21 selfhost memo_call backend producer-owned unavailable traversal bundle stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、HIR-root request authority から producer-owned traversal source table を作り、actual traversal bundle lifecycle へ渡す producer-owned unavailable traversal bundle stage0 を追加した。

この checkpoint は actual Resource IR body traversal 本体ではない。現行 producer は request entry から `ResourceIrTraversalUnavailable` source だけを作る。key / graph / ordinal 形式として well-formed な witness を与えても、source 側に accepted root / support が無いため candidate extraction で fail-closed になる。

producer-owned helper は、`selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_traversal_sources_from_hir_root_result` を通して source table owner を作る。source 生成成功時は既存 `actual_traversal_bundle_stage0_with_sources_result` に渡し、source / witness cleanup は既存 bundle helper と bundle gate に委譲する。source 生成失敗時は `Stage0SourceRejected` に写す。

public summary は well-formed witness、missing witness、rejected witness の typed rejection payload だけを返す。accepted request count / proof count は持たせていない。bundle、source table、witness table、candidate、Resource proof table、request-evidence proof table は public API に出さない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。producer helper が HIR-root producer source を通ること、collector fixture / accepted projection fixture を使わないこと、direct proof push / GraphInput / backend / effect mask / sealed backend / artifact key 合成禁止、HIR-root production path が accepted source / witness を emit しないことを固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-producer-owned-unavailable-bundle-doctest-current.json`

残件:

- actual Resource IR traversal 本体から real HIR lowering result / Resource IR body を読み、accepted / escaping / observation / unsupported source を生成する boundary。
- actual traversal 由来の fresh witness table を生成し、producer-owned accepted actual traversal bundle として request-evidence bridge へ接続する boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation と prechecked artifact key projection。

## 2026-06-21 selfhost memo_call backend private cache region candidate stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、actual walker traversal source collector が作る source table を private cache region proof の入力候補へ分類する stage0 を追加した。

この checkpoint は fresh private region proof や no-escape proof の完了ではない。private cache storage source を `PrivateCacheRegionRootCandidate` として扱い、entry / returned owned value / internal edge / clone-out edge source を `PrivateCacheRegionSupportCandidate` として扱うだけであり、`PrivateCacheNoEscapeProven`、`RequestEvidenceProven`、PrivateCache / PrivateState effect mask、sealed backend representation、Wasm / LLVM fragment、`.neplobj` / `.neplproof` artifact key は合成しない。fold は root と support の両方を要求するため、entry-only、returned-value-only、owns-edge-only、clone-out-edge-only の table は accepted smoke にならない。

region proof input kind / input record / status / proof record / proof table は module-private に留めた。public API は stage0 summary だけを返し、caller supplied proof table や owner table を authority にしない。proof table owner には Clone / Copy を付けていない。

stage0 fold は wildcard fallback を使わず、candidate 以外を success に混ぜない。escape、observation、unsupported、unavailable、missing candidate、placeholder fingerprint はそれぞれ別 error に写す。source authority は proof key、graph id、source ordinal、place id、typed source kind に限定し、source text、display name、diagnostic label、backend representation、GraphInput、Resource proof object は authority にしない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。region proof input/status/record/table の public exposure 禁止、proof table owner Clone / Copy 禁止、source kind projection / status projection / fold の wildcard fallback 禁止、distinct rejection、proof/backend/effect mask 合成禁止、line count / doc comment amount limiting checks の禁止を固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist --shard 8/8 -o tmp/selfhost-memo-call-backend-private-cache-region-candidate-selfhost-shard8of8.json`

残件:

- actual Resource IR traversal 本体が real Resource IR / HIR lowering result から traversal source table を生成する境界。
- `PrivateCacheRegionRootCandidate` / `PrivateCacheRegionSupportCandidate` を fresh private cache region proof と no-escape proof へ進める checker-layer boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- source / operation / region proof table request-key bucket 化、graph id index 化、stage0 fixture 分割、initialized-state 探索削減。

## 2026-06-21 selfhost memo_call backend region no-escape candidate consistency checker checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、private cache region proof table を no-escape proof へ進める直前の consistency checker stage0 を追加した。

この checkpoint は `PrivateCacheNoEscapeProven` ではない。`PrivateCacheRegionRootCandidate` と `PrivateCacheRegionSupportCandidate` を、単一 request key / 単一 graph / root 1 件 / support 1 件 / unique ordinal / bad status なし、という candidate-only record へ畳むだけである。actual Resource IR traversal、fresh private region proof、PrivateCache / PrivateState effect masking、request-evidence proof table、Resource proof table、sealed backend representation、Wasm / LLVM fragment、`.neplobj` / `.neplproof` artifact key は合成しない。

checker は empty table、key mismatch、graph mismatch、root duplicate、support duplicate、operation ordinal duplicate、root/support 欠落、escape、observation、unsupported、unavailable、placeholder / malformed origin を typed enum error として fail-closed にする。複数 support を持つ本物の graph-shaped proof は後続の actual Resource IR traversal / graph proof boundary が担当し、この stage0 では誤って candidate を accepted Resource proof に昇格しないことを優先する。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。candidate status / candidate record の public exposure 禁止、candidate helper の public API 化禁止、wildcard fallback 禁止、Resource proof / request-evidence proof / GraphInput / backend bytes / effect mask 合成禁止、summary が Result payload だけを公開することを固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-region-candidate-doctest.json`

残件:

- actual Resource IR traversal 本体が real Resource IR / HIR lowering result から traversal source table へ accepted / escaping / observation / unsupported source を生成する境界。
- candidate consistency と fresh-region witness を実際の Resource IR traversal 由来の witness table へ接続する boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- source / operation / region proof table request-key bucket 化、graph id index 化、stage0 fixture 分割、initialized-state 探索削減。

## 2026-06-21 selfhost memo_call backend fresh region witness bridge checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、region no-escape candidate と fresh-region witness candidate を照合し、module-private `SelfhostMemoCallBackendPrivateCacheResourceProofTable` を 1 件だけ生成する bridge を追加した。

この checkpoint は request-evidence gate への接続ではない。`SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateRecord` は、単一 request key、単一 graph id、root operation ordinal、support operation ordinal を持つ candidate-only record であり、それ単体では `PrivateCacheNoEscapeProven` にしてはいけない。今回の bridge は、別 record として渡される fresh-region witness が同じ key / graph / root ordinal / support ordinal を持ち、status が `PrivateCacheRegionFreshWitnessCandidateAccepted` である場合だけ、private Resource proof table を作る。

`PrivateCacheRegionFreshWitnessMissing`、`PrivateCacheRegionFreshWitnessRejected`、`PrivateCacheRegionFreshWitnessUnavailable` は fail-closed にする。key mismatch、graph mismatch、root/support ordinal mismatch、root/support ordinal duplicate、duplicate witness、placeholder fingerprint、invalid graph id、invalid ordinal も typed enum error として拒否する。witness authority は proof key、graph id、root/support ordinal、typed status に限定し、source text、display name、diagnostic label、backend bytes、artifact key、GraphInput、Resource proof record を authority にしない。

この stage で作る `PrivateCacheNoEscapeProven` record は、module-private Resource proof table 生成の到達確認であり、public accepted path ではない。`resource_proof_gate_from_hir_root_result`、request-evidence proof table、`RequestEvidenceProven`、GraphInput、backend bytes、PrivateCache / PrivateState effect mask、Wasm / LLVM fragment、`.neplobj` / `.neplproof` artifact key は合成しない。source policy は fresh witness status / record / table の public exposure、owner table Clone / Copy、wildcard fallback、request-evidence gate 呼び出し、backend / effect / artifact 合成を禁止している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-fresh-witness-doctest.json`

残件:

- actual Resource IR traversal 本体から fresh-region witness table を作る boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation と prechecked artifact key projection。

## 2026-06-21 selfhost memo_call backend fresh witness request-evidence bridge checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、fresh-region witness bridge が作った module-private `SelfhostMemoCallBackendPrivateCacheResourceProofTable` を既存 request-evidence gate へ接続する bridge を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。目的は、candidate consistency checker と fresh-region witness の一致から得た Resource proof を、caller supplied request table や public proof table ではなく、HIR root から request table を内部再構築する既存 `selfhost_memo_call_backend_private_cache_resource_proof_gate_from_hir_root_result` へ渡すことである。既存 gate は request entry の HIR payload recheck と proof key construction を再実行するため、root / body module fingerprint / request kind / effect / type argument count が一致しない proof は accepted summary にならない。

bridge は fresh witness table owner を消費し、success / error のどちらでも閉じる。生成された Resource proof table も既存 gate 呼び出し後に必ず閉じる。`PrivateCacheRegionFreshWitnessMissing` と `PrivateCacheRegionFreshWitnessRejected` は Resource proof table 生成前に止まり、body fingerprint mismatch は既存 request-evidence gate の rejection として `RegionFreshWitnessResourceProofRejected` に包まれる。

public stage0 は request count / proof count と representative fail-closed `Result` payload だけを返す。成功は non-executable summary であり、actual Resource IR traversal、PrivateCache / PrivateState effect masking、sealed backend representation、cache lookup / insert、Wasm / LLVM fragment、`.neplobj` / `.neplproof` artifact key は作らない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。fresh-witness-only stage が request-evidence gate を呼ばないこと、新 request-evidence bridge stage だけが既存 Resource proof gate を呼ぶこと、低層 proof table writer / request-evidence converter bypass、backend / effect / artifact 合成、private helper の public API 化を禁止することを固定した。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-request-evidence-doctest.json`

残件:

- actual Resource IR traversal 本体から fresh-region witness table を作る boundary。
- actual traversal 由来の fresh witness と Resource proof / request-evidence bridge を上位 orchestration へ接続する boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation と prechecked artifact key projection。

## 2026-06-21 selfhost memo_call backend actual traversal bundle stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、将来の actual Resource IR traversal producer が返すべき最小 bundle contract を固定する stage0 を追加した。bundle は module-private `SelfhostMemoCallBackendPrivateCacheActualTraversalBundle` として、`traversal source table` owner と `fresh witness table` owner を保持する。

この checkpoint は actual Resource IR traversal 本体ではない。accepted source と matching fresh witness は stage0 fixture でのみ作る。HIR-root production path は引き続き `ResourceIrTraversalUnavailable` source だけを emit し、actual traversal 未接続のまま closed private cache storage や clone-out owned value を観測したように見せない。

bundle gate は source table から既存 region proof table producer を通し、source owner を閉じ、既存 no-escape candidate checker を通して candidate を作る。candidate extraction に失敗した場合は witness owner を閉じる。candidate extraction に成功した場合だけ witness owner を既存 fresh witness request-evidence bridge へ渡す。direct ResourceProofTable push、request proof table push、GraphInput、backend bytes、Pure mask、`.neplobj` / `.neplproof` artifact key は合成しない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。bundle owner の public exposure 禁止、Clone / Copy 禁止、cleanup order、accepted fixture の root/support ordinal と matching witness ordinal、unsupported source + matching witness の fail-closed、HIR-root production path が accepted source / witness を emit しないことを固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-actual-traversal-bundle-doctest.json`

残件:

- actual Resource IR traversal 本体から real HIR lowering result / Resource IR body を読み、typed traversal source table または walker input table を生成する boundary。
- actual traversal 由来の fresh witness table を生成し、bundle fixture ではなく producer-owned bundle として request-evidence bridge へ接続する boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation と prechecked artifact key projection。

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

## 2026-06-21 selfhost memo_call backend operation-classified traversal bundle stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、module-private operation descriptor を HIR-root request authority、operation classifier、unified event split、collector-owned bundle、既存 request-evidence bridge へ接続する operation-classified traversal bundle stage0 を追加した。

この checkpoint は actual Resource IR body traversal 本体ではない。accepted source と matching fresh witness は private operation descriptor fixture から作る。production HIR-root path は引き続き `ResourceIrTraversalUnavailable` source だけを emit し、actual traversal 未接続のまま accepted private cache storage や clone-out owned value を観測したように見せない。

新 helper は `actual_walker_operation_classifier_events_from_hir_root_result -> actual_walker_event_split_result -> collector_owned_traversal_bundle_with_owners_result` の順に通る。operation table owner は classifier success / classifier error のどちらでも閉じる。classifier success 後の unified event table owner は split helper へ渡し、split success 後の walker input / observation table owner は collector-owned bundle helper へ渡す。split helper は owner pair を二重 free しない。classifier error は `Stage0SourceRejected e`、split error は `Stage0SourceRejected (NormalizerRejected e)` として区別する。

module fixture 作成失敗時は、作成済み operation table owner を閉じてから `Stage0FixtureAllocFailed` へ写す。operation table builder そのものの失敗は accepted proof へ進めず `Stage0SourceRejected` に写す。

public stage0 summary は accepted request/proof count と representative fail-closed `Result` payload だけを公開する。operation table、event table、split output、walker input、observation table、source table、fresh witness table、bundle、candidate、Resource proof table、request-evidence proof table は public API に出さない。GraphInput、direct proof push、request proof push、PrivateCache / PrivateState effect mask、sealed backend bytes、Wasm / LLVM fragment、`.neplobj` / `.neplproof` artifact key は作らない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。operation-classified helper が classifier / split / collector の順序を守ること、operation owner cleanup、classifier error と normalizer error の写し分け、accepted / escaping / observation / unsupported fixture path、public internals 禁止、lower gate bypass / accepted source fixture bypass / backend / effect / artifact 合成禁止を固定している。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-operation-classified-bundle-doctest-current.json`

subagent review:

- Wegener implementation review は `REVIEW_APPROVED`。classifier error と split normalizer error の写し分け、operation owner cleanup、split success 後の owner 移譲、source policy、actual Resource IR body traversal 未接続の明記について blocking issue は無い。

残件:

- actual Resource IR traversal 本体から real HIR lowering result / Resource IR body を読み、accepted / escaping / observation / unsupported source または operation event を生成する boundary。
- actual traversal 由来の fresh witness table を生成し、operation-classified actual traversal bundle として request-evidence bridge へ接続する boundary。
- PrivateCache / PrivateState effect masking。
- sealed memoized backend representation。
- `MemoKey` / `MemoValue` aggregate proof と producer-owned private cache region proof の接続。
- `.neplobj` / `.neplproof` / prechecked artifact 用 stable request key への投影。
- operation table request/key bucket 化、event split index 化、proof lookup index 化、stage0 fixture 分割、initialized-state 探索削減。

## 2026-06-21 selfhost memo_call backend actual traversal body adapter stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、HIR-root production path の actual traversal body source 生成を `actual_traversal_body_adapter` helper へ分離した。

この checkpoint は actual Resource IR body traversal 本体ではない。real Resource IR body、HIR lowering result、cache lookup / insert operation、effect operation はまだ読まない。現 stage0 adapter は `ResourceIrTraversalUnavailable` source record だけを返す。

今回の目的は、request recheck / proof key 生成 / graph id 作成 / owner cleanup を担当する `append_request_result` が、actual body traversal の source 生成責務まで抱え込まないようにすることである。`append_request_result` は adapter を呼び、返された source record を source table に push するだけにした。adapter error が返った場合は source table owner を閉じて fail-closed にする。

production path は引き続き accepted private cache storage、clone-out owned value、fresh witness、GraphInput、Resource proof table、request-evidence proof table、backend bytes、PrivateCache / PrivateState effect mask、`.neplobj` / `.neplproof` artifact key を合成しない。accepted source と matching witness は private fixture 専用のままであり、request identity だけから `PrivateCacheStoragePlace + CloneOutOwnedValueEdge` を作らない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。adapter が future real-body input boundary を保持し、availability boundary を通ることを固定している。stage0 の producer 未接続だけは unavailable source helper へ委譲し、available input は単一 source record に潰さず owner を閉じて typed unsupported error にする。accepted source / fresh witness / lower proof / backend / effect mask / artifact record を合成しないこと、`append_request_result` が unavailable source や resource place id を直接作らないことも固定している。

この分離は今やっておくべき semantic boundary である。actual traversal body を接続した後に request collection と source generation が混ざっていると、探索範囲や owner cleanup の責務が不透明になり、cache に頼らないコンパイル高速化の計算量設計も曖昧になる。一方で、request-key bucket 化、event split index 化、proof lookup index 化、adapter 内 stage0 分岐の分割は後からできる最適化として残す。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-actual-traversal-body-adapter-doctest-current.json`

subagent review:

- Meitner design review は、次 slice を accepted proof 生成ではなく `actual traversal body adapter` の fail-closed unavailable boundary にするべきだと指摘した。actual Resource IR / HIR lowering body を読めるまでは、production path で accepted source や fresh witness を合成せず、既存 classifier / normalizer / collector / candidate checker を経由する必要がある。今回の実装はその指摘に従っている。
- Wegener implementation review は `REVIEW_APPROVED`。`append_request_result` が direct unavailable source / place id を作らず adapter に委譲していること、adapter stage0 が unavailable-only で accepted source / fresh witness / lower proof / backend / effect / artifact を合成しないこと、source policy と doc comment が今回の境界に合っていることが確認された。

残件:

- adapter の内側で real Resource IR body / HIR lowering result を読み、typed place / edge / observation / unsupported source を生成する。
- fresh private cache region witness は source generation と別 authority として生成し、matching key / graph / ordinal で照合する。
- accepted source と fresh witness が揃った場合だけ producer-owned actual traversal bundle を request-evidence bridge へ接続する。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend actual traversal body input adapter stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、actual traversal body adapter が既存 `ResourceWalkerInput` owner と `ObservationBanTable` owner を消費して `ActualWalkerTraversalSourceTable` owner を返す private helper を追加した。

この checkpoint は actual Resource IR body traversal 本体ではない。real HIR lowering result / Resource IR body から body input owner を作る部分はまだ未接続である。今回固定したのは、real body input が届いた後に adapter が複数 traversal source を返せる owner boundary である。

`actual_traversal_body_adapter_sources_from_input_owners_result` は既存 collector を経由し、success / failure のどちらでも walker input owner と observation table owner を閉じる。成功時に返す source table owner は caller が閉じる。source count smoke は count を読んだ後に source table owner を閉じる。production HIR-root path はまだこの fixture helper を呼ばず、real body input が届くまでは unavailable-only のままである。

public stage0 summary は accepted-shaped input source count、observation-shaped input source count、unsupported input source count、placeholder rejected result だけを返す。private walker input、observation table、source table、operation table、fresh witness table、Resource proof table、request-evidence proof table、backend bytes、effect mask、artifact key は public API に出さない。accepted-shaped source count は source authority shape の smoke であり、fresh private region proof や backend representation 完了ではない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。body input adapter が existing collector を経由すること、input / observation owner cleanup、source table cleanup、summary の public payload 制限、adapter internals の public API 化禁止、proof / fresh witness / backend / effect / artifact 合成禁止を固定している。行数や doc comment 量を制限する検査は追加していない。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `git diff --check`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-actual-traversal-body-input-adapter-doctest-current.json`

subagent review:

- Meitner design review は、次 slice として private typed actual traversal body input/result boundary を推奨した。fixture-only body input は、production HIR-root path が accepted source を合成しない限り許容できる。key / graph / body identity binding、source kind taxonomy、duplicate / orphan rejection、fail-closed owner cleanup、actual-vs-fixture naming separation、proof / fresh witness coupling 禁止は今固定すべき境界であり、index 化は後続最適化でよいという指摘だった。

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- production HIR-root adapter が real body input available のときだけ source table owner path へ進む分岐。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- accepted source と fresh witness が揃った producer-owned actual traversal bundle の request-evidence bridge 接続。

## 2026-06-21 selfhost memo_call backend request-local source table merge checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、operation producer bridge が request ごとに単一 source record を push する形をやめ、request-local `ActualWalkerTraversalSourceTable` owner を producer-owned source table へ merge する形に変更した。

この checkpoint は actual Resource IR body traversal 本体ではない。production HIR-root path はまだ real body input を読まず、`actual_traversal_body_adapter_sources_from_request_result` から unavailable-only source table を返す。目的は、後続で real body input が available になったとき、1 request から複数 place / edge / observation / unsupported source が返っても、request collection や source-to-operation projection を再設計しなくてよい owner boundary を先に固定することである。

`actual_traversal_body_adapter_unavailable_sources_from_request_result` は unavailable source 1 件だけを持つ request-local source table owner を作る。`actual_walker_operation_producer_bridge_append_request_sources_result` は request-local source table owner を success / failure のどちらでも閉じる。producer source table owner は success の場合だけ返し、merge 失敗時は fail helper で閉じる。

public stage0 summary は unavailable fallback source count、accepted-shaped input source count、observation-shaped input source count、unsupported input source count、merged source count、placeholder rejected result だけを返す。private walker input、observation table、request-local source table、producer source table、operation table、fresh witness table、Resource proof table、request-evidence proof table、backend bytes、effect mask、artifact key は public API に出さない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。unavailable fallback が source table owner を返すこと、request adapter が source table owner boundary を持つこと、request-local source table owner を merge helper が必ず閉じること、`append_request_result` が plural helper を呼ぶこと、proof / fresh witness / backend / effect / artifact 合成をしないことを固定している。行数や doc comment 量を制限する検査は追加していない。

subagent review:

- Meitner design review は、次 slice として production HIR-root adapter の availability typed Result と source table owner transfer を推奨した。今回の checkpoint は、その前段として単一 source record push を request-local source table merge へ広げた。
- Wegener implementation review は `REVIEW_APPROVED`。request-local source table owner cleanup、production unavailable-only、public owner 非公開、proof / fresh witness / backend / effect / artifact 合成禁止が確認され、availability typed Result は次 slice 残件として扱えると判断された。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-source-table-merge-doctest-current.json`

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- production HIR-root adapter が real body input available のときだけ owner source table path へ進み、missing / unavailable / unsupported / malformed body input は fail-closed にする分岐。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- accepted source と fresh witness が揃った producer-owned actual traversal bundle の request-evidence bridge 接続。

## 2026-06-21 selfhost memo_call backend actual traversal body availability checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、production HIR-root adapter が real Resource IR body input を読む前段として、body input availability を typed `Result` と enum error に分ける boundary を追加した。

`SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind` は module-private とし、`ProducerNotConnected` / `Missing` / `Unavailable` / `Unsupported` / `Malformed` を bool や unavailable source へ潰さない。`ProducerNotConnected` は stage0 の producer 未接続 fallback だけを表し、real body reader が返す `Unavailable` と混同しない。public stage0 用の bridge error には `ActualTraversalBodyInputMissing` / `ActualTraversalBodyInputUnavailable` / `ActualTraversalBodyInputUnsupported` / `ActualTraversalBodyInputMalformed` を追加し、doctest と source policy が enum / Result / match で拒否理由を検査できるようにした。

`actual_traversal_body_adapter_input_availability_from_request_result` は production request path 上の availability 判定境界である。現段階では real body reader が未接続なので `ProducerNotConnected` を返す。`actual_traversal_body_adapter_sources_from_request_result` はこの判定を必ず通し、available の場合だけ split output から `walker_input` / `observations` owner を取り出して既存 input-owner adapter へ渡す。`ProducerNotConnected` だけは現 checkpoint の fail-closed fallback として unavailable source table へ写すが、`Missing` / real `Unavailable` / `Unsupported` / `Malformed` は unavailable source に偽装せず typed bridge error として返す。

public stage0 summary は従来の source count smoke に加えて、availability available / missing / unavailable / unsupported / malformed の代表結果を返す。private owner table、fresh witness、Resource proof table、request-evidence proof table、backend bytes、effect mask、artifact key は public API に出さない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。availability enum が module-private であること、producer 未接続 fallback と real unavailable を分けること、bridge error mapping が distinct であること、production request helper が availability 判定を通ること、available path だけが owner transfer を行うこと、proof / fresh witness / backend / effect / artifact を合成しないことを固定した。行数や doc comment 量を制限する検査は追加していない。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-availability-doctest.json`

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- actual body reader が available の場合に、real input owner を production path へ渡す接続。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- accepted source と fresh witness が揃った producer-owned actual traversal bundle の request-evidence bridge 接続。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend actual traversal body reader output connector checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、future real body reader が返す `ResourceWalkerInput` owner と `ObservationBanTable` owner を `ActualWalkerEventSplitOutput` へ束ねる module-private connector を追加した。

この checkpoint は actual Resource IR body traversal 本体ではない。real HIR lowering result / Resource IR body から owner pair を作る producer はまだ未接続であり、production HIR-root path は引き続き `ProducerNotConnected` fallback のままである。目的は、reader output が available になったときの owner pair authority と cleanup を、accepted proof / fresh witness / backend representation より先に固定することである。

`actual_traversal_body_reader_split_output_from_parts_result` は input result と observation table result の両方が `Ok` の場合だけ `ActualWalkerEventSplitOutput` を返す。input owner だけが作られて observation table が失敗した場合は input owner を閉じる。input が scanner error で observation table owner だけが作られていた場合は observation table owner を閉じる。input scanner error は `ActualTraversalBodyInputMalformed` bridge error へ写し、unavailable source や accepted source には変えない。

stage0 summary には `reader_connector_available_source_count` を追加した。これは reader connector が作る split output を既存 availability adapter と input-owner adapter へ渡せることを確認する smoke であり、actual traversal reader、fresh private region proof、PrivateCache / PrivateState effect masking、sealed backend representation の完了を意味しない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。reader connector が public API に出ないこと、`Ok` branch だけが owner pair を `ActualWalkerEventSplitOutput` へ載せること、partial owner failure で作成済み owner を閉じること、production availability helper が既存 `resource_walker_producer_bridge_input_from_hir_root_result` や actual walker event producer bridge を real body reader として呼ばないこと、proof / fresh witness / backend / effect / artifact を合成しないことを固定した。行数や doc comment 量を制限する検査は追加していない。

subagent review:

- Meitner design review は、この slice を future real body reader output connector に限定するなら妥当と判断した。production HIR-root path は `ProducerNotConnected` fallback のまま保つこと、既存 unsupported producer bridge を production availability に接続しないこと、owner pair は `Ok` だけで運び Err path は owner-free typed error にすること、actual traversal 本体 / fresh witness / effect masking / artifact 接続 / index 化は後続でよいことが確認された。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-reader-connector-doctest.json`

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- reader producer が available の場合だけ production request path へ `ActualWalkerEventSplitOutput` owner を渡す接続。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- accepted source と fresh witness が揃った producer-owned actual traversal bundle の request-evidence bridge 接続。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend actual traversal body reader request context checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、actual traversal body reader が将来読む request authority を module-private `ActualTraversalBodyReaderRequestContext` として束ねる checkpoint を追加した。

context は request entry、root expr id、body module fingerprint、proof key、Resource graph id を owner-free に保持する。`actual_traversal_body_reader_request_context_from_entry_result` は `proof_gate_recheck_entry_result`、`proof_key_from_entry_result`、`resource_graph_id_new` をこの順で実行し、成功した場合だけ context を返す。recheck 失敗は `RequestRecheckRejected`、proof key 失敗は `ProofKeyRejected` に写す。

`actual_walker_operation_producer_bridge_append_request_result` は、raw に proof key や graph id を組み立てず、context helper を通してから `actual_traversal_body_adapter_sources_from_request_context_result` へ渡すようにした。後続の production reader checkpoint では context helper が owner-bearing reader output を返し、missing / real unavailable / unsupported / malformed は typed bridge error として fail-closed にする。既存 unsupported producer bridge は real body reader として接続しない。

public stage0 summary は後続 checkpoint で `reader_context_reader_source_count` に更新した。これは HIR-root request entry から context を作り、production reader output を source table にできることを示す。context、walker input、observation table、source table、operation table、fresh witness table、Resource proof table、request-evidence proof table、backend bytes、effect mask、artifact key は public API に出していない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。context type / helper の public API 化禁止、context helper の recheck / proof key / graph id 作成順序、context availability が既存 unsupported producer bridge を呼ばないこと、append_request が raw key / graph id を直接作らないこと、proof / fresh witness / backend / effect / artifact 合成禁止を固定した。行数や doc comment 量を制限する検査は追加していない。

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- reader producer が available の場合だけ context-bound output validation を通して production request path へ `ActualWalkerEventSplitOutput` owner を渡す接続。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- accepted source と fresh witness が揃った producer-owned actual traversal bundle の request-evidence bridge 接続。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend actual traversal body context-bound reader output checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、available reader output から作った traversal source table を `ActualTraversalBodyReaderRequestContext` に束縛して検査する checkpoint を追加した。

`actual_traversal_body_adapter_sources_from_request_context_result` は available output を受け取った時、input-owner adapter を直接呼ばず、`actual_traversal_body_adapter_sources_from_request_context_output_result` へ渡す。output helper は source table owner を作った後、非空、proof key 一致、Resource graph id 一致を検査する。拒否時は source table owner を閉じてから `ActualTraversalBodyInputEmpty`、`ActualTraversalBodyInputKeyMismatch`、`ActualTraversalBodyInputGraphMismatch` の typed bridge error を返す。

この checkpoint は sealed private cache backend representation そのものではない。accepted source、fresh witness、Resource proof、request-evidence proof、GraphInput、PrivateCache / PrivateState effect mask、backend bytes、`.neplobj` / `.neplproof` artifact key は作らない。purpose は、将来 real body reader が available output を返した時に、別 request key / 別 graph / 空 source table を request-local source table として受け入れない semantic boundary を先に固定することである。

public stage0 summary には `reader_context_available_source_count`、`reader_context_key_mismatch_rejected`、`reader_context_graph_mismatch_rejected`、`reader_context_empty_source_rejected` を追加した。これらは context-bound available output validation の smoke であり、private context、walker input、observation table、source table、operation table、fresh witness table、proof table、backend bytes、effect mask、artifact key を public API にしない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。context-bound output helper 経由、proof key / graph id の両方の比較、empty source rejection、rejected source table owner cleanup、context helper の input-owner adapter 直呼び禁止、proof / fresh witness / backend / effect / artifact 合成禁止を固定した。行数や doc comment 量を制限する検査は追加していない。

## 2026-06-21 selfhost memo_call backend context-bound reader traversal bundle checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、context-bound available reader output を source-derived fresh witness bundle として actual traversal bundle gate へ渡す stage0 checkpoint を追加した。

`context_bound_reader_traversal_bundle_from_output_result` は、available output を直接 input-owner adapter に渡さず、既存の `actual_traversal_body_adapter_sources_from_request_context_output_result` を通して source table owner を得る。これにより、source table が非空であり、request context と同じ proof key / Resource graph id に閉じていることを確認してから次段へ進む。source validation に失敗した場合は `Stage0SourceRejected` に写し、fresh witness owner を作らない。

source validation に成功した後だけ、`actual_traversal_bundle_source_derived_witness_result` が source table から region proof table と no-escape candidate を作り、その candidate の proof key / graph id / root-support ordinal だけで fresh witness table owner を作る。witness body fingerprint、graph index、operation ordinal、witness status を外部から受け取る fixture path は context-bound helper から外した。candidate extraction 後は region proof table を閉じ、candidate extraction または witness table 作成に失敗した場合は source table owner も閉じる。bundle gate へ進んだ場合は、既存 `actual_traversal_bundle_request_evidence_gate_result` が source / witness / proof owner lifecycle を担当する。

public stage0 summary には `accepted_request_count`、`accepted_proof_count`、`context_key_mismatch_rejected`、`context_graph_mismatch_rejected` を追加した。これらは typed `Result` による smoke であり、reader context、split output、source table、fresh witness table、bundle、candidate、Resource proof table、backend bytes、effect mask、artifact key は public API に出さない。`RequestEvidenceProven` は request occurrence と no-escape proof の照合結果であり、PrivateCache / PrivateState の pure mask や backend 完了を意味しない。

この checkpoint 自体は production real reader 接続ではない。後続の production reader checkpoint では HIR-root production availability が context-derived reader output を返す。real HIR lowering result / Resource IR body から cache lookup / insert / effect operation を列挙する producer は未接続である。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。context-bound source helper 経由、direct input-owner adapter bypass 禁止、source validation 成功後だけ source-derived witness 作成へ進む順序、candidate field 由来の witness 作成、proof table / source table cleanup、request table / HIR module cleanup、helper 非公開、seed / availability rejection path の fail-closed 維持、proof / GraphInput / backend / effect / artifact 非生成を固定した。行数や doc comment 量を制限する検査は追加していない。

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- context-derived reader output を full Resource IR / HIR lowering body reader に拡張する接続。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- producer-owned actual traversal bundle を request-evidence bridge へ接続し、stage0 fixture ではなく real traversal output から accepted source と witness を供給する境界。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend context-bound reader seed availability checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、future real body reader の最小 evidence を表す module-private `ActualTraversalBodyReaderSeed` と、その seed を context-bound availability result へ変換する stage0 checkpoint を追加した。

この checkpoint は actual Resource IR traversal 本体ではない。目的は、stage0 accepted output を直接 split-output fixture で作らず、recheck 済み request context と同じ proof key / Resource graph id を持つ seed だけを owner-bearing output に変換する boundary を固定することである。後続の production reader checkpoint では accepted path が seed evidence ではなく request context 由来の reader output を使い、seed path は rejection smoke として残る。

seed availability helper は `Option::None` を missing seed として owner-free に拒否し、`Some seed` は key / graph id 照合、place / edge / observation shape validation、owner output construction の順で処理する。accepted representative は `PrivateCacheStorage`、`CloneOutOwnedValue`、`NoObservationDetected` だけであり、key mismatch、graph mismatch、missing seed、unsupported shape、observation detected は typed seed rejection として fail-closed にする。walker input owner 作成後に observation owner 作成が失敗した場合は walker input owner を閉じる。

`SelfhostMemoCallBackendPrivateCacheContextBoundReaderTraversalBundleStage0Summary` は accepted request/proof count、seed key mismatch、seed graph mismatch、missing seed、observation seed、unsupported seed、malformed seed、producer-not-connected availability、missing-reader availability の typed `Result` payload だけを公開する。seed、reader context、split output、source table、fresh witness table、bundle、Resource proof table、request-evidence proof table、GraphInput、backend bytes、effect mask、artifact key は public API に出していない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。seed type / helper の public API 化禁止、availability error taxonomy、seed authority validation、place / edge / observation の wildcard fallback 禁止、owner output cleanup、seed 系 availability error の bridge mapping、`to_place_index = -1` malformed seed rejection、availability helper が proof / fresh witness / backend / effect / artifact を合成しないこと、stage0 runner が split-output fixture ではなく seed availability helper を通ることを固定した。行数や doc comment 量を制限する検査は追加していない。

subagent review:

- Ramanujan review は blocking issue なし。seed が public accepted path / caller supplied authority になっていないこと、accepted path が context -> seed availability -> context-bound source validation -> source-derived witness -> request-evidence gate を通ること、seed mismatch / missing / unsupported / observation detected が owner 作成前に typed rejection へ落ちること、owner cleanup と Clone/Copy 禁止に問題がないことを確認した。
- 推奨指摘として、seed 系 availability error の bridge mapping と `to_place_index = -1` malformed seed case の source-policy 固定が出たため、`seed_malformed_rejected` と bridge mapping assertion を追加した。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-seed-availability-doctest.json`。17/17。
- pass: `node nodesrc/analyze_tests_json.js tmp/selfhost-memo-call-backend-private-cache-seed-availability-doctest.json`

残件:

- real HIR lowering result / Resource IR body から seed ではなく `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- context-derived reader output を full Resource IR / HIR lowering body reader に拡張する接続。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- producer-owned actual traversal bundle を request-evidence bridge へ接続し、stage0 seed fixture ではなく real traversal output から accepted source と witness を供給する境界。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend context-bound availability traversal bundle checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、future real body reader の availability result を context-bound traversal bundle へ渡す module-private helper を追加した。

`context_bound_reader_traversal_bundle_from_availability_result` は `Result::Ok ActualWalkerEventSplitOutput` の場合だけ既存 `context_bound_reader_traversal_bundle_from_output_result` へ渡す。これにより、available output は context-bound source validation、source-derived witness generation、actual traversal bundle request-evidence gate の順序を必ず通る。`Result::Err AvailabilityErrorKind` の場合は split output owner、source table owner、fresh witness table owner、bundle owner を作らず、bridge error へ写したうえで `Stage0SourceRejected` として返す。

`ProducerNotConnected` は source adapter の production fallback では unavailable source table へ写るが、accepted bundle path では proof に到達させない。この違いを doccomment と source policy に固定した。bridge error 上では `ActualTraversalBodyInputUnavailable` に畳まれるため、public stage0 summary の field は `producer_not_connected_availability_rejected` とし、availability 入力条件が rejected になったことを明示した。

public stage0 summary には `producer_not_connected_availability_rejected` と `missing_reader_availability_rejected` を追加した。これらは owner-free typed `Result` payload であり、reader context、split output、source table、fresh witness table、bundle、Resource proof table、GraphInput、backend bytes、effect mask、artifact key は public API に出さない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。availability helper 経由、Err availability から bundle 非到達、direct output helper bypass 禁止、availability rejection runner が split output fixture を作らないこと、private helper 非公開、GraphInput / proof push / backend bytes / effect mask / artifact record 非生成を固定した。行数や doc comment 量を制限する検査は追加していない。

subagent review:

- Locke review は `CHANGES_REQUESTED`。source-derived witness へ閉じること、bundle helper で `ProducerNotConnected` を unavailable source table にしないこと、helper 非公開、Ok output だけの委譲、Err availability の owner-free typed rejection、ProducerNotConnected / Missing の代表 smoke、GraphInput / proof push / backend bytes / effect mask / artifact key 非生成の固定が要求された。
- 指摘に対して、現在の `from_output_result` は source-derived witness helper へ閉じていることを確認しつつ、`ProducerNotConnected` と bridge error 表現の混同を避けるため summary field 名を `producer_not_connected_availability_rejected` に変更し、doccomment / source policy に bundle path 非到達を追記した。
- Locke implementation review は `REVIEW_APPROVED`。Err availability path が split output / source table / witness table / bundle を作らないこと、Ok output path が context-bound source validation と source-derived witness helper を迂回しないこと、`ProducerNotConnected` と source adapter fallback の差が doccomment / summary comment で明確であること、helper 非公開、GraphInput / proof push / backend / effect / artifact 合成禁止、行数制限や doccomment 抑制検査の非追加が確認された。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-context-bound-availability-doctest.json`。17/17。

残件:

- real HIR lowering result / Resource IR body から `ResourceWalkerInput` / `ObservationBanTable` owner を作る producer。
- context-derived reader output を full Resource IR / HIR lowering body reader に拡張する接続。
- actual traversal 由来 fresh witness table の生成と、source table owner との key / graph / ordinal 照合。
- producer-owned actual traversal bundle を request-evidence bridge へ接続し、stage0 fixture ではなく real traversal output から accepted source と witness を供給する境界。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、stable artifact key projection。

## 2026-06-21 selfhost memo_call backend production actual traversal reader checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、HIR-root production availability を `ProducerNotConnected` fallback ではなく、recheck 済み `ActualTraversalBodyReaderRequestContext` から作る owner-bearing reader output へ接続した。

`actual_traversal_body_reader_output_from_request_context_result` は context の proof key と Resource graph id を authority とし、memo_call backend wrapper body の代表 evidence として `PrivateCacheStorage` place と `CloneOutOwnedValue` edge を持つ `ResourceWalkerInput` owner と、空の `ObservationBanTable` owner を返す。seed、既存 unsupported producer bridge、public source table、GraphInput、proof table、backend bytes、effect mask、artifact record は使わない。

accepted bundle runner は production availability `Ok`、context-bound source validation、source-derived witness generation、actual traversal bundle request-evidence gate の順に進む。operation producer bridge stage0 は production reader output 由来の traversal source table を operation table へ投影する accepted smoke を持つ。explicit unavailable smoke は専用 unavailable source helper を呼び、production bridge helper を流用しない。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。production reader が seed fixture、既存 unsupported producer bridge、proof / backend / effect / artifact 合成を使わないこと、context availability helper が `ProducerNotConnected` fallback を残さないこと、accepted bundle runner が production availability を通ること、explicit unavailable smoke が production bridge helper を呼ばないことを固定している。

review 対応:

- Rawls の implementation review は `REVIEW_CHANGES_REQUESTED`。compat availability helper が caller-supplied key / graph id から context を直接作っていたこと、context source helper が `ProducerNotConnected` を unavailable source table へ写していたことが blocker とされた。
- 指摘に従い、compat availability helper も `actual_traversal_body_reader_request_context_from_entry_result` を通してから context availability へ委譲し、context source helper は availability error をすべて typed bridge error へ写すようにした。unavailable source table は explicit unavailable smoke helper だけが作る。
- Rawls re-review は `REVIEW_APPROVED`。split output helper の stale comment も production reader output が recheck 済み context 由来である説明へ修正済みであることが確認された。

検証:

- pass: `node --check nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `$env:NEPL_TEST_CASE_TIMEOUT_MS='600000'; node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-private-cache-production-reader-doctest.json`。17/17。
- pass: `node nodesrc/analyze_tests_json.js tmp/selfhost-memo-call-backend-private-cache-production-reader-doctest.json`。17 passed / 0 failed。
- pass: `node nodesrc/issues.js check --dir issues`
- pass: `git diff --check`。CRLF warning のみ。
- pass: `trunk build`
- pass: `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-production-reader.json`。13/13、JSON は `caseCount: 13`、`passedCount: 13`、`failedCount: 0`。
- pass: `node nodesrc/run_source_policy_regressions.js`

残件:

- context-derived wrapper body reader を full Resource IR / HIR lowering body reader へ拡張し、cache lookup / insert / observation / effect operation を typed traversal source または operation event として発行する。
- full traversal 由来 source と source-derived witness を、private effect no-escape gate と request-evidence bridge の両方へ同じ body identity で渡す upper orchestration を追加する。
- PrivateCache / PrivateState effect masking、sealed memoized backend representation、Wasm / LLVM bytes、`.neplobj` / `.neplproof` stable key projectionへ接続する。

## 2026-06-21 selfhost memo_call backend reader operation policy source checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、production operation producer bridge が使う request-context source helper を、split-output availability authority ではなく module-private reader operation policy source table に寄せた。

`SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind` は、wrapper default の `WrapperPrivateCacheStorage` / `WrapperCloneOutOwnedValue` と、cache lookup / insert、PrivateCache / PrivateState effect、cache / function / raw observation を typed vocabulary として持つ。production default は wrapper 2 source だけで、lookup / insert / effect / observation policy は source-to-operation projection と classifier / normalizer へ届くが accepted proof へ混ざらない。

source vocabulary と operation vocabulary には `CacheLookupOperation`、`CacheInsertOperation`、`PrivateCacheEffectOperation`、`PrivateStateEffectOperation`、cache hit/miss/size/stats/clear/debug/region identity observation、function hash/debug/closure allocation observation、raw representation observation を追加した。source-to-operation projection、operation classifier、region proof input projection は wildcard fallback なしで更新した。cache lookup / insert は `PrivateCacheOperationUnsupported`、private effect は `PrivateStateBoundaryUnsupported`、observation は既存 observation ban payload として fail-closed に流す。

この checkpoint は executable cache operation、GraphInput、request proof table、Resource proof table、fresh witness table、backend bytes、effect mask、sealed backend representation、`.neplobj` / `.neplproof` artifact key を作らない。actual Resource IR / HIR lowering body reader 本体、実 traversal 由来 fresh witness table、PrivateCache / PrivateState effect masking、sealed memoized backend representation は後続で接続する。

## 2026-06-22 selfhost memo_call backend event producer convergence checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` で、actual walker event producer bridge の direct `Body + UnknownResourceOperation` event constructor を production path から外し、resolver-bound HIR body reader source plan、operation projection、operation classifier を通る path へ収束させた。

`actual_walker_event_producer_bridge_events_from_hir_root_result` は borrowed resolution table を受け取り、operation producer bridge で operation table owner を作り、operation classifier で unified event table owner を作ってから operation table owner を閉じる。stage0 runner は operation producer bridge と同じ resolution table owner を作成し、success / failure のどちらでも閉じる。public summary の first field は `accepted_result` とし、reader-derived clean wrapper source が classifier / normalizer / graph gate を通る smoke であることを明示した。

この checkpoint は full Resource IR traversal、fresh-region witness table、request-evidence proof、PrivateCache / PrivateState effect mask、sealed backend representation、backend bytes、`.neplobj` / `.neplproof` artifact key を作らない。残件は、full Resource IR / HIR lowering body traversal から accepted / escaping / observation / unsupported source と fresh witness を同じ body identity で発行し、private effect no-escape gate と request-evidence bridge の両方へ渡す上位 orchestration である。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。event producer が operation producer / classifier を経由する順序、operation owner cleanup、stage0 resolution owner cleanup、direct event table constructor / `UnknownResourceOperation` / direct proof key / graph id 作成禁止、backend / effect / artifact 非生成を固定している。

検証:

- pass: `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- pass: `node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl --dist web/dist -o tmp/selfhost-memo-call-backend-event-producer-convergence.json`。17/17。
- pass: `node nodesrc/test_stdlib_documentation_contract.js`
- pass: `trunk build`
- pass: `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=output/playground_editor_selfhost_memo_event_producer_convergence.json`。13/13。

## 2026-06-21 selfhost private effect no-escape gate dependency checkpoint

`stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_no_escape_gate.nepl` を追加し、memo_call backend proof chain が将来発行する `PrivateState` / `PrivateCache` no-escape proof を、operation method body fact table へ渡す直前で消費できる checker-layer boundary を固定した。

この checkpoint は memo_call backend request-evidence proof の完了ではない。gate は typed proof table と HIR-root scan record を照合するだけで、`RequestEvidenceProven`、Resource proof production、GraphInput、sealed backend bytes、PrivateCache / PrivateState effect mask、`.neplobj` / `.neplproof` artifact key は作らない。backend 側の残件は、real Resource IR / HIR lowering result から accepted traversal source と fresh witness を生成し、その proof を request-evidence bridge と private effect no-escape gate の両方へ整合した identity で渡す上位 orchestration である。
