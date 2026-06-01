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
target: "nepl-core/src/typecheck/materializer.rs; nepl-core/src/typecheck/public_surface.rs; nepl-core/src/artifact.rs; nepl-web/src/lib.rs"
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
