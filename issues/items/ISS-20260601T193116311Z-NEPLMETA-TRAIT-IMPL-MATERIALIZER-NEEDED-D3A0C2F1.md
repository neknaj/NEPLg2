---
id: ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1
title: ".neplmeta trait and impl materializer needed for prelude capability surface"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-01
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
- fallback rate、base `compile_ms`、warm edit `compile_ms` を Node JSON / `CompilerSession` stats で継続的に実測する。
- `memo_call` / private cache proof / function value identity は、stable codegen artifact と Resource proof が揃うまで `.neplmeta` callable だけでは許可しない。
