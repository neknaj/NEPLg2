---
id: ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649
title: "binary intermediate artifacts needed for incremental compile"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-02
target: "nepl-core/src/compiler.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_performance_cache_design.md"
---

# ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649: binary intermediate artifacts needed for incremental compile

## 概要

現状の NEPLg2 は typed HIR、Resource summary、compiled output cache を主に process-local memory に保持している。JVM の `.class` や C 系の `.o` に相当する、session をまたいで再利用できる永続 binary intermediate artifact はまだ持っていない。

## 対象

- `nepl-core/src/compiler.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_performance_cache_design.md`

## 根拠

- Zenn 方針では、純粋性、依存関係の DAG 化、静的検査、cache により探索範囲と計算量を削減することが要求されている。
- `.class` は verifier が検査できる typed bytecode、`.o` は linker が再接続できる target-specific relocatable fragment である。NEPLg2 ではこの 2 つを混ぜず、platform-neutral typed/proof artifact と target-specific codegen fragment を分離する必要がある。
- stdlib や selfhost compiler は大きくなっており、毎回 source から全 pipeline を再構築すると 0.5 秒未満 compile / 0.1 秒未満 warm recompile / 10ms 級 expression edit に届かない。

## 問題

現行の compiled-output cache は source 全体 key が同一の場合には強いが、小さな式枝差し替えでは miss し、typecheck、Resource IR、summary replay、codegen を広く再実行する。Resource summary value cache は function-level proof の一部を保持できるが、typed public surface、typed HIR、Resource proof、codegen fragment を統一した永続 artifact boundary にはなっていない。

## 影響

安定した中間 artifact 境界がないままだと、リテラル変更や小さな式枝差し替えで source 全体 key が変わり、変更されていない module / function / stdlib proof / codegen fragment まで再処理される。この状態では 0.1 秒 warm recompile と 10ms 級の微小差分 compile target が構造的に達成しにくい。

## 修正方針

NEPL object artifact stack を `.nepl...` 形式の artifact として設計し、段階的に実装する。
短い `.nei` / `.nehir` / `.ners` / `.neo` 形式は採用しない。NEPL 固有の artifact で
あることを拡張子から確認できるようにし、役割名も読み取れる形にする。

- `.neplmeta`: import graph、exported type/function/trait impl surface、effect signature、typed public signature、source capability policy surface を保持する。
- `.neplhir`: stable lexical path id、typed HIR、typed diagnostics enum、local binding shape、expected type boundary を保持する。永続化は stable typed id 導入後に行い、MVP では same-session cache に限定する。
- `.neplproof`: Resource IR summary、private effect mask proof、drop/borrow/owner/initialized proof summary を stable mirror として保持する。
- `.neplobj`: wasm / LLVM の function fragment、signature table entry、function table entry、data segment、relocation metadata を保持する。
- `.nepllink`: fragment の symbol / relocation / table index / data offset を再接続し、final wasm / LLVM artifact を生成する。

cache key には compiler version、artifact schema version、target/profile、stdlib content hash、module public surface hash、dependency public surface hash、source capability policy hash、type/effect boundary hash、generic type arguments、backend feature set を含める。どれかが再投影できない場合は stale hit を避けるため fail-closed に再計算する。

実装順序は `.neplmeta`、`.neplproof`、same-session `.neplhir` query cache、`.neplobj` /
`.nepllink`、persistent `.neplhir` とする。`.neplobj` を先に作る案は採用しない。
NEPLg2 の prefix call boundary は依存 module の callable candidate / arity / effect /
generic surface を必要とするため、interface artifact なしでは `.o` 相当を持っても
再型検査の支配コストを削れないからである。

## 検証

same-session と cross-session の RPN expression edit を測定し、次を確認する。

