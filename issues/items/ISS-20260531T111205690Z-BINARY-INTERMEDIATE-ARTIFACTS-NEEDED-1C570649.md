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
target: "nepl-core/src/loader.rs; nepl-core/src/compiler.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_performance_cache_design.md"
---

# ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649: binary intermediate artifacts needed for incremental compile

## 概要

現状の NEPLg2 は typed HIR、Resource summary、compiled output cache を主に process-local memory に保持している。JVM の `.class` や C 系の `.o` に相当する、session をまたいで再利用できる永続 binary intermediate artifact はまだ持っていない。

## 対象

- `nepl-core/src/loader.rs; nepl-core/src/compiler.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_performance_cache_design.md`

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

補助 artifact として、native CLI の `--check` には `.neplcheck` を使う。これは `.neplproof` や
`.neplobj` と違い、部分的な証明や code fragment を保持しない。前回成功した完全一致入力に対して、
読み込まれた source manifest を loader 前に照合し、同じ compiler binary / target / profile /
stdlib root / source set の場合だけ成功結果を再利用する exact success cache である。source set が
1 つでも違う場合は通常 compile へ fail-closed に戻る。

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

## 2026-06-02 content-addressed stdlib dependency aggregate cache checkpoint

`LoaderSessionCache` に、通常 session と bundled stdlib の content-addressed session を分ける境界を
追加した。Web `CompilerSession` は bundled stdlib 全体の content hash を cache namespace に使うため、
同一 session 内で同じ stdlib source を読む dependency aggregate query は path/source hash だけで
閉じた結果を再利用できる。

通常 session では従来どおり source hash と child aggregate hash、または module public surface hash と
child aggregate hash を key に含める。closed-source key は child hash を持たないが、stdlib content hash
namespace が変わらない限り依存先内容も変わらない Web bundled stdlib にだけ opt-in する。
stdlib overlay、non-stdlib edge、mutable provider ではこの shortcut を使わない。

この checkpoint は loader-level の固定費削減であり、永続 `.nepl...` artifact format や `.neplobj`
object store の完成ではない。issue は open のまま維持する。残件は source fallback / full compile から
`NeplObjDirectCallFragmentArtifact` を same-session object store へ保存し、Web / loader が次回 compile で
`PublicInterfaceArtifactInputs` へ渡す経路、bundled stdlib `.neplmeta` / `.neplproof` preseed、persistent
artifact codec、そして `memo_call` の PrivateCache proof である。

`tmp/artifact-closed-source-aggregate-cache-20260602-r3.json` では、cold base `compile_ms=423`、
warm store probe `compile_ms=230`、body edit candidate `compile_ms=206`、body edit repeat
`compile_ms=185` だった。body edit はまだ 0.1 秒以下ではないため、`.neplobj` / `.nepllink` と
stdlib preseed artifact の残件を継続する。

## 2026-06-02 same-session `.neplobj` direct-call fragment store checkpoint

`NeplObjDirectCallFragmentStore` を追加し、direct-call fragment payload を same-session で保持し、
Web `CompilerSession` から `PublicInterfaceArtifactInputs` へ渡せる経路を接続した。store は
target/profile、stdlib content hash、target source key、dependency public surface hash、source
capability policy hash、backend feature set、private effect policy hash、link symbol、空 generic
instantiation hash を照合する。materialized symbol だけでは hit しない。

body-missing negative skip は、同じ edge context の fragment candidate が store にある場合には
materialized edge probe を省略しない。これは source fallback / full compile で fragment producer が
入った後に、古い skip entry が object hit を隠さないための境界である。

この checkpoint でも issue は open のまま維持する。まだ fragment producer は存在せず、
source fallback / full compile 後に `resource_function_body_stable_hash` と wasm relocatable lowering から
`NeplObjDirectCallFragmentArtifact` を生成して store へ保存する経路は未実装である。`FnValue`、
`CallIndirect`、`MemoizedFunctionValue`、`memo_call` は direct-call fragment の対象外として
fail-closed に残す。

