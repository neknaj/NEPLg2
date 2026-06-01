---
id: ISS-20260531T223904937Z-NEPLMETA-NEEDS-TYPECHECK-SURFACE-MAT-E7FF61B7
title: ".neplmeta needs typecheck surface materializer"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/typecheck; nepl-core/src/loader.rs; nepl-core/src/artifact.rs; nepl-web/src/lib.rs"
---

# ISS-20260531T223904937Z-NEPLMETA-NEEDS-TYPECHECK-SURFACE-MAT-E7FF61B7: .neplmeta needs typecheck surface materializer

## 概要

Even with a structured .neplmeta payload, the compiler needs a safe way to project artifact-owned public surfaces into the current compile session's fresh TypeCtx, Env, trait table, impl table, and diagnostic origin.

## 対象

- `nepl-core/src/typecheck; nepl-core/src/loader.rs`

## 根拠

- 未記入

## 問題

Even with a structured .neplmeta payload, the compiler needs a safe way to project artifact-owned public surfaces into the current compile session's fresh TypeCtx, Env, trait table, impl table, and diagnostic origin.

## 影響

Using .neplmeta directly or by parsing stable text would either be unsafe or too slow, and dependency body-skip would remain blocked.

## 修正方針

Introduce a fail-closed materializer that converts structured public surfaces into current-session TypeId/Env entries and import visibility, with diagnostics anchored to import directives rather than artifact spans.

## 検証

Check-only tests should use .neplmeta-derived surfaces for stdlib/prelude/import modules and reject stale or incomplete artifacts; alias, reexport, noshadow, trait impl lookup, and public signature edit invalidation must be covered.

## 2026-06-01 materializer MVP gate checkpoint

`NeplMetaArtifact::materializer_mvp_reject` を追加した。

これは materializer 本体ではなく、`.neplmeta` artifact 全体が stdlib public surface materializer MVP へ
渡せるかを判定する fail-closed gate である。payload consistency、module surface、export surface、
module identity、`TypedPublicSurfaceTable::materializer_blockers()`、module / re-export edge の import
projection を一か所で確認する。

受け入れる範囲:

- local export。
- `ImportClause::Open`。
- alias なし、glob なしの simple named selective import。

拒否する範囲:

- module surface / export surface / module identity の欠落。
- public surface preflight blocker。
- `Include`。現行 loader では AST inline 境界なので import と同一視しない。
- `Merge`、default alias、alias、glob。target artifact と collision / ambiguity 判定なしに展開しない。
- `Impl` lookup。`Impl` は名前 export ではなく structured public surface と module visibility map で扱う。