- public surface が不変の edit で、stdlib / dependency module artifact が再利用される。
- changed function と dependent Resource summary だけが再計算される。
- unchanged codegen fragment は relocation/link だけで再接続される。
- final wasm は full compile と同一の挙動を持つ。
- source capability、generic substitution、diagnostic span、private effect mask proof を再投影できない場合は cache hit せず安全側に再計算する。

## 2026-06-01 checkpoint

`.neplproof` の disk schema を先に固定せず、`ResourceSummaryValueCache` の stable mirror から
in-memory snapshot / preseed 境界を追加した。

実装した API は `export_neplproof_snapshot` と `preseed_neplproof_snapshot` である。snapshot
は `ResourceSummaryValueCacheKey` と stable mirror entry だけを保持し、
`ResourceSummaryReplaySnapshot` や `InitializedFunctionCheckPassSnapshot` は保持しない。
これらの replay plan は前回 compile の関数順序と local fingerprint に依存するため、
cross-session の `.neplproof` authority にはしない。

preseed は現在 cache の entry を上書きしない。同じ key/value は existing hit として扱い、
同じ key で異なる value は conflict として拒否する。実際の proof replay は従来どおり現在の
`TypeCtx`、function signature、source capability policy、generic boundary へ再投影できる
entry だけを使う。

次の作業は、この snapshot に compiler version、schema version、target/profile、stdlib hash、
dependency public surface hash、private effect policy hash を持つ envelope を付けることである。

## 2026-06-01 checkpoint 2

`.neplproof` snapshot に `ResourceSummaryProofArtifactHeader` / `ResourceSummaryProofArtifact`
を追加した。これは disk codec ではなく、artifact payload を cache へ preseed する前に
schema、compiler identity、target、profile、stdlib content hash、dependency public surface hash、
Resource summary namespace hash、source capability policy set hash、private effect policy hash を
照合する envelope である。

`preseed_neplproof_artifact` は header が一致しない artifact を payload merge 前に拒否する。
header が一致した場合でも、個別 summary entry は従来の replay API が現在の `TypeCtx` /
function signature / source capability policy へ再投影できる場合だけ使う。

次の作業は、Web / CLI / selfhost 側でこの header を作るための canonical compiler identity hash、
target/profile hash、stdlib artifact hash の生成と、preseed/export を session API に薄く接続することである。

## 2026-06-01 checkpoint 3

`.neplproof` の expected header を Web / CLI 側で再実装しないよう、
`ResourceSummaryCacheNamespaceKey::resource_summary_proof_header` を追加した。
この API は typecheck 後に確定する typed public signature hash、dependency public surface hash、
target/profile、source capability policy set、private effect policy version から
`ResourceSummaryProofArtifactHeader` を作る。

`resource_summary_private_effect_policy_hash` は常に `Some` として header に入れる。
これにより private effect policy 未実装同士の `None == None` で artifact を受け入れる経路を避ける。
source capability policy set も `SourceMap` から集約し、private cache use-site の span / operation /
source content が変わると artifact-level で miss する。

compile pipeline には `ResourceSummaryProofArtifactCacheOptions` と
`compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_resource_summary_value_cache_and_neplproof`
を追加した。preseed artifact は Resource static check の直前に header 照合され、mismatch の場合は
通常 compile へ fail-closed に戻る。disk / IndexedDB codec はまだ実装しない。codec を追加するときは、
header を先に decode / compare し、mismatch の artifact では payload を decode しない二段階にする。

## 2026-06-01 checkpoint 4

Web `CompilerSession` に same-session `.neplproof` artifact slot を追加した。
compiled-output cache miss の実 compile では、前回 slot の artifact を core compiler の
`ResourceSummaryProofArtifactCacheOptions` へ渡し、compile 成功後に現在の
`ResourceSummaryValueCache` snapshot と core が返した header から新しい artifact を保存する。
compiled-output cache hit では core pipeline が走らないため、cache entry が保持している artifact を
session slot へ戻すだけにし、Web 側で header を再計算したり現在 cache から再 export したりしない。

