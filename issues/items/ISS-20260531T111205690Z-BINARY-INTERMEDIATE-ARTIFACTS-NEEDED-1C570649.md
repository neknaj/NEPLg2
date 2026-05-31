---
id: ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649
title: "binary intermediate artifacts needed for incremental compile"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
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