Web `CompilerSession` stats JSON には `nepl_meta_artifact_materializer_mvp_ready` と
`nepl_meta_artifact_materializer_mvp_reject_code` を追加した。現段階では ready は body skip 実行ではなく、
次 checkpoint の `typecheck/materializer` へ進める前提条件だけを表す。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo test -p nepl-core materializer_preflight --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`

## 2026-06-01 typecheck materializer callable MVP checkpoint

`nepl-core/src/typecheck/materializer.rs` を追加し、`.neplmeta` の `TypedPublicSurfaceTable` から
local public callable を現在 compile session の fresh `TypeCtx` / `Env` へ投影する内部 API を
実装した。

今回の受け入れ範囲:

- primitive / tuple / function / generic parameter / box / reference type。
- `PublicCallableLinkSymbol` を持つ local public callable。
- `PublicTypeParamRef { binder_depth, index }` による generic binder 対応。
- 同じ stable link symbol の再 materialize は idempotent に skip。

今回の fail-closed 範囲:

- callable 以外の surface。
- entry kind mismatch、function type ではない callable type、arity mismatch、effect mismatch、signature hash mismatch。
- link symbol 欠落または entry name との不一致。
- field accessor、trait bound、named type、`TraitSelf`、unbound generic、type application。
- existing value conflict と `no_shadow` 同signature conflict。

subagent review 後、materializer は two-phase staging に変更した。後続 entry で reject した場合、
先に検査済みの callable も `Env` へ挿入しない。通常 source typecheck fallback へ戻るための
fail-closed 境界として、`Env` を半端に汚さないことを regression で固定した。

この checkpoint は body skip 完了ではない。`def_id` は `None` のため `@func` / `memo_call @func`
のような function value identity 依存経路は、stable function identity materializer が入るまで
通常 source load / typecheck fallback に残す。次は import / prelude boundary で target
artifact を引き、local export / Open / simple named import projection をこの materializer へ
接続する。

追加検証:

- `cargo test -p nepl-core materializer --lib -- --nocapture`

## 2026-06-01 import projection checkpoint

target `.neplmeta` artifact を読めた後、import clause で見える public surface だけを
typecheck materializer へ渡す projection API を `artifact.rs` に追加した。

今回の受け入れ範囲:

- local callable export。
- `Open` / clause なしの local callable export 全体。
- alias なし selective import の指定 callable export。

今回の fail-closed 範囲:

- payload consistency mismatch。
- module/export surface 欠落、module identity 欠落。
- public surface blocker。
- re-export projection。さらに target artifact が必要なのでこの checkpoint では展開しない。
- alias、glob、merge、default alias。
- struct / enum / trait export。
- selective import の missing name。
- export entry と structured public surface の不一致。

この checkpoint も body skip 完了ではない。現在の `CompilerSession` は module path keyed な
`.neplmeta` artifact store を持たないため、loader の import / prelude boundary にはまだ接続しない。
次は target artifact store、header compatibility、dependency public surface hash の確認を入れてから、
projection と `typecheck/materializer` を接続する。

追加検証:

- `cargo test -p nepl-core neplmeta_projection --lib -- --nocapture`

## 2026-06-01 artifact store checkpoint

`NeplMetaArtifactStore` を追加し、target `.neplmeta` artifact を module path keyed に保持してから
header compatibility と import projection を再確認する in-memory store 境界を実装した。

今回の受け入れ範囲:

- `NeplMetaModuleSurface::canonical_module_path` を持つ artifact。
- payload consistency が header と一致する artifact。
- 保存済み artifact に対する compatible header。
- `materializer_import_public_surface_mvp` が受け入れる Open / simple named import projection。

今回の fail-closed 範囲:

- missing artifact。
- module surface 欠落。
- canonical module path 欠落。
- payload consistency mismatch。
- compiler / target / profile / dependency public surface / schema などの compatibility mismatch。
- import projection unsupported。

store は `NeplMetaArtifactStoreStats` を持ち、store、store reject、hit、miss、payload reject、
compatibility reject、projection reject を文字列解析なしに区別する。性能改善の計測で
「cache が存在しない」のか「存在するが安全条件で拒否された」のかを確認するための authority である。

この checkpoint は body skip 完了ではない。現行 loader/typecheck は依存 module AST の merge と
SourceMap/FileId 由来の visibility に依存しているため、次は `CompilerSession` / loader boundary へ
store を接続する前に、`def_id=None` の callable が `@func` / `memo_call @func` のような
function identity 必須経路へ流れない guard を追加する。

追加検証:

- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta_projection --lib -- --nocapture`

## 2026-06-01 function identity guard checkpoint

`.neplmeta` materializer が復元する callable を通常の直接呼び出し候補として使えるようにする一方で、
`def_id=None` の callable が関数値 identity 必須経路へ流れない guard を追加した。

追加した境界:

- `typecheck/env.rs` に `resolved_function_value_identity` と `FunctionValueIdentityReject` を追加。
- callable / no capture / `def_id=Some` の場合だけ `FunctionValueIdentity` を構築。
- 明示 `@name`、期待関数型による callable value 選択、qualified callable 選択、overload fallback、
  trait method forced value で `def_id=None` を合成しない。
- higher-order argument coercion と indirect call boundary でも unresolved identity を型診断にする。
- `FunctionValueUnresolvedIdentity` diagnostic code を追加。

この checkpoint は stable function identity materializer ではない。`.neplmeta` 由来 callable を
直接呼び出し候補として使う準備であり、`@func`、`memo_call @func`、indirect function value へ進む
経路は、source body / DefId 相当の stable identity と Resource IR proof boundary が入るまで
通常 source fallback または型診断で止める。

追加検証:

- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `cargo test -p nepl-core --test functions -- --nocapture`
- `cargo check -p nepl-core -p nepl-language`