## 2026-06-02 direct-call `.neplobj` leaf fragment producer checkpoint

source fallback / full compile 成功時に、checked HIR から direct-call `.neplobj` fragment を生成して
same-session object store へ保存する最小 producer を追加した。Web `CompilerSession` は
materialized public surface と loader edge probe から `NeplObjDirectCallFragmentExportRequest` を作り、
full source fallback compile へ渡す。

producer は call relocation を持たない leaf function に限定する。direct call、indirect call、
function value、memoized function value、string literal、private-cache intrinsic、generic function、
raw wasm/LLVM body は fragment を返さない。省略は cache miss と同じ扱いであり、diagnostic や
compile 成功可否を変えない。

`selected_callable_body_hash` は HIR から作らず、`resource_function_body_stable_hash` を使う。
request がある場合だけ final codegen HIR を Resource IR へ lowering し、Resource summary value cache
と同じ body hash authority を `.neplobj` key に使う。これにより `TypeId`、`Span`、temporary id、
`FileId` を long-lived artifact key に入れない方針を維持する。

この checkpoint でも issue は open のまま維持する。残件は、direct-call relocation producer、
generic instantiation hash、string/data relocation、raw wasm/LLVM body の relocatable representation、
persistent `.neplobj` codec、function value / memoized function value backend、`memo_call` PrivateCache
mask proof accepted path である。

## 2026-06-02 direct-call `.neplobj` relocation producer checkpoint

direct-call `.neplobj` producer を call relocation 付き function へ広げた。producer は request された
public callable link symbol と full/source compile 済み HIR の function name を対応付け、callee が
同じ request set に含まれる public direct call の場合だけ placeholder function index を割り当てる。
private helper、unrequested callable、builtin、trait dispatch は stable public relocation target を
持たないため fragment を生成しない。

relocation offset は wasm body bytes を後から scan して作らない。`lower_user` が作る
`Instruction::Call(placeholder_index)` を `wasm_encoder::Function` へ emit する直前に
`byte_len() + 1` を記録し、call immediate offset と target `PublicCallableLinkSymbol` を
`NeplObjWasmDirectCallRelocation` に保存する。consumer 側は引き続き offset 直前が call opcode かを
検査し、壊れた persistent payload を fail-closed に拒否する。

追加 regression:

- `neplobj_wasm_export_produces_direct_call_relocation_fragment`
- `neplobj_wasm_export_rejects_direct_call_to_unrequested_target`
- `neplobj_wasm_export_rejects_non_direct_call_leaf_boundaries`
- `neplobj_wasm_codegen_rejects_relocation_offset_without_call_opcode`

この checkpoint でも issue は open のまま維持する。残件は generic instantiation hash、
string/data relocation、raw wasm/LLVM body の relocatable representation、persistent `.neplobj` codec、
function value / memoized function value backend、`memo_call` PrivateCache mask proof accepted path、
bundled stdlib `.neplmeta` / `.neplproof` preseed である。

## 2026-06-02 Web `.neplobj` direct-call store regression checkpoint

`tests/compiler/tree/20_compiler_session_outputs_cache.js` に same-session `.neplobj` direct-call fragment
store の実運用 regression を追加した。fixture は `core/char` の `char_utf8_cont_byte` を使い、body edit
を繰り返して materialized body-missing fallback から fragment store、次回 lookup hit まで進める。

検査する counter:

- `nepl_obj_direct_call_fragment_store_stores`
- `nepl_obj_direct_call_fragment_store_lookup_hits`
- `nepl_obj_direct_call_fragment_store_lookup_fragments_returned`
- `nepl_meta_materialized_compile_body_missing_fallbacks`
- `nepl_obj_candidate_last_body_missing_surfaces`

store hit 後は lookup hit / returned fragment が増え、body-missing fallback は増えないことを確認する。
これにより body-missing skip cache が object hit を隠さず、direct-call `.neplobj` fragment が
`PublicInterfaceArtifactInputs` へ渡ることを Web 側から固定する。

## 2026-06-02 fallback root-cause checkpoint