stdlib overlay がある compile では loader cache / Resource summary value cache と同じ条件で
`.neplproof` preseed/export も無効にする。bundled stdlib hash は `fnv1a64:{hex}` の suffix を
Rust 側で `u64` へ変換し、変換できない場合は artifact を作らない。JS number / `f64` 経由の
hash 変換は使わない。

`loader_cache_stats_json` には artifact slot の有無、entry 数、store/preseed candidate 数、
stdlib hash parse 可否を追加した。payload 本体や diagnostic span、`TypeId`、`SourceMap` は
JSON へ出さない。

## 2026-06-01 checkpoint 5

`.neplmeta` の最小 in-memory artifact 境界を追加した。payload は
`TypedPublicSignatureTable` だけであり、依存側 typecheck に必要な public callable / type /
trait / impl surface を stable text/hash として運ぶ。`TypeId`、`Span`、`SourceMap`、typed HIR、
Resource IR body、diagnostic span は含めない。

`NeplMetaArtifactHeader` は schema、compiler identity、target/profile、stdlib content hash、
dependency public surface hash、typed public signature hash、public entry count、
source capability policy set hash、private effect policy hash を保持する。payload decode 後は
header が主張する typed public signature hash と entry count が実 payload と一致することも
別に確認する。

`CompilationArtifact` と Web `CompilerSession` は `.neplmeta` artifact を保持する。compiled-output
cache hit では cache entry の artifact を session slot へ戻し、通常 compile 成功時は core が
返した artifact を slot へ保存する。`loader_cache_stats_json` は artifact の有無、public entry
数、typed public signature hash、payload consistency だけを公開し、payload 本体は JSON へ出さない。

この checkpoint では disk / IndexedDB codec、`.neplmeta` からの typecheck environment 注入、
typed HIR reuse、dependency module body skip はまだ実装しない。次段階では、この stable
interface artifact を loader / typecheck の import boundary へ渡し、stdlib や依存 module の
body 再 typecheck を減らす。

## 2026-06-01 checkpoint 6

subagent review により、`.neplmeta` から base compile time を下げる次段階を body skip へ直行させない方針を確認した。

現 payload は `TypedPublicSignatureTable` の stable text/hash だけであり、`TypeCtx`、`Env`、trait table、impl table を復元する structured public surface ではない。このため、`.neplmeta` の永続 codec や stdlib body skip より先に、次の issue へ分割して進める。

- [ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31](./ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31.md): `.neplmeta` payload に arena 非依存 structured public surface を追加する。
- [ISS-20260531T223904937Z-NEPLMETA-NEEDS-TYPECHECK-SURFACE-MAT-E7FF61B7](./ISS-20260531T223904937Z-NEPLMETA-NEEDS-TYPECHECK-SURFACE-MAT-E7FF61B7.md): structured surface を現在 compile の fresh `TypeCtx` / `Env` へ fail-closed に materialize する。

この分割により、`TypeId`、`Span`、`SourceMap`、typed HIR、Resource IR body を artifact に保存しない方針を維持しつつ、stdlib / dependency body 再 typecheck 削減へ進める。

## 2026-06-01 checkpoint 7

`.neplmeta` に structured public surface payload を追加した。既存の typed public signature text/hash は残し、header には structured public surface hash と entry count を追加した。

artifact 形状が変わったため `.neplmeta` schema / hash / compiler identity は v2 にした。Web stats では structured public surface hash と entry count を観測できるが、payload 本体は公開しない。

この checkpoint は body skip ではない。subagent review の指摘に基づき、`Named(String)`、名前だけの generic param、span-derived callable symbol は materializer authority にしない。次は stable nominal identity、binder-indexed generic parameter reference、stable public ABI/link symbol、field accessor surface を足してから typecheck materializer へ進む。

## 2026-06-01 checkpoint 8

