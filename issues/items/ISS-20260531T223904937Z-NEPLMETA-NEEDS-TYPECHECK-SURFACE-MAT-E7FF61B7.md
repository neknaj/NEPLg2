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
