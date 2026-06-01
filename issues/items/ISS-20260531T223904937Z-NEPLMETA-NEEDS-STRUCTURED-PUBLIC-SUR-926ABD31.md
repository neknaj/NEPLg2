---
id: ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31
title: ".neplmeta needs structured public surface payload"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/artifact.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/typecheck/public_surface.rs; nepl-core/src/loader.rs"
---

# ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31: .neplmeta needs structured public surface payload

## 概要

.neplmeta currently stores TypedPublicSignatureTable stable text/hash only, which is not enough to rebuild TypeCtx/Env for imported modules without reparsing and retypechecking bodies.

## 対象

- `nepl-core/src/artifact.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/typecheck/public_surface.rs; nepl-core/src/loader.rs`

## 根拠

- 未記入

## 問題

.neplmeta currently stores TypedPublicSignatureTable stable text/hash only, which is not enough to rebuild TypeCtx/Env for imported modules without reparsing and retypechecking bodies.

## 影響

Base compile time cannot drop toward the stdlib prechecked artifact target while dependency modules still need full body parse/typecheck for public callable/type/trait/impl surfaces.

## 修正方針

Add arena-independent structured public surface entries to .neplmeta: module canonical path, public callable/type/trait/impl headers, trait bounds, noshadow and symbol policy. Do not store TypeId, Span, SourceMap, typed HIR, Resource IR, or diagnostics.

## 検証

Unit tests should materialize stable surface payloads for public functions, types, traits, impl headers, reexports, prelude edges, and body-only edits without changing payload hashes.

## 2026-06-01 structured payload checkpoint

`TypedPublicSurfaceTable` を追加し、`.neplmeta` payload が既存の `TypedPublicSignatureTable` と structured public surface の両方を持つようにした。

structured surface は public callable、struct、enum、trait、impl header を arena 非依存の enum/struct として保持する。`TypeId`、`Span`、`FileId`、`SourceMap`、`ImportResolution`、typed HIR、Resource IR、diagnostic span は保存しない。

`NeplMetaArtifactHeader` に `structured_public_surface_hash` と `structured_public_surface_entry_count` を追加し、payload consistency check でも structured surface hash / entry count の不一致を拒否する。artifact 形状を変更したため `.neplmeta` schema、artifact hash version、compiler identity は v2 に上げた。

Web `CompilerSession` stats JSON には structured public surface entry count と hash を追加した。payload 本体は出さない。

## 2026-06-01 public surface module split checkpoint

`TypedPublicSurfaceTable` の model / hash / builder / tests を `nepl-core/src/typecheck/public_surface.rs` へ分離した。

`public_signature.rs` は `TypedPublicSignatureTable` の stable text/hash 境界だけを担当する。structured surface は `.neplmeta` materializer の authority になるため、text signature の互換 hash と同じ module に置いたまま拡張しない。

`typecheck.rs` の public re-export は維持しており、`crate::typecheck::{PublicTypeTerm, TypedPublicSurfaceTable}` などの外部 API は変えない。`driver.rs` は signature builder と surface builder を別 module から呼ぶ。

この checkpoint はまだ `Named(String)` や generic parameter 参照を materializer authority として認めるものではない。次 checkpoint では stable nominal identity と binder-indexed generic reference を導入する。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_signature_hash --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `trunk build --release`
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/test_playground_compiler_session_policy.js`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-public-surface-split-20260601.json`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

## 2026-06-01 stable nominal surface checkpoint

`NominalStableTypeIdentity` の kind / source path / name / arity / definition hash を、structured public surface の `PublicNominalTypeIdentity` へ投影するようにした。

public struct / enum surface は自身の stable nominal identity を保持する。`PublicTypeTerm::Named` は name だけではなく、SourceMap から identity を得られる場合は `PublicNominalTypeIdentity` も保持する。SourceMap がない compile では identity は `None` のままであり、将来の `.neplmeta` materializer はこの entry を fail-closed に拒否できる。

structured public surface hash namespace は `neplg2-typed-public-surface-v2` に上げた。payload 形状が変わったため `.neplmeta` schema、artifact hash version、compiler identity は v3 に上げた。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_signature_hash --lib -- --nocapture`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/test_playground_compiler_session_policy.js`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-stable-nominal-surface-20260601.json`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

追加検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_signature_hash --lib -- --nocapture`

## 2026-06-01 binder-indexed generic surface checkpoint

`PublicTypeParamRef { binder_depth, index }` を追加し、`PublicTypeTerm::GenericParam` は `PublicTypeParam` 全体ではなく binder reference だけを保持するようにした。

`PublicTypeParam` は binder metadata に限定する。term 側へ name / capability / index を複製しないことで、`.neplmeta` materializer が同名 generic parameter を名前だけで誤対応させる経路を閉じる。

nested generic function type では、inner function が type parameter を導入する場合だけ外側 binder depth を 1 つ押し出す。type parameter を導入しない function type は新しい binder を作らない。これにより、`%fn .T .T` のような function type と、`%fn .T fn .T .T` のような nested generic function type を区別できる。

callable の trait bound は `PublicTypeParamBoundTarget` で表す。root function binder に対応できる場合は `Ref(PublicTypeParamRef)`、対応できない場合は `Unbound(PublicTypeParam)` とし、後者は将来 materializer が fail-closed に拒否する。