`.neplmeta` header に `source_key_hash` を追加した。これは public surface や typed public signature が
変わらない body-only edit でも、保存済み artifact が現在 source に由来しない場合は import
materializer や body skip へ進まないようにする invalidation 境界である。

`source_key_hash` は `compiled_source_cache_key_part` から作るため、通常コメントや doc comment、
span だけの変更では同じ値になる。一方で literal、identifier、directive、indent / dedent、
raw wasm / llvm text など compile 結果に影響し得る token が変わると値が変わる。

`SourceMap` または canonical module path がない artifact は `source_key_hash=None` になり、
body skip の authority にはしない。Web stats には `nepl_meta_artifact_source_key_hash` を追加し、
payload や source text を出さずに stale-hit 防止境界だけを観測できるようにした。

追加修正として、typed public signature / structured public surface を要求しない
`NeplMetaArtifactPreTypecheckEnvelope` を追加した。body skip 前は依存先 body をまだ typecheck
していないため、expected header を full typed artifact から作れない。この envelope は
target/profile、stdlib、dependency public surface、module surface、source capability policy、
private effect policy、source key だけを照合し、payload decode 前の fail-closed gate として使う。

また、`source_key_hash=None` の artifact は `NeplMetaArtifactStore` と materializer MVP が
`MissingSourceKey` として拒否する。これにより `None == None` で source identity を証明できない
artifact が compatible 扱いになる経路を閉じた。

追加検証:

- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `node tests\compiler\tree\run.js`

## 2026-06-01 checkpoint 9

`.neplmeta` store projection に pre-typecheck envelope 用 API を追加した。
`materializer_import_public_surface_pre_typecheck_mvp` は full typed header を要求せず、
loader/source map 由来の `NeplMetaArtifactPreTypecheckEnvelope` で artifact header を先に照合する。

この API は `.neplmeta` を body skip に使う実装ではない。成功時に返すのは materializer 入力の
`TypedPublicSurfaceTable` だけであり、依存 module AST inline、typecheck、Resource IR proof は
従来通り実行される。目的は、loader/import/prelude boundary で「この edge の public callable
surface は artifact から復元可能か」を fail-closed に観測することである。

source key、dependency public surface、module surface の mismatch は
`NeplMetaArtifactCompatibilityReject` の enum reason として拒否する。projection unsupported や
payload inconsistency も既存 reason で区別する。

追加検証:

- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`

## 2026-06-01 checkpoint 10

`.neplmeta` pre-typecheck projection probe の観測統計を追加した。store 全体の `hits` は
「artifact が module path で見つかった」ことだけを表すため、performance 判断では
projection success と reject reason を別に見る必要がある。

追加した統計:

- `pre_typecheck_probe_attempts`
- `pre_typecheck_probe_projected`
- `pre_typecheck_probe_missing_artifacts`
- `pre_typecheck_probe_payload_rejects`
- `pre_typecheck_probe_compatibility_rejects`
- `pre_typecheck_probe_projection_rejects`
- `pre_typecheck_probe_projected_entries`
- `last_pre_typecheck_probe_reject_kind`
- `last_pre_typecheck_probe_reject_code`
- `last_pre_typecheck_probe_projected_entries`

compatibility / payload / projection reject enum には stable code を追加した。これは disk / IndexedDB
codec や Web benchmark から、fallback reason を文字列に依存せず集計するための中間 artifact
仕様である。通常 compile path はまだ probe を呼ばないため、今回の変更は body skip や import
materializer 接続を開始しない。

追加検証:

- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `node tests\compiler\tree\run.js`

## 2026-06-01 checkpoint 11

`.neplmeta` pre-typecheck projection probe を Web `CompilerSession` の実 compile path へ接続した。

現段階では root module artifact だけを観測する。compile 成功時に store 済みの前回 root artifact が
存在する場合、loader/source-map 由来の `NeplMetaArtifactPreTypecheckEnvelope` を作り、
`materializer_import_public_surface_pre_typecheck_mvp` で compatibility と projection を確認する。
結果は stats にだけ反映し、依存 module AST inline 省略や typecheck materializer 接続は行わない。

この checkpoint の意味:

- 中間 artifact が「存在するだけ」では高速化の根拠にならないため、pre-typecheck envelope で
  再利用可能性を測る実 compile path の観測点を作った。
- dependency body-only edit では、root source key と dependency public surface が不変でも、
  現 payload の materializer blocker が残る artifact は projection reject として数えられる。
- root source literal edit では typed public signature が不変でも source key mismatch で拒否され、
  projection 判定へ進まない。

次の `.neplmeta` 作業は、subagent review に従い `Loader::process_directives_with` の import/prelude
edge 単位の probe hook へ進める。そこでは prelude は `import_clause=None`、import は loader が持つ
`NeplMetaImportClause` を渡し、`Include` は引き続き artifact 境界にしない。

## 2026-06-01 checkpoint 12

Web `CompilerSession` に stdlib dependency `.neplmeta` artifact producer を追加した。

この producer は、store が空でない compile で収集された import/prelude edge probe を入力にする。
初回 compile では edge probe を収集しないため、base compile の固定費を増やさない。二回目以降の
compile で edge target が missing artifact として観測された後、compile 成功時にその bundled stdlib
module を non-root dependency として読み直し、typecheck までで止めた `.neplmeta` artifact を store へ
追加する。

重要な境界:

- `compile_nepl_meta_artifact_with_source_identity` は Resource IR、drop 挿入、wasm codegen を実行しない。
  `.neplmeta` は公開 interface artifact であり、body safety proof や executable object ではない。
- target source key と source capability policy は root `SourceMap` からではなく、edge probe が target
  source 単位で計算した値を header へ固定する。
- `std/prelude_base` を root として読み直すと既定 prelude 注入により自己循環するため、
  `Loader::load_dependency_inline_with_provider_and_cache` で non-root load する。
- stdlib overlay / `/stdlib` VFS override、non-stdlib import、`#include` は producer 対象にしない。
- store に同じ pre-typecheck envelope と互換な artifact が既にある場合は、統計を汚さず再 typecheck を
  避ける。

この checkpoint でも `TypedPublicSurfaceTable` の `TypeCtx` / `Env` 注入、依存 module AST inline 省略、
Resource IR skip、codegen skip は行わない。固定した観測は、三回目の同一 edge probe で missing artifact が
増えず、現在の materializer MVP の未対応 surface が projection reject として見えることである。

追加検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node tests\compiler\tree\run.js`

## 2026-06-01 checkpoint 13

`.neplmeta` projection 成功後の materialized compile attempt と source fallback を `CompilerSession`
stats で観測できるようにした。

この checkpoint は `.neplobj` / `.nepllink` 実装ではない。`.neplmeta` は metadata-only artifact なので、
materialized callable が HIR / codegen 入力へ到達した場合は `MaterializedFunctionBodyMissing` を理由に
source fallback へ戻る。この fallback は safety 上の正常経路であり、性能改善の失敗ではなく
「次に object/link artifact が必要な箇所」を示す観測値として扱う。

`.neplobj` / `.nepllink` へ進む前の注意点:

- `.neplobj` key には `.neplmeta` / `.neplproof` の header 境界に加え、backend feature set、
  WAT comment mode、ABI / link symbol version、function table / data segment layout version、
  relocation schema version、selected callable body hash、generic instantiation hash を含める。
- `.nepllink` key には object fragment 集合 hash だけでなく、symbol resolution order、
  table index allocation policy、data offset allocation policy、entry / export set、target / profile /
  backend feature set を含める。
- artifact decoder は header を先に decode / compare し、mismatch artifact では payload を読まない。
- projection success と compile fallback は別 layer なので、cache stats も別 counter として維持する。

追加検証:

- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node tests\compiler\tree\20_compiler_session_outputs_cache.js`

