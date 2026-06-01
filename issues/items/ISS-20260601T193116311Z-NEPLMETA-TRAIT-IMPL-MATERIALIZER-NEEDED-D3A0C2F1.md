---
id: ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1
title: ".neplmeta trait and impl materializer needed for prelude capability surface"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-02
target: "nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/materializer.rs; nepl-core/src/typecheck/public_surface.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/artifact.rs; nepl-web/src/lib.rs"
---

# ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1: .neplmeta trait and impl materializer needed for prelude capability surface

## 概要

`.neplmeta` stdlib dependency artifact producer により、import / prelude edge probe は missing artifact ではなく projection blocker まで到達できるようになった。次の root blocker は `std/prelude_base` が `core/traits/copy` を `@merge` し、`Clone` / `Copy` impl surface が private capability trait の stable identity を持てないことである。

## 対象

- `nepl-core/src/typecheck/materializer.rs`
- `nepl-core/src/typecheck/public_surface.rs`
- `nepl-core/src/artifact.rs`
- `nepl-web/src/lib.rs`

## 根拠

- 3 回目の `std/prelude_base` edge probe は `MissingArtifact` を増やさず、`Projection` / `PublicSurfaceBlocker` まで進む。
- `last_pre_typecheck_edge_probe_projection_blocker_reason_code=7` は `MissingTraitIdentity` を示す。
- `last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code=5` は blocker が `Impl` surface 由来であることを示す。
- `stdlib/std/prelude_base.nepl` は `core/traits/copy` を `@merge` import する。
- `core/traits/copy` は `copy/primitive` を public import し、そこで `Clone` / `Copy` trait と primitive impl 群を定義する。
- `trait Clone` / `trait Copy` は public export API ではないが、prelude capability registration には必要な semantic surface である。

## 問題

現行 materializer は public callable surface の復元だけを MVP としており、trait table と impl table を `.neplmeta` から復元しない。そのため、stdlib prelude の capability impl を body なしで読み込めず、base compile は依然として stdlib source merge / typecheck に戻る。

`Clone` / `Copy` を単に `pub trait` にする、または `std/prelude_base` を特殊扱いするのは根本対応ではない。必要なのは、公開 API として export されないが import 先の型検査に必要な semantic surface を `.neplmeta` artifact 内で安全に復元することである。

## 影響

`.neplmeta` dependency artifact が生成されても、prelude capability surface を materialize できないため、stdlib prechecked artifact による base compile 短縮へ進めない。特に `Copy` / `Clone` capability はほぼ全 program の default prelude に関わるため、この blocker を解消しない限り 0.5 秒未満 base compile と 0.1 秒未満 warm recompile の主要経路が閉じない。

## 修正方針

Trait / impl materializer を callable materializer とは別 authority として追加する。

- `PublicTraitSurface` を current session の trait table へ復元する。
- `PublicImplSurface` を current session の impl table / capability registration へ復元する。
- public export ではない dependency-local trait identity を、artifact 内の semantic surface として安全に扱う境界を定義する。
- `Clone` / `Copy` のような capability trait は名前だけではなく source path、arity、definition hash で照合する。
- `Impl` target type、generic parameter、trait bound、trait application を `PublicTypeParamRef` / stable trait identity / stable nominal identity から復元する。
- 復元できない named type、type application、field accessor、re-export projection は従来通り fail-closed にする。
- `.neplmeta` 由来 callable は引き続き `def_id=None` の direct call 専用とし、`@func` / `memo_call @func` / indirect call の stable function identity には使わない。

## 検証