structured public surface hash namespace は `neplg2-typed-public-surface-v3` に上げた。payload 形状が変わったため `.neplmeta` schema、artifact hash version、compiler identity は v4 に上げた。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core public_type_term_shifts_outer_generic_refs_inside_nested_generic_function --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_signature_hash --lib -- --nocapture`
- `cargo test -p nepl-core generics --lib -- --nocapture`
- `cargo test -p nepl-core --test generics -- --nocapture`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/test_playground_compiler_session_policy.js`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-binder-generic-surface-20260601.json`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

残件:

- `PublicTypeTerm::Named` の identity が `None` の entry を materializer で fail-closed に拒否する。
- `PublicTypeTerm::UnboundGenericParam` と `PublicTypeParamBoundTarget::Unbound` を materializer で fail-closed に拒否する。
- generic impl parameter / bound を round-trip できるようにするか、materializer では fail-closed に拒否する。
- reexport / prelude edge と module canonical path を per-module `.neplmeta` surface に追加する。

## 2026-06-01 materializer preflight checkpoint

`TypedPublicSurfaceTable::materializer_blockers` と `TypedPublicSurfaceTable::is_materializer_preflight_ready` を追加した。

これは materializer 本体ではなく、structured public surface を current compile の `TypeCtx` / `Env` へ投影する前に、body skip してはいけない surface を fail-closed に検出する preflight である。preflight は primitive だけで構成された callable surface を通すが、次の surface を blocker として列挙する。

- `PublicTypeTerm::Named { identity: None }`
- `PublicTypeTerm::UnboundGenericParam`
- `PublicTypeParamBoundTarget::Unbound`
- `PublicTraitRef` の name-only reference
- identity を持たない public struct / enum surface

これにより、materializer 実装前に残っていた「名前だけの nominal type や対応 binder のない generic を推測で materialize する経路」をコード上で閉じた。trait reference はまだ stable trait identity を持たないため、現 checkpoint では name-only trait reference を明示的 blocker として扱う。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo test -p nepl-core materializer_preflight --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`

## 2026-06-01 stable trait surface checkpoint

`PublicTraitIdentity` を追加し、trait surface 自体と `PublicTraitRef` が SourceMap 由来の stable identity を保持できるようにした。

identity は source path、trait name、arity、definition hash で構成する。definition hash は trait type parameter、capability、method name、method type surface から作り、doc comment、method body、Span、TypeId、typed HIR、Resource IR は含めない。

callable bound と trait impl header の trait application は、この stable identity を持つ場合、`MissingTraitIdentity` blocker にならない。SourceMap がない compile では identity は `None` のままなので、materializer preflight は引き続き fail-closed に拒否する。

structured public surface hash namespace は `neplg2-typed-public-surface-v4` に上げた。payload 形状が変わったため `.neplmeta` schema、artifact hash version、compiler identity は v5 に上げた。

残件:

- `PublicTypeTerm::UnboundGenericParam` と `PublicTypeParamBoundTarget::Unbound` を materializer 本体の body-skip 判定へ接続する。
- callable surface に field accessor kind と stable public ABI/link symbol を追加する。ただし span-derived `mangle_function_symbol_for_def` は保存しない。
- generic impl parameter / bound を round-trip できるようにするか、materializer では fail-closed に拒否する。
- reexport / prelude edge と module canonical path を per-module `.neplmeta` surface に追加する。

検証:

- `cargo test -p nepl-core materializer_preflight --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`

## 2026-06-01 callable authority checkpoint

`PublicCallableSurface` に `PublicFieldAccessorKind` と `PublicCallableLinkSymbol` を追加した。

field accessor helper は普通の public function と同じ型だけでは materializer 後の Resource / SourceCapability 境界を復元できないため、`get_field` / `get_field_ref` / `set_field` 由来の helper kind を structured surface に保持する。

stable link symbol payload は source path、public callable name、structured type surface の signature hash で構成する。span-derived `mangle_function_symbol_for_def` は保存しない。body-only edit では link symbol は変わらず、signature edit や source path 変更では変わる。

materializer preflight は stable link symbol を持たない callable を `MissingCallableLinkSymbol` blocker として fail-closed に拒否する。SourceMap がない compile や、まだ stable ABI を付けられない artifact は body skip へ進まない。

structured public surface hash namespace は `neplg2-typed-public-surface-v5` に上げた。payload 形状が変わったため `.neplmeta` schema、artifact hash version、compiler identity は v6 に上げた。

残件:

- reexport / prelude edge と module canonical path を per-module `.neplmeta` surface に追加する。

検証:

- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core materializer_preflight --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`

## 2026-06-01 generic impl surface checkpoint

`ImplInfo` に impl header 自身の generic binder と bound environment を保持し、structured public surface の `PublicImplSurface` へ投影するようにした。

これまで impl surface は `target` と trait application だけを持ち、`public_impl_surface` 側では空の generic map で `target` / trait args を変換していた。そのため `impl<.T: Touch> Touch for Holder .T` の `.T` が binder-indexed ref にならず、materializer が名前から推測するか fail-closed に止まるしかなかった。

新しい surface は `type_params` と `type_param_bounds` を持つ。target type、trait application args、bound target はこの binder を使って `PublicTypeParamRef { binder_depth: 0, index }` へ変換される。bound の trait reference は既存の `PublicTraitIdentity` を使うため、private trait bound や identity 欠落も preflight で fail-closed に検出できる。

`PublicImplKind::Trait` からは trait definition 内部の `Self` type term を外した。これは public impl header の入力ではなく、artifact materializer が復元すべき authority ではないためである。public impl header として必要な情報は stable trait application と impl target type に集約する。

typed public signature hash namespace は `neplg2-typed-public-signature-v2`、structured public surface hash namespace は `neplg2-typed-public-surface-v6`、`.neplmeta` schema / artifact hash / compiler identity は v7 に上げた。

残件:

- reexport / prelude edge と module canonical path を per-module `.neplmeta` surface に追加する。
- fail-closed materializer を import / prelude boundary へ接続する。

検証:

- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