## 2026-06-01 checkpoint 14

`nodesrc/run_test.js` の timing に `compiler_session_stats` を追加し、`.neplmeta` materialized compile
counter の before / after / delta を compile 単位で出すようにした。`compare_git_versions.js` はこの
delta を `compile_ms` と同じ report に集計し、Markdown に `Materialized Compile` table を追加する。

この checkpoint により、`.neplmeta` projection が成功した後に body missing で source fallback した
件数を、base / warm edit compile time と同じ JSON / Markdown で追える。`.neplobj` は、まず
`body_missing_fallbacks_delta_sum` が増える callable surface を候補として設計する。

この checkpoint は object/link artifact の payload codec ではない。`TypeId`、`Span`、`SourceMap` を
artifact に保存しないという方針、header-first decode、selected callable body hash / generic
instantiation hash / backend feature set を key に含める方針は checkpoint 13 のまま維持する。

## 2026-06-01 checkpoint 15

`CompilerSession` stats に `.neplobj` candidate surface counter を追加した。

- `nepl_obj_candidate_body_missing_surfaces`
- `nepl_obj_candidate_last_body_missing_surfaces`

これは `MaterializedFunctionBodyMissing` を理由に source fallback した compile で、実際に compile
pipeline へ渡された materialized public surface 数を数える。compile attempt 1 件につき複数 surface が
入る場合があるため、`.neplobj` 実装の対象規模は fallback compile 件数だけでは判断しない。

この counter はまだ symbol / function / call-kind 単位ではない。`TypeId`、`Span`、`SourceMap`、typed
HIR を artifact 化せず、既存の `.neplmeta` projection 境界と fallback reason enum だけから surface 数を
観測するための最小 checkpoint である。`compare_git_versions.js` は
`body_missing_candidate_surfaces_delta_sum` として report に出す。

`nodesrc/bench_materialized_compile_fallbacks.js` も追加した。通常の tree / doctest suite では worker
単位の `CompilerSession` 温度や fixture 順序が混ざるため、`.neplobj` candidate の実測は専用 script で
cold / warm / body edit sequence を固定する。この script は timing threshold ではなく、`compile_ms`、
materialized compile delta、`resource_typecheck` / `resource_static_check` / `wasm_codegen` の stage
timing を JSON に保存する。

## 2026-06-01 checkpoint 16

`OtherCoreError` に丸められていた materialized compile fallback を、primary `DiagnosticCode::as_str()`
で観測できるようにした。

- `CompilerSession.loader_cache_stats_json()` は
  `nepl_meta_materialized_compile_last_fallback_diagnostic_code` を返す。
- `nodesrc/bench_materialized_compile_fallbacks.js` は per-run の
  `materialized_compile.last_fallback_diagnostic_code` と summary の
  `materialized_fallback_diagnostic_code_counts` を出す。
- `PublicSurfaceMaterializeRejectReason` は coarse な
  `type.public_surface.materializer_rejected` ではなく typed `TypeDiagnosticCode` へ写される。

実測では `tmp/materialized-fallback-detail-20260601.json` の warm compile 3 回すべてが
`type.public_surface.materializer.field_accessor_unsupported` であり、`.neplobj` body missing では
なかった。したがって、次の root gap は selected callable body fragment ではなく、field accessor callable
surface を current session の typecheck environment へ安全に materialize することである。

この root gap は
`ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B`
として分離した。

## 2026-06-01 body-missing skip checkpoint

`.neplmeta` projection 後に `backend.codegen.materialized_function_body_missing` で fallback した dependency
edge を、同一 `CompilerSession` 内の source-hash scoped negative cache に記録するようにした。

この cache は `.neplobj` の代替ではない。`target_module_path`、`target_source_key_hash`、
`dependency_public_surface_hash` が一致する間だけ、`.neplobj` 未実装で必ず失敗する materialized compile
attempt を再実行しないための fail-closed 境界である。`.neplobj` availability resolver が入った後は、
同じ key 境界で skip ではなく object fragment 解決へ置き換える。

