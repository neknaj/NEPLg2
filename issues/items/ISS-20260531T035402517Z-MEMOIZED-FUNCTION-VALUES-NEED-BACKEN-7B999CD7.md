---
id: ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7
title: "memoized function values need backend representation and identity-observation ban"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-15
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