same-session `.neplobj` direct-call store の実運用経路を通した後、materialized compile は
`backend.codegen.materialized_function_body_missing` 以外の blocker を露出した。今回の checkpoint では
diagnostic を丸めず、各 blocker を artifact authority の不整合として修正した。

修正した境界:

- `TypeKind::Named` が stable nominal identity を持つ場合、nominal definition hash は
  `stable_key_component()` を使う。これにより `.neplmeta` materializer が forward nominal
  placeholder を含む surface を `NominalDefinitionHashUnavailable` として拒否しない。
- loader の import/prelude edge は、再帰ロード中に見つかった child artifact をただちに root の
  materialized input へ漏らさない。parent edge の materialize が成功した場合は staged child
  artifact を discard し、parent edge が source fallback した場合だけ commit する。
- typecheck driver は materialized public surface の `source_path` を使い、同じ source file の
  source AST 定義を file id だけでなく path boundary でも skip する。impl surface も
  `source_path` を持ち、source impl と materialized impl の二重登録を防ぐ。
- `.neplobj` direct-call fragment store lookup は relocation dependency closure を返す。callee
  fragment が欠ける caller fragment は返さず、link invalid ではなく source fallback に戻る。
- wasm `.neplobj` producer は call / call_indirect を含まない raw wasm body を relocation-free leaf
  fragment として保存できる。raw wasm body に call boundary がある場合は引き続き fail-closed にする。

`tmp/materialized-raw-wasm-neplobj-20260602-rerun.json` では、`core/char` fixture の
cold base `compile_ms=387`、warm store probe `compile_ms=221`、body edit candidate `compile_ms=21`、
body edit repeat `compile_ms=21` になった。fallback は
`backend.codegen.materialized_function_body_missing` の 1 件だけで、
`type.public_surface.materializer.nominal_definition_rejected`、`resolve.item.name_conflict`、
`backend.wasm.neplobj_direct_call_link_invalid` は出ていない。

この issue は open のまま維持する。今回の変更は same-session direct-call `.neplobj` と
materialized public surface の fallback blocker を取り除く checkpoint であり、persistent
`.nepl...` codec、bundled stdlib `.neplmeta` / `.neplproof` preseed、generic instantiation hash、
string/data relocation、raw LLVM body、function value / memoized function value backend、
`memo_call` PrivateCache proof は未完了である。

## 2026-06-02 explicit `.neplmeta` preseed API checkpoint

Web `CompilerSession` に明示 preseed API を追加した。`preseed_nepl_meta_artifacts_for_source` は
root source の import / prelude edge probe を収集し、到達した bundled stdlib dependency module を
typecheck して `.neplmeta` public interface artifact を same-session store に保存する。
`preseed_nepl_meta_artifacts_for_source_with_profile` は同じ処理を debug / release profile 指定で行う。

通常の `prewarm_loader_cache_for_source` は loader / parser query だけを warm する契約のまま維持する。
preseed は typecheck を伴うため、通常 compile path から暗黙には呼ばない。これは compile 時間を
prewarm に移しただけの測定を避け、将来の persistent `.neplmeta` codec / IndexedDB / disk artifact が
同じ invalidation boundary で compile 前に artifact を用意できるかを測るための入口である。

Web regression では、fresh `CompilerSession` が暗黙の `.neplmeta` dependency artifact を持たないこと、
明示 preseed が store entry を増やすこと、preseed 後の初回 compile が missing artifact ではなく
pre-typecheck edge projection を試すことを固定した。

`nodesrc/bench_materialized_compile_fallbacks.js` には `--preseed-neplmeta` を追加し、JSON の
`preseed` に availability、artifact count、elapsed ms、error を出すようにした。実測では
`tmp/neplmeta-preseed-api-baseline-20260602.json` が cold base `compile_ms=438`、warm store probe
`compile_ms=229`、body edit repeat `compile_ms=22` で、`tmp/neplmeta-preseed-api-enabled-20260602.json`
は `preseed.artifact_count=40`、`preseed.elapsed_ms=373`、preseed 後 cold base `compile_ms=261`、
warm store probe `compile_ms=23`、body edit repeat `compile_ms=23` だった。

