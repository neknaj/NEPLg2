---
id: ISS-20260601T105003551Z-NEPLMETA-NOMINAL-TYPE-MATERIALIZER-NEEDED-5C9B2A10
title: ".neplmeta nominal type materializer needed for impl target surface"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-01
target: "nepl-core/src/typecheck/public_surface.rs; nepl-core/src/typecheck/materializer.rs; nepl-core/src/types.rs"
---

# ISS-20260601T105003551Z-NEPLMETA-NOMINAL-TYPE-MATERIALIZER-NEEDED-5C9B2A10: .neplmeta nominal type materializer needed for impl target surface

## 概要

`.neplmeta` semantic trait surface により、`std/prelude_base` edge probe の最初の blocker は `MissingTraitIdentity` から `MissingNamedTypeIdentity` へ進んだ。残っている blocker は `Impl` surface 内の target type / trait argument に現れる nominal type application を current session の `TypeCtx` へ安全に復元できないことである。

## 根拠

- `last_pre_typecheck_edge_probe_projection_blocker_reason_code=3` は `MissingNamedTypeIdentity` を示す。
- `last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code=5` は blocker が `Impl` surface 由来であることを示す。
- `core/traits/copy` は `core/mem/types` を import し、`MemPtr .T` の `Clone` / `Copy` impl を定義する。
- trait identity は semantic surface として保持できるようになったため、次は nominal type identity と `Apply` の materialize が必要である。

## 問題

現行 materializer は primitive / tuple / function / box / reference / generic parameter だけを `TypeCtx` へ戻せる。`PublicTypeTerm::Named` と `PublicTypeTerm::Apply` は fail-closed に拒否されるため、`MemPtr .T` のような impl target を持つ capability impl を `.neplmeta` から復元できない。

名前だけで `MemPtr` や `RegionToken` を再構築すると、別 module の同名型や異なる definition hash の型を誤対応させる。したがって、source path、kind、arity、definition hash を持つ `PublicNominalTypeIdentity` を authority として使う必要がある。

## 修正方針

- `PublicTypeTerm::Named { identity: Some(...) }` を `TypeCtx` の stable nominal registry へ復元する materializer を追加する。
- `PublicTypeTerm::Apply` は base nominal identity と type argument list を materialize して復元する。
- identity が欠落した named type や unsupported application は引き続き fail-closed にする。
- semantic-only nominal type surface が必要な場合は、public export と混同しない形で `.neplmeta` に保持する。
- trait/impl materializer では、nominal type 復元後に `ImplInfo` を validated semantic registry へ注入する。

## 検証

- `std/prelude_base` edge probe が `MissingNamedTypeIdentity` / `Impl` blocker を解消し、次の未対応 blocker または projection success まで進む。
- 同名 struct / enum が別 source path にある場合、definition hash mismatch で拒否される。
- `MemPtr .T` の `Clone` / `Copy` impl target が stable nominal identity と generic argument で復元される。
- semantic-only nominal type が public export に混ざらない。
- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `node tests\compiler\tree\run.js`

## 2026-06-01 backend scalar / int128 locality checkpoint

`PublicTypeTerm::Named` のうち `i64` / `u64` / `f64` / `u32` は
`BackendScalarType` domain から復元できるようにした。これらは `TypeKind::Named` として流れるが、
user-defined nominal type ではなく compiler-owned backend scalar なので、stable nominal identity
欠落 blocker にはしない。

`i128` / `u128` は backend scalar ではなく `core/math/int128/types` の public struct である。
そのため `copy/primitive` から `i128` / `u128` の `Clone` / `Copy` impl を外し、型定義 module へ
移した。これにより、prelude の primitive capability artifact が型定義を持たない `i128` / `u128`
impl surface を抱え込まない。

この checkpoint 後の `std/prelude_base` edge probe は、少なくとも一部の stored stdlib artifact を
projection success まで進める。残る reject は `MissingNamedTypeIdentity` ではなく
`UnsupportedExportKind` (`reject_code=11`) であり、non-callable export と semantic trait / impl
registry materializer が次の実装単位になった。この issue は、stable nominal identity を持つ
`Named` / `Apply` を materializer 本体で復元する残件として引き続き open とする。

## 2026-06-01 non-callable export projection checkpoint

`.neplmeta` projection は `Callable` だけでなく `Struct` / `Enum` / `Trait` local export も
kind を保って `TypedPublicSurfaceTable` へ戻せるようになった。これにより、root probe と一部の
stdlib edge probe は `UnsupportedExportKind` projection reject を越えて success まで進む。

ただし、この issue はまだ解決していない。projection は artifact の stable public surface を
復元するだけであり、`PublicTypeTerm::Named { identity: Some(...) }` と `Apply` を
`TypeCtx` へ materialize する本体は未実装である。`MemPtr .T` のような impl target を
名前だけで対応させないため、この issue は stable nominal identity materializer の残件として
open のまま維持する。

## 2026-06-01 nominal definition materializer checkpoint

`typecheck/materializer` は、semantic table を受け取る入口では `Struct` / `Enum` surface を
`PublicNominalTypeIdentity` から `TypeCtx` へ復元できるようになった。`PublicTypeTerm::Named` は
同じ stable identity の nominal definition が predeclare 済みのときだけ `TypeId` へ戻し、
`PublicTypeTerm::Apply` も base / args を materialize して `TypeCtx::apply` へ戻す。

これにより、callable が後続の struct entry を引数に取る場合でも、predeclare pass により
entry order に依存せず materialize できる。途中で callable link symbol 欠落などの reject が起きた場合は
`TypeCtx` checkpoint を rollback し、named table や constructor binding を汚さない。

artifact が提示する `PublicNominalTypeIdentity.definition_hash` は、materializer 側でも
`Struct` / `Enum` surface から再計算して照合するようにした。登録直前には materialized
`TypeKind` からも hash を再計算し、public surface hash mirror と `TypeCtx` 側 fingerprint が
ずれた場合も fail-closed に拒否する。

この issue はまだ open のまま維持する。理由は、`PublicImplSurface` の target / trait application を
validated impl registry へ注入する経路が未実装であり、`MemPtr .T` のような impl target が
実際の capability registration に届くところまでは完了していないためである。
