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