この checkpoint は in-memory preseed の入口であり、issue は open のまま維持する。残件は persistent
`.neplmeta` codec、bundled stdlib artifact embedding、`.neplproof` preseed、generic/string/raw body
を含む `.neplobj` の永続化、function value / memoized function value backend、`memo_call` PrivateCache
proof である。

## 2026-06-02 RPN `.neplproof` cold-base checkpoint

RPN cold base の再測定により、`.neplmeta` / `.neplobj` の小 fixture 改善だけでは
stdlib-heavy workload の初回 compile 目標に届かないことを再確認した。

`examples/rpn.nepl` の native release check は `resource_static_check=6172ms` で、その内訳は
`resource_initialized_moves=5320ms`、`resource_initialized_i32_scalar_summaries=1330ms`、
`resource_initialized_raw_init_summaries=2315ms`、`resource_initialized_function_checks=1609ms`、
`resource_owner_obligations=736ms` だった。release Web cold base は `compile_ms=9283` で、
`resource_static_initialized_moves=6626.938ms` と `resource_static_owner_obligations=1455.416ms`
が支配的だった。

局所的な leaf pruning 試行では、enum variant projection の不可能 i32 scalar facts を落としても
RPN native release が `resource_static_check=7455ms` へ悪化した。したがって、この issue の次段階は
proof 探索を少しずつ削ることではなく、既に same-session で再利用できる Resource summary proof を
cross-session / cold-start の `.neplproof` artifact として利用できる形にすることである。

永続 `.neplproof` codec は、現在の in-memory `ResourceSummaryProofSnapshot` をそのまま binary に
するものではない。snapshot は serialization schema ではなく、stable entry map も private payload である。
必要な境界は次である。

- header を先に decode し、schema / compiler identity / target / profile / stdlib hash /
  dependency public surface / Resource summary namespace / source capability policy / private effect policy
  が一致しない artifact では payload を読まない。
- stable entry は `TypeId`、`Span`、`FileId`、`SourceMap`、diagnostic、compile-local replay plan を
  含めず、既存 replay API が現在の `TypeCtx` と source capability policy へ再投影できる単位だけにする。
- generic type argument key と function body hash を entry key に含め、stdlib generic の具体化違いで
  stale proof を共有しない。
- Web は IndexedDB / bundled artifact、CLI は disk cache を host storage とし、`nepl-core` は
  serialization format ではなく decoded artifact の header / stable entry を fail-closed に検査する。

この checkpoint は `.neplproof` 永続 codec の実装前設計であり、issue は open のまま維持する。

2026-06-02 の `.neplproof` preseed report checkpoint で、same-session preseed の accepted /
existing / conflict / compatibility reject を `CompilationArtifact` と Web `CompilerSession` stats へ
公開した。これは永続 codec そのものではないが、header mismatch や cache hit / stdlib overlay bypass
で stale な preseed 観測値を残さないための artifact boundary である。persistent / bundled
`.neplproof` を実装する際は、この report を disk / IndexedDB / build-time bundled artifact の
reject visibility へ接続する。

2026-06-02 の RPN `str_trim` scan helper split checkpoint では、stdlib 側の関数構造を整理して
native release RPN の `resource_static_check` を後続 run で `5787ms` / `5397ms` まで下げた。
これは `str_trim` の loop / branch merge を 1 つの関数へ集中させないための stdlib 構造改善であり、
binary intermediate artifact の完了ではない。

この issue は open のまま維持する。native `--check` はまだ actual `.neplproof` artifact を cold start
で preseed できず、`apply_op` / `dealloc_raw` / `sb_append_result` などの stdlib-heavy proof は初回
compile で構築されている。次の artifact 側作業は、empty cache ではなく、final initialized pass、
owner obligation、raw-init / i32 scalar summary の stable entry を header-first / fail-closed な
`.neplproof` codec と bundled / persistent preseed へ接続することである。