追加した counter:

- `nepl_meta_body_missing_skip_entries`
- `nepl_meta_body_missing_skip_hits`
- `nepl_meta_body_missing_skip_stores`
- `nepl_meta_body_missing_skip_stale_entries`
- `nepl_meta_body_missing_skip_last_hits`
- `nepl_meta_body_missing_skip_last_stores`

`tmp/neplmeta-body-missing-skip-20260601.json` では、warm 3 回の body-missing fallback が 3 から 1 へ減り、
後続 body edit 2 回は `body_missing_skip_hits_delta_sum=10` で materialized compile attempt を避けた。

残る作業:

- direct call に限定した `.neplobj` selected callable body key を設計する。
- `stable link symbol`、selected callable body hash、generic instantiation hash、backend feature set を key に含める。
- function value / indirect call / `memo_call` は stable codegen artifact と Resource proof が揃うまで fail-closed に残す。

## 2026-06-01 direct-call `.neplobj` key schema checkpoint

`nepl-core::artifact` に `NeplObjDirectCallKey` を追加し、direct call に限定した
`.neplobj` selected callable body artifact の invalidation boundary を固定した。

key は `TypeId`、`Span`、`FileId`、`SourceMap`、typed HIR body を含めない。代わりに、
compiler identity、target/profile、stdlib content hash、backend feature set、stable link symbol、
materialized `neplmeta$...` symbol、target source key hash、selected callable body hash、
generic instantiation hash、dependency public surface hash、source capability policy hash、
private effect policy hash を含める。

`typecheck/public_surface.rs` には `materialized_callable_symbol_for_link_symbol` と
`public_callable_link_symbol_stable_hash` を追加し、materializer と `.neplobj` key が同じ
stable link symbol 規則を使うようにした。

この checkpoint は availability resolver / codegen fragment payload / `.nepllink` 実装ではない。
次は `NeplObjDirectCallKey` を `CompilerSession` の object availability store と
`PublicInterfaceArtifactInputs` の direct-call body resolver へ接続する。function value、indirect call、
`memo_call` は stable backend representation と Resource proof が揃うまで fail-closed のまま維持する。

## 2026-06-01 materialized body-missing kind checkpoint

`backend.codegen.materialized_function_body_missing` の収集結果を direct call、function value、
memoized function value に分類するようにした。diagnostic code は既存の body-missing code を維持し、
message に use kind を含める。

この checkpoint は `.neplobj` availability resolver そのものではない。次の resolver は direct call
だけを候補にし、function value、indirect call、`memo_call` は stable codegen artifact と Resource proof が
揃うまで body-missing / source fallback に残す。

## 2026-06-01 direct-call `.neplobj` availability input checkpoint

`PublicInterfaceArtifactInputs` に direct-call `.neplobj` availability slice を追加し、materialized callable
body-missing dependency を diagnostic 化する前に direct-call availability を見る境界を作った。

この checkpoint では compiler session から key を渡していないため、通常 compile は引き続き
source fallback / body-missing に倒れる。`.neplobj` key は codegen fragment payload と同時に渡された
場合だけ body-missing を解消できる authority として扱う。

direct call key は `MaterializedCodegenDependencyKind::DirectCall` にだけ適用し、function value、
indirect call、`memo_call` は同じ symbol が key に存在しても解決しない。

この availability slice は、2026-06-02 の payload checkpoint で破棄した。backend が payload を
消費しないまま diagnostic だけを消す経路になるため、backend/linker 登録済み token ができるまで
core pipeline は body-missing を fail-closed に維持する。

## 2026-06-02 direct-call `.neplobj` fragment payload checkpoint