- `std/prelude_base` edge probe が `MissingTraitIdentity` / `Impl` blocker を解消し、次の未対応 blocker または projection success まで進む。
- `Clone` / `Copy` capability が `.neplmeta` materializer 経由でも typecheck の copy/clone 判定に使える。
- 同名 trait を別 module から materialize しても source path / definition hash mismatch で拒否される。
- private trait identity を public API として export しない。
- `memo_call` / private cache proof / function value identity は `.neplmeta` trait/impl materializer で bypass されない。
- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`
- `node tests\compiler\tree\run.js`

## 2026-06-01 semantic trait surface checkpoint

`Clone` / `Copy` のような private capability trait を public export に昇格せず、semantic
support surface として `.neplmeta` に保持する境界を追加した。`TypedPublicSurfaceEntry` は
`exported` flag を持ち、export surface は `exported=true` の entry だけを local export として扱う。

この変更により、`std/prelude_base` edge probe の blocker は `MissingTraitIdentity`
(`reason_code=7`) から `MissingNamedTypeIdentity` (`reason_code=3`) へ進んだ。entry kind は
引き続き `Impl` (`entry_kind_code=5`) である。

この issue の trait identity 部分は前進したが、trait/impl materializer はまだ完了していない。
次の作業では、nominal type identity / type application を復元したうえで、validated `TraitInfo`
と `ImplInfo` を current session の semantic registry へ注入する必要がある。

## 2026-06-01 unsupported export kind checkpoint

backend scalar named terms を `BackendScalarType` domain で復元し、`i128` / `u128` の capability
impl を `core/math/int128/types` へ移したことで、`std/prelude_base` の stored dependency artifact は
`MissingNamedTypeIdentity` で止まらず一部 projection success まで進むようになった。

残る reject は `PublicSurfaceBlocker` ではなく `UnsupportedExportKind` (`reject_code=11`) である。
これは `.neplmeta` projection が callable export だけを materializer 入力として受け入れる MVP
境界に到達したことを示す。`Clone` / `Copy` trait、`i128` / `u128` struct、関連 impl のような
non-callable surface を `Env` の visible export と semantic registry へ分けて復元する必要がある。

この checkpoint 時点では、次の実装単位を以下のように見積もっていた。

- `Trait` / `Struct` / `Enum` export kind を名前空間へ安全に投影する。
- semantic-only `Impl` を public export に混ぜず、validated `ImplInfo` と capability registration へ注入する。
- `PublicTypeTerm::Named { identity: Some(...) }` と `Apply` の復元を先に通し、impl target を名前だけで対応させない。
- `UnsupportedExportKind` の stats は public surface blocker ではないため、blocker reason / entry kind code を 0 のまま保つ。

## 2026-06-01 non-callable export projection checkpoint

`Trait` / `Struct` / `Enum` local export は `.neplmeta` projection で kind を保ったまま
`TypedPublicSurfaceTable` へ戻せるようになった。これにより、`UnsupportedExportKind` は
root pre-typecheck probe と一部 stdlib edge probe の主要 blocker ではなくなった。

この checkpoint は projection 層の前進であり、trait / impl materializer 完了ではない。
`typecheck/materializer` はまだ callable 以外の `PublicSurfaceShape` を current session の
`TypeCtx` / trait table / impl table へ登録しない。また、artifact MVP gate は `Impl` surface を
含む場合 `UnsupportedImplLookup` に残る。したがって、この issue の残件は以下へ絞られた。

- `PublicTraitSurface` を stable identity で trait table へ復元する。
- `PublicStructSurface` / `PublicEnumSurface` を stable nominal identity で `TypeCtx` へ復元する。
- semantic-only `PublicImplSurface` を visible export に混ぜず、validated impl registry と capability registration へ注入する。
- impl target / trait application の `Named` / `Apply` materialize は、nominal issue と同期して進める。

## 2026-06-01 nominal and trait definition materializer checkpoint

semantic table を渡す `.neplmeta` materializer 入口で、`PublicStructSurface` /
`PublicEnumSurface` / `PublicTraitSurface` を current session へ staging できるようになった。
`TraitInfo` は `TraitStableIdentity` を持ち、`.neplmeta` 由来の trait と既存 trait の同名衝突を
名前だけで握りつぶさない。trait method surface 内の `TraitSelf` は trait materializer の内側でのみ
`self_ty` へ戻す。

この checkpoint により、trait / struct / enum definition の復元は前進した。ただし、この issue は
まだ open のまま維持する。`PublicImplSurface` を `ImplInfo` と capability registration へ注入する
処理は未実装であり、`Clone` / `Copy` capability が `.neplmeta` materializer 経由で実際の copy/clone
判定に使えるところまでは到達していない。

post-implementation review で見つかった `definition_hash` 信頼境界の穴もこの checkpoint で塞いだ。
trait surface は `PublicTraitIdentity.definition_hash` をそのまま採用せず、capability と method surface
から再計算して照合する。重複 method 名は `BTreeMap` への投入で潰さず、materializer reject として
扱う。

## 2026-06-01 semantic impl materializer checkpoint

semantic table を渡す `.neplmeta` materializer 入口で、`PublicImplSurface` を `ImplInfo` へ
staging できるようになった。impl は public export へ混ぜず、artifact の semantic support surface として
扱う。`PublicTraitRef.identity` は必須であり、`TraitInfo.stable_identity` と source path / name /
arity / definition hash が一致する場合だけ `TraitApplication` へ戻す。

`Clone` / `Copy` / `Drop` capability target は、staging 中ではなく全 entry 成功後にだけ
`TypeCtx` へ登録する。`Copy` impl は対応する `Clone` impl がなければ reject し、`Drop` impl は
copyable target と重なる場合に reject する。duplicate impl は同一 trait application と target type
pattern の重なりとして検出し、同一 impl の再投影は `AlreadyPresent` として扱う。

artifact MVP gate の `UnsupportedImplLookup` は外した。これにより、impl surface を含む `.neplmeta`
artifact も projection / preflight を越えられる。

この issue はまだ open のまま維持する。理由は、materializer の部品は揃ったが、dependency import /
prelude の通常 typecheck body skip 経路へ接続して base compile time を更新する作業が未完了である。

残件:

- dependency artifact から得た `TypedPublicSurfaceTable` を import / prelude の `Env` / semantic registry
  構築へ接続する。
- `std/prelude_base` edge probe の `UnsupportedImplLookup` 解消後の次 blocker と base compile time を
  実測する。
- `memo_call` / private cache proof / function value identity はこの materializer で bypass しない。

## 2026-06-01 typecheck materialized dependency surface checkpoint

`TypedPublicSurfaceTable` を source body 検査前の `Env` / nominal table / trait table / impl table へ
注入できる typecheck API を追加した。これにより、loader が dependency artifact projection を
検証済み入力として渡せる最小境界ができた。

この API は `module_path` と `file_id` を要求し、現在の `SourceMap` 上で同じ path を持つ file と
一致しない場合は fail-closed に拒否する。import visibility は binding の `file_id` に依存するため、
body skip でも dependency target file slot を予約する契約を先に固定する必要がある。

また、materialized dependency は root module の public signature / public surface には混ぜない。
`StructInfo` / `EnumInfo` / `ImplInfo` に origin span を持たせ、materialized file id を export 生成時に
除外することで、「名前解決と semantic registry には使うが、現在 module の `.neplmeta` export には
しない」境界を固定した。

この checkpoint でも issue は open のまま維持する。理由は、loader / web 側で実際に import /
prelude body merge を省く経路は未接続であり、直接呼び出される dependency function body を落とすには
`.neplobj` / codegen fragment 相当の authority か source fallback 判定が必要だからである。

残件:

- loader が dependency source body を merge せず、target file id と projected surface だけを渡す経路を作る。
- callable body が codegen に必要な dependency は `.neplobj` ができるまで source fallback する。
- prelude capability だけを materialize する case で base compile time を実測する。
- `memo_call` / private cache proof / function value identity は引き続き `.neplmeta` materializer で bypass しない。

## 2026-06-01 compiler public-interface pipeline checkpoint

compile / prepare pipeline は `PublicInterfaceArtifactInputs` を受け取れるようになった。
この入力は dependency public surface hash、root module surface、artifact projection 済み
`MaterializedPublicSurfaceInput` をまとめる。既存 wrapper は空の materialized surface を渡すため、
通常の source compile は従来どおりである。

`.neplmeta` 由来 callable は stable link symbol を持つが、関数本体、Resource proof、
backend code fragment を持たない。したがって materialized surface を使った prepare では、
`neplmeta$...` callable が HIR の直接呼び出し、function value、memoized function value、indirect call
として codegen 入力へ到達した場合、`backend.codegen.materialized_function_body_missing` 診断で止める。
これは `.neplobj` が入るまでの source fallback boundary である。

loader の `NeplMetaDependencyEdgePreTypecheckProbe` には `target_file_id` を追加した。
`MaterializedPublicSurfaceInput` は SourceMap 上の `module_path` / `file_id` 一致を authority にするため、
後続の Web bridge は path 文字列を再探索せず、loader が登録した target file slot を使える。

この checkpoint でも issue は open のまま維持する。Web / loader の通常 import / prelude body merge を
実際に省く接続は未完了であり、selected dependency callable が必要な case は `.neplobj` 実装まで
source fallback へ戻す必要がある。

残件:

- `.neplmeta` store projection 成功結果を Web / loader で `MaterializedPublicSurfaceInput` へ変換する。
- import / prelude edge の body skip を artifact-ready かつ selected callable body 不要な case から本線化する。
- fallback rate と base compile time を `CompilerSession` stats / Node JSON で実測する。
- `.neplobj` / `.nepllink` がない状態では materialized callable を `memo_call` / function value identity へ入れない。

## 2026-06-01 Web materialized body-skip checkpoint

Web / loader は `.neplmeta` projection 成功結果を `MaterializedPublicSurfaceInput` へ変換し、
`PublicInterfaceArtifactInputs` から typecheck へ渡せるようになった。loader は projection が成功した
import / prelude edge について dependency root item merge を省き、target file slot と materialized
surface を current compile の authority として返す。

この checkpoint で完了した範囲:

- loader-level edge materializer callback を追加し、artifact store を持つ Web / CLI session と
  source graph / `SourceMap` authority を持つ loader を分離した。
- materialized edge でも target file id を current `SourceMap` に残し、typecheck の
  `module_path` / `file_id` 一致検査を維持した。
- Web `CompilerSession` は warm `.neplmeta` store がある場合だけ materialized load path を使う。
- materialized compile attempt が失敗した場合は full source load / compile に戻し、
  metadata-only callable body 欠落や materializer reject をユーザー向け診断として露出させない。

この issue はまだ open のまま維持する。理由は、今回の body skip は speculative optimization として
source fallback を持つ段階であり、base compile time を恒常的に下げるには次の残件が残るためである。

- bundled stdlib `.neplmeta` を初回 compile 前から preseed し、base compile でも projection を使えるようにする。
- dependency source を読み込まず interface artifact だけから `SourceMap` file slot と semantic surface を復元する。
- selected materialized callable body を `.neplobj` / `.nepllink` で解決し、direct call case の source fallback を減らす。
- fallback rate、base `compile_ms`、warm edit `compile_ms` の report を使って、`.neplobj` / `.nepllink` の対象を絞る。
- `memo_call` / private cache proof / function value identity は、stable codegen artifact と Resource proof が揃うまで `.neplmeta` callable だけでは許可しない。

## 2026-06-01 materialized compile fallback stats checkpoint

`CompilerSession` stats に、projection 成功後の materialized compile attempt と source fallback を
分けて観測する counter を追加した。これにより、`.neplmeta` store の projection stats だけでは見えなかった
「metadata は使えたが `.neplobj` / `.nepllink` がないため source fallback へ戻った」case を
文字列 error 解析なしで追える。

追加した境界:

- materialized surface を compile pipeline へ渡した compile だけを attempt として数える。
- attempted surface 数を累積し、last compile の surface 数も別に出す。
- source fallback の成功 / 失敗を分ける。
- `backend.codegen.materialized_function_body_missing` は enum reason code として記録する。
- compiled-output cache hit や stdlib overlay compile では last outcome を `NotAttempted` に戻し、
  前回の fallback 状態を最新 compile の結果として扱わない。

この issue は引き続き open のまま維持する。今回の checkpoint は fallback rate の実測基盤であり、
実際に fallback を減らすには次が必要である。

- bundled stdlib `.neplmeta` preseed により base compile でも projection を使えるようにする。
- `.neplobj` / `.nepllink` を導入し、selected materialized callable body を source fallback なしで解決する。
- `memo_call` / function value identity は `.neplmeta` projection stats ではなく、stable codegen artifact と Resource proof が揃った後に別 issue で開く。

## 2026-06-01 materialized compile performance report checkpoint

Node runner の `timing` に `compiler_session_stats` を追加し、materialized compile counter を before /
after snapshot ではなく compile 単位の delta として出すようにした。`available=false` の場合は
`missing_snapshot`、`missing_counter`、`invalid_counter`、`counter_decreased` などの reason を出し、
欠落と実際の 0 を混同しない。

`compare_git_versions.js` は `compile_ms` と同じ revision summary に materialized compile delta を集計し、
Markdown には `Materialized Compile` table を出す。これにより、base / warm edit `compile_ms` と
`source_fallbacks_delta_sum` / `body_missing_fallbacks_delta_sum` を同じ report で比較できる。

この checkpoint も fallback を減らす実装ではない。次は report で body missing fallback が増える
surface を `.neplobj` candidate として抽出し、bundled stdlib `.neplmeta` preseed と object/link artifact の
実装順序を決める。

## 2026-06-01 `.neplobj` candidate surface counter checkpoint

materialized compile fallback stats に `.neplobj` candidate surface counter を追加した。
`MaterializedFunctionBodyMissing` を理由に source fallback した compile では、attempted surface 数を
`nepl_obj_candidate_body_missing_surfaces` へ加算し、last compile の値を
`nepl_obj_candidate_last_body_missing_surfaces` として出す。

`run_test.js` と `compare_git_versions.js` はこの値を compile 単位の delta として扱い、
`body_missing_candidate_surfaces_delta_sum` を Markdown report に出す。これにより、fallback compile
件数だけでなく、`.neplobj` が解決すべき materialized surface 数を実測できる。

この checkpoint では symbol や function identity を保存しない。`.neplobj` stable link symbol、
selected callable body hash、generic instantiation hash の設計が固まるまでは、`TypeId` / `Span` /
`SourceMap` 由来の情報を永続化しない方針を維持する。

同じ checkpoint で、同一 `CompilerSession` の cold / warm / body edit sequence を固定する
`nodesrc/bench_materialized_compile_fallbacks.js` を追加した。通常 test runner の worker 分散に
依存せず、`compile_ms` と candidate surface delta と stage timing を同じ JSON に出すための実測入口である。

## 2026-06-01 materialized fallback diagnostic detail checkpoint

`OtherCoreError` に丸められていた materialized compile fallback の primary diagnostic code を
`nepl_meta_materialized_compile_last_fallback_diagnostic_code` として出すようにした。
`PublicSurfaceMaterializeRejectReason` も typed `TypeDiagnosticCode` へ写すため、
`type.public_surface.materializer_rejected` だけではなく、materializer authority の不足箇所を
JSON から直接読める。

`core/char` fixture の warm compile では、fallback は `MaterializedFunctionBodyMissing` ではなく
`type.public_surface.materializer.field_accessor_unsupported` だった。これは trait/impl materializer の
次に残った root gap が、field accessor callable surface の復元であることを示す。

この issue は trait / impl materializer の親 issue として open のまま維持する。field accessor callable
materializer は `ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B`
として分離し、`memo_call` / private cache proof / function value identity は引き続きこの materializer
で bypass しない。

## 2026-06-01 body-missing skip checkpoint

field accessor wrapper arity 修正後、`core/char` fixture は `backend.codegen.materialized_function_body_missing`
まで到達した。これは typecheck materializer の blocker ではなく、`.neplobj` / `.nepllink` が未実装で
selected callable body を供給できないための fail-closed fallback である。

この checkpoint では、同じ `CompilerSession` 内で一度 body-missing になった dependency edge を
`target_source_key_hash` と `dependency_public_surface_hash` で記録し、同じ boundary では再度
materialized compile へ投機投入しないようにした。`.neplobj` がない間は source merge へ戻し、`.neplobj`
実装後は同じ key 境界で object fragment availability を見る。

`tmp/neplmeta-body-missing-skip-20260601.json` の実測では、body-missing fallback は warm 3 回中 1 回に
減り、後続 2 回は `body_missing_skip_hits_delta_sum=10` で既知 body-missing edge を避けた。

この issue の残件は trait / impl materializer と direct-call `.neplobj` の境界を分けたまま維持する。
function value / indirect call / `memo_call` は、stable codegen artifact と Resource proof が揃うまで
`.neplmeta` callable だけでは許可しない。

## 2026-06-01 direct-call `.neplobj` key schema checkpoint

direct-call `.neplobj` 側の最初の実装として、`NeplObjDirectCallKey` を追加した。
これは trait / impl materializer の責務ではなく、`.neplmeta` projection 後に direct call body が
必要になった場合の backend artifact key である。

key は stable link symbol だけに依存しない。selected callable body hash と generic instantiation hash を
含めるため、公開 signature が同じ body-only edit や generic 具体化違いを誤って同じ codegen fragment に
しない。source capability policy hash と private effect policy hash も含め、SourceCapability や
private effect mask policy の変更では古い object を fail-closed に捨てる。

この checkpoint では `.neplmeta` callable を function value、indirect call、`memo_call` へ入れる経路を
広げていない。次は direct call resolver / availability store だけを接続し、高階関数と memoization は
別 issue の private cache proof と backend representation が揃うまで継続して拒否する。

## 2026-06-01 materialized body-missing kind checkpoint

`.neplmeta` materialized callable が codegen 入力へ到達した場合の body-missing 診断を、
direct call、function value、memoized function value の use kind 付きで作るようにした。

trait / impl materializer の issue では、この分類を使って direct call だけを `.neplobj` resolver へ
進ませる。function value、indirect call、`memo_call` は、引き続きこの materializer issue で
bypass しない。

## 2026-06-01 direct-call `.neplobj` availability input checkpoint

`PublicInterfaceArtifactInputs` に direct call 用 `.neplobj` availability 入力を足し、body-missing dependency を
diagnostic に落とす前に structured resolver を通すようにした。

trait / impl materializer 側では、これはまだ `CompilerSession` の object store 接続ではない。
availability を渡せるのは対応する direct-call body fragment payload が backend に接続済みである場合
だけであり、function value、indirect call、`memo_call` は direct-call fragment では解決しない。

この key-only availability 入口は、backend が payload を消費しないまま diagnostic だけを消す危険が
あるため、2026-06-02 checkpoint で破棄した。trait / impl materializer は引き続き public surface
のみを提供し、codegen body の authority は `.neplobj` / `.nepllink` 側で別に証明する。

## 2026-06-02 direct-call `.neplobj` fragment payload checkpoint

key-only availability を使う代わりに、`NeplObjDirectCallFragmentArtifact` を追加した。`.neplmeta`
trait / impl materializer は公開 surface を materialize できても、backend body を持たないため、それ
だけでは direct call の body-missing を消してはならない。

`NeplObjDirectCallFragmentArtifact` は `NeplObjDirectCallKey` と backend payload を同じ artifact に
まとめる。Wasm payload では signature、function body bytes、direct-call relocation を保持し、final
module assembly で relocation を解決する。これにより、key-only hit で unknown function symbol を
隠す経路を閉じるための payload 境界を固定した。現 checkpoint ではまだ diagnostic を消さず、backend
登録済み token ができるまで source fallback を維持する。

この checkpoint でも function value、indirect call、`memo_call` は解決しない。高階関数と
memoization には、別途 function value backend representation と PrivateCache proof が必要である。

## 2026-06-02 wasm direct-call link-plan token checkpoint

`.neplmeta` materialized callable の direct call を安全に通す前段として、wasm backend 側で
`.neplobj` fragment payload の link-plan token を作れるようにした。これは trait / impl materializer の
surface projection ではなく、backend が function index 空間と relocation target を確認する境界である。

この token は direct call fragment 専用であり、function value、indirect call、`memo_call` は扱わない。
そのため、この issue の trait / impl materializer は引き続き public surface の復元に集中し、codegen body
availability は binary intermediate artifact issue の `.neplobj` / `.nepllink` 側で扱う。

## 2026-06-01 selected body hash authority checkpoint

direct-call `.neplobj` key の selected body hash は、Resource summary value cache と同じ
`resource_function_body_stable_hash` を使う。trait / impl materializer で callable body を別の
文字列 hash や typed HIR hash に分岐させず、Resource IR まで下げた後の安定 body hash を object
store の authority にする。

この issue では、body hash があっても function value、indirect call、`memo_call` を解決しない。
高階関数と memoization は別途 backend representation と PrivateCache proof が必要である。