2026-06-02 の RPN operator / builder helper split checkpoint では、RPN source と StringBuilder helper
の構造を Resource IR が扱いやすい粒度へ整理し、native release RPN stage-only の
`resource_static_check` を同 branch baseline `5870ms` / `5900ms` から `5372ms` / `4927ms` へ下げた。
per-function timing では `apply_op` raw-init が `611ms` から `426ms` へ下がり、`sb_append_result`
wrapper は `3ms` になった。一方で、`dealloc_raw` raw-init `498ms`、`sb_append_non_empty_result`
i32 scalar `320ms`、ByteBuilder 系 helper、owner summary / obligation はまだ cold start で再構築される。

この checkpoint は binary intermediate artifact の完了ではない。局所的に関数を分けても RPN cold base
はまだ 5 秒前後であり、0.5 秒未満へ入れるには actual `.neplproof` artifact を cold start 前に
preseed する必要がある。`.neplproof` 側では、ResourceSummaryProofSnapshot をそのまま保存せず、
header-first decode、stable entry codec、generic type-argument key、source capability policy hash、
private effect policy hash を持つ fail-closed な `.neplproof` codec と bundled / persistent preseed を
次の主経路として維持する。

2026-06-02 の proof-backed check gate checkpoint では、actual `.neplproof` codec の前段として
native check 用の core wrapper と cache activation gate を追加した。
`check_module_with_source_map_resource_summary_value_cache_and_neplproof` は artifact emission へ進まず、
通常 `--check` と同じ Resource IR static gate を通る。`ResourceSummaryValueCacheActivation::OnlyAfterAcceptedPreseed`
を指定した場合は、matching header でも empty snapshot なら cache を起動しない。これにより、
disk / bundled artifact loader を追加する次 checkpoint で、empty cache を渡しただけの RPN cold base
regression を起こさず、accepted usable entry を持つ artifact だけを cold path へ接続できる。

2026-06-02 の RPN `print_i32` allocation-free checkpoint では、RPN が `alloc/string/integer/format`
由来の StringBuilder / ByteBuilder proof graph を error path と整数表示のために引き込んでいたため、
`print_i32` を digit byte 直接出力に変え、RPN の stack count error も文字列連結から直接出力へ変えた。
native release RPN stage-only 5 run の median は `resource_static_check=2922ms -> 1539ms`、到達関数は
`307 kept=304 -> 271 kept=268` へ下がった。`origin/main` `c3066b8a` merge 後の current tree では
`resource_static_check` median は `1500ms` だった。

ただし、この改善は reachable stdlib graph を小さくする局所構造改善であり、binary intermediate artifact
の完了ではない。RPN cold base はまだ 0.5 秒未満ではなく、final initialized pass、owner obligation、
raw-init / i32 scalar summary を cold start 前から使える `.neplproof` codec / bundled preseed が引き続き
主経路である。

2026-06-02 の `.neplproof` persistent codec checkpoint で、native CLI 向けの Resource proof artifact
保存・読込を実装した。ファイル名の系統は `.neplproof` とし、`.class` / `.o` を単純に模倣するのではなく、
Resource IR proof summary の再利用に特化する。

実装済みの境界は次の通り。

- fixed header を payload より先に decode し、互換性がない artifact では payload を読まない。
- container schema `2` では fixed header に payload hash も持たせ、header は一致するが payload bytes が壊れている artifact も decode 前に拒否する。
- payload は stable entry map だけで、`TypeId`、`Span`、`SourceMap`、diagnostic、compile-local replay plan を含めない。
- `nepl-core` は no-std / alloc のまま、serde/postcard codec と preseed 判定だけを持つ。
- disk path、temporary write、rename、環境変数、compiler executable identity は `nepl-cli/src/proof_cache.rs` に閉じる。
- RPN 実測で raw-alias stable entry は再計算より高いため、native disk proof 経路では raw-alias kind を永続 proof から外す。
- `.neplproof` は compiler が生成した local build cache を信頼する設計である。未信頼の CI cache / workspace artifact を扱う場合は `NEPL_DISABLE_PROOF_CACHE=1` で通常検査へ戻す。