key-only availability を使う代わりに、backend が呼び出せる body を表す
`NeplObjDirectCallFragmentArtifact` を追加した。`.neplobj` direct-call key は invalidation boundary であり、
backend が呼び出せる body ではない。key だけで body-missing を消すと、final codegen は materialized
symbol の body を持たないまま進むため、key と backend payload を同じ artifact にまとめた。

Wasm payload は `NeplObjWasmDirectCallFragment` として、params / results / function body bytes /
direct-call relocation を保持する。relocation は final module assembly で解決されるため、payload
作成時点では function index を固定しない。fragment hash は key stable hash、Wasm signature、body
bytes、正規化済み relocation を含める。重複 relocation offset と body 範囲外 offset は artifact
作成時点で拒否する。

この checkpoint でも `.neplobj` fragment は body-missing diagnostic を解決しない。次の作業は source
fallback / full compile で作った payload を same-session object store へ保存し、`PreparedProgram` /
wasm codegen が payload を function body set と relocation map に登録した場合だけ diagnostic を
消す backend 登録済み token を導入することである。

## 2026-06-02 wasm direct-call link-plan token checkpoint

`nepl-core::codegen_wasm` に `plan_neplobj_direct_call_fragments_for_wasm` を追加した。これは
`.neplobj` fragment payload を diagnostic 抑制へ使う API ではなく、現在の wasm assembly plan に
登録できるかを検査する backend authority である。

この plan は、`HirModule` の extern / user function から作る function index 空間へ `.neplobj`
fragment を追加したと仮定し、materialized symbol 衝突、backend feature set mismatch、relocation
target missing を拒否する。成功時は assigned function index、direct-call key hash、fragment hash、
resolved relocation を持つ `NeplObjWasmDirectCallLinkPlanToken` を返す。

raw body bytes を `CodeSection` へ投入し relocation を patch する実装はまだないため、この token は
body-missing を消さない。次の作業は、source fallback / full compile で作った payload を session store
へ保存し、その後 token と actual code insertion を同じ backend 境界にまとめることである。

## 2026-06-02 checkpoint 6

`generate_wasm_with_neplobj_direct_call_fragments` を追加し、`.neplobj` direct-call fragment を
Wasm backend の `FunctionSection` / `CodeSection` へ実際に投入できる最小境界を実装した。

この API は `plan_neplobj_direct_call_fragments_for_wasm` で symbol 衝突、backend feature set、
relocation target、周辺 HIR / extern signature を確認したうえで、resolved relocation の function
index を fragment body bytes の call immediate へ patch する。LEB128 immediate は長さが変わる場合も
後方から差し替え、fragment が古い function index を固定した object にならないようにした。

`PublicInterfaceArtifactInputs` は `NeplObjDirectCallFragmentArtifact` slice を受け取れるようになった。
materialized callable body-missing の抑制は direct call だけに限定し、function value、indirect call、
`memo_call` / memoized function value は同じ materialized symbol を持っていても source fallback に残す。
これは高階関数 identity と private cache proof を `.neplobj` direct-call MVP が暗黙に代替しないための
fail-closed 境界である。

この checkpoint でも issue は open のまま維持する。まだ source fallback / full compile から
`NeplObjDirectCallFragmentArtifact` を session object store へ保存する経路と、Web / loader から
`PublicInterfaceArtifactInputs` へ渡す実運用経路は未接続である。

## 2026-06-01 selected body hash authority checkpoint

`.neplobj` key の `selected_callable_body_hash` が使う authority として、
`resource_function_body_stable_hash` を Resource API から公開した。これは既存の Resource summary
value cache が使う Resource IR body hash を wrapper として公開するものであり、object key 側で
`TypeId` / `Span` / 一時値 ID / storage ID の正規化を再実装しない。

source body を skip した `.neplmeta` compile ではこの hash は得られない。object store は source fallback
または full compile で selected callable を Resource IR へ下げたときに hash を作り、次回の
direct-call availability key に保存する。