RPN proof-backed cold base は `resource_static_check=1417ms / 988ms / 911ms / 1304ms / 1017ms`、
中央値 `1017ms` だった。proof bootstrap は `resource_static_check=2771ms` で、生成された `.neplproof`
は約 `2.17MB` だった。これは `.neplproof` 境界の実装 checkpoint であり、issue は open のまま維持する。
残件は bundled stdlib `.neplproof` preseed、owner obligation pass-level snapshot、`.neplobj` の
generic / string-data / raw body / function value / memoized function value 対応である。

2026-06-02 の `.neplproof` pass snapshot checkpoint では、関数単位 stable entry map に加えて、
stable key / function fingerprint だけの replay snapshot と deferred counter だけの pass snapshot を
payload に含めた。これは summary 本体や diagnostic を保存する `.neplhir` / `.neplobj` ではなく、
現在の `TypeCtx` へ再投影できる stable proof entry を高速に見つけるための `.neplproof` 内部 index である。
schema は `2` へ上げ、古い `.neplproof` は fail-closed に拒否する。

この変更で RPN proof-backed `resource_static_check` median は約 `416ms` になった。ただし native CLI
wall-clock はまだ約 `0.9-1.0s` であり、binary intermediate artifact issue は open のまま維持する。
次の artifact 側作業は bundled stdlib `.neplmeta` / `.neplproof` preseed、typecheck/interface artifact、
bootstrap proof generation 短縮、`.neplobj` の generic / string-data / raw body / function value /
memoized function value 対応である。

2026-06-02 の `.neplproof` no-op rewrite skip checkpoint では、native CLI の disk proof cache が
preseed artifact を完全に再利用した場合、同じ `.neplproof` bytes を再度 export/store しない policy を
追加した。bootstrap と reject / conflict / recomputed stable work のある run は保存を維持するため、
stale artifact を残す方向には働かない。

RPN no-stage wall-clock 15 run の中央値は `1048.574ms` であり、stage timing 付きでは
`resource_static_check` が主に `404-438ms`、`resource_typecheck` が主に `153-171ms` だった。したがって、
store skip は同一 artifact への不要 I/O と競合窓を減らす正しい policy だが、binary intermediate
artifact issue の本筋はまだ loader / typecheck / proof decode を含む base compile 前半である。次は
bundled stdlib `.neplmeta` / `.neplproof` preseed と typecheck/interface artifact を優先する。

2026-06-02 の RPN loader/process-directives checkpoint では、native CLI の cold base を
`NEPL_CLI_STAGE_TIMING=1` で分解し、`.neplproof` read が RPN では約 `0.7-0.9ms` しかないことを確認した。
同じ release CLI の proof-backed run では、stage timing 付き 5 run の wall-clock 中央値が `968.434ms`、
no-stage 14 run の wall-clock 中央値が `965.283ms` だった。階層は `loader_load=421.186ms`、
`check_pipeline=539.710ms`、`resource_typecheck=130ms`、`resource_static_check=379ms` であり、
binary intermediate artifact の次の削減対象は proof cache I/O ではなく loader / dependency surface /
typecheck 境界である。

一時的な loader 詳細計測では、`loader_load` 約 `450ms` のほとんどが `load_file_tree` で、root
`examples/rpn.nepl` の `process_directives` が約 `401ms` を占めた。`process_directives` と
`process_directives_with` は所有済み `Module` の `directives` / `root.items` を clone せず move して
再構築する形へ修正し、`examples/rpn.nepl` には明示 import と整合する `#no_prelude` を追加した。
それでも wall-clock はまだ約 `0.96s` であり、この checkpoint は `.neplmeta` / typed interface artifact
の必要性を強める結果である。

したがって、この issue の次 checkpoint は、native CLI `--check` が bundled stdlib `.neplmeta` または
materialized typed public surface から依存先 environment を構成し、stdlib body merge と依存先再
typecheck を避ける設計・実装である。`.neplobj` や codegen fragment より前に、`import` / `prelude` を
interface boundary として扱えるようにする。
