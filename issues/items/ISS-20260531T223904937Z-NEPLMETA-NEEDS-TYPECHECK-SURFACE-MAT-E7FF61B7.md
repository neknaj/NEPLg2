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

## 2026-06-01 import/prelude edge pre-typecheck probe checkpoint

`Loader::process_directives_with` の prelude / import load 成功地点から、実際に解決・load された
stdlib dependency edge だけを `.neplmeta` pre-typecheck probe へ渡す観測境界を追加した。

今回の接続は body skip ではない。`NeplMetaDependencyEdgePreTypecheckProbe` は target module の
module surface、target source key、target file 単体の source capability policy hash、
dependency public surface hash、import clause だけを持ち、target body の
typed HIR / Resource IR / `TypeId` / `Span` は保持しない。Web `CompilerSession` はこの probe から
pre-typecheck envelope を作り、`NeplMetaArtifactStore::materializer_import_public_surface_pre_typecheck_edge_probe_mvp`
を呼んで root probe とは別の edge probe stats を更新する。

安全境界:

- `.neplmeta` store が空の compile では edge probe 材料を収集しない。再利用候補がない状態で
  dependency edge ごとの source 再読込や dependency hash 計算を増やさないためである。
- `#include` は source merge 境界として扱い、dependency artifact edge として計測しない。
- non-stdlib VFS edge は store lookup へ流さない。
- projection 結果を `TypeCtx` / `Env` へ注入せず、通常 load / typecheck / Resource IR / codegen の
  fallback を変えない。
- edge envelope は root compile の `SourceMap` hash を使わない。root file や他 dependency の
  capability policy が混ざると、同じ target artifact が呼び出し元 root に依存して reject されるため、
  loader が target source identity を probe へ運ぶ。

regression では、dependency body-only edit 後に stdlib edge probe が missing artifact として
数えられること、`#no_prelude` + `#include` のみの compile では store が埋まった二回目でも edge
probe attempt が 0 のままであること、target source capability policy hash が root `SourceMap`
hash ではないことを固定した。

追加検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`
- `cargo test -p nepl-core source_import_surface_preserves_clause_visibility_and_order --lib -- --nocapture`
- `cargo test -p nepl-core root_dependency_aggregate_public_surface_hash --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta_edge_probe_uses_target_source_policy_boundary --lib -- --nocapture`
- `trunk build --release`
- `node tests\compiler\tree\run.js`
- `node nodesrc\test_run_test_compiler_session.js`

## 2026-06-01 session root pre-typecheck probe checkpoint

Web `CompilerSession` の実 compile path から、保存済み root `.neplmeta` artifact に対して
`materializer_import_public_surface_pre_typecheck_mvp` を観測専用に呼ぶようにした。

この checkpoint は body skip ではない。戻り値の `TypedPublicSurfaceTable` は使わず、通常の
load / typecheck / Resource IR / codegen は従来通り実行する。目的は、既存 store に artifact が
存在する二回目以降の compile で、pre-typecheck envelope が通るのか、`SourceKey` や
`DependencyPublicSurface` で拒否されるのかを Web stats から確認できるようにすることである。

安全境界:

- store が空の初回 compile では probe しない。
- stdlib overlay compile では store を compile path へ渡さない。
- `TypeCtx` / `Env` への materialize は行わない。
- `Include`、AST merge、import/prelude edge の load 順は変えない。

regression では、dependency body-only edit で previous root artifact が存在しても現 payload の
materializer blocker により projection reject になること、root literal edit では projection 前に
source key mismatch の compatibility reject になることを固定した。

subagent review では、依存 module 単位の本命接続は `Loader::process_directives_with` の
prelude/import `load_file_with` 成功直後に観測 hook として置くべきと確認した。次 checkpoint では、
root artifact だけでなく import/prelude edge ごとの module surface と import clause を渡す設計へ進む。

## 2026-06-01 pre-typecheck probe observation checkpoint

`NeplMetaArtifactStoreStats` に pre-typecheck probe 専用の観測 field を追加した。既存の
`hits` は artifact が store に見つかったことだけを表し、projection 成功や body skip 可能性を
意味しないため、probe attempts / projected / missing artifact / payload reject /
compatibility reject / projection reject / projected entries を分離して記録する。

`last_pre_typecheck_probe_reject_kind` は fallback reason の大分類、`last_pre_typecheck_probe_reject_code`
は対応する enum reason の安定 code である。`SourceKey` mismatch、dependency surface mismatch、
unsupported alias/glob などを Web stats から文字列解析なしに切り分けるための境界であり、
通常 source fallback の挙動は変えない。

Web `loader_cache_stats_json` にも同じ field を追加した。現 checkpoint では通常 compile path が
pre-typecheck probe をまだ呼ばないため値は 0 のままだが、field の存在を tree test で固定した。

追加検証:

- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node tests\compiler\tree\run.js`
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

## 2026-06-01 session store checkpoint

Web `CompilerSession` に `NeplMetaArtifactStore` を追加し、通常 compile 成功時の `.neplmeta`
artifact を session 内 store へ保存するようにした。

今回の受け入れ範囲:

- 通常 compile 成功時に生成された `.neplmeta` artifact。
- payload consistency が store により確認できる artifact。
- module path keyed に上書きできる in-memory store。

今回の fail-closed 範囲:

- compiled-output cache hit は新しい compile artifact ではないため store count を増やさない。
- stdlib override compile と `/stdlib/...` を含む通常 VFS overlay は、通常 stdlib artifact と
  取り違えないよう store を clear し、override artifact を store へ入れない。
- store hit / miss / projection はまだ loader import boundary へ接続しない。

`loader_cache_stats_json` には `.neplmeta` store entries、stores、rejects、hits、misses、
payload rejects、compatibility rejects、projection rejects を追加した。現 checkpoint では
projection API 未接続なので hits/misses は 0 のままでよく、まず「session が artifact を安全に保持
できているか」を観測する。

追加検証:

- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node tests\compiler\tree\run.js`
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp\playground-editor-neplmeta-session-store-20260601.json`

## 2026-06-01 source key invalidation checkpoint

`.neplmeta` を import materializer の入力へ使う前提として、header に `source_key_hash` を追加した。
これは dependency body skip を始める前に、artifact が現在の module source と同じ token-level
source に由来することを確認するための boundary である。

この hash は typed public signature hash を置き換えない。public signature が変わらない式 body
変更では typed public signature は同じままになり得るため、`source_key_hash` が別途必要になる。
通常コメントや doc comment だけの変更は source key に入れず、literal や directive など compile
結果に影響する token は key に残す。

`source_key_hash=None` は reusable artifact ではなく、現在 compile で source identity を証明
できない状態として扱う。次に loader/import boundary へ store を接続するときは、payload decode、
projection、`typecheck/materializer` 実行より先に expected header とこの field を照合する。

subagent review を受け、expected header を full typed artifact からしか作れない問題も分けた。
新しい `NeplMetaArtifactPreTypecheckEnvelope` は typed public signature / structured public surface
を要求せず、loader と source map で得られる field だけを照合する。body skip の入口ではこの
pre-typecheck envelope を先に通し、その後で payload consistency、projection、
`typecheck/materializer` の順に進める。

`source_key_hash=None` の artifact は store と materializer MVP で `MissingSourceKey` として拒否する。
SourceMap や canonical module path が欠落した artifact は、通常 source load / typecheck fallback
に戻す。

追加検証:

- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `node tests\compiler\tree\run.js`

## 2026-06-01 pre-typecheck store projection checkpoint

`NeplMetaArtifactStore` に `materializer_import_public_surface_pre_typecheck_mvp` を追加した。
これは body skip ではなく、loader/import boundary が target module source と module edge surface
を得た段階で、保存済み `.neplmeta` artifact を materializer 入力へ投影できるかを確認する
probe API である。

full typed header を要求する既存 projection は、typed public signature / structured public surface
を expected value として持つため、target module body typecheck 前には使えない。新しい API は
`NeplMetaArtifactPreTypecheckEnvelope` を受け取り、source key、dependency public surface、
module surface、source capability policy、private effect policy など payload decode 前に分かる
field だけを先に照合する。

pre-typecheck envelope が通っても、payload consistency と MVP projection は従来通り確認する。
成功しても `TypeCtx` / `Env` への注入や dependency AST inline の省略はまだ行わない。
失敗時は `MissingArtifact`、`PayloadConsistency`、`Compatibility(SourceKey / DependencyPublicSurface / ModuleSurface)`、
`Projection` の enum reason で fallback できる。

追加検証:

- `cargo test -p nepl-core neplmeta_store --lib -- --nocapture`

## 2026-06-01 projection blocker detail checkpoint

`.neplmeta` pre-typecheck probe の projection reject が `PublicSurfaceBlocker` で止まった場合に、
blocker reason と entry kind を stats へ保持するようにした。

これまでは `rejectKind=Projection` / `rejectCode=6` までしか Web stats から分からず、次に直すべき
materializer authority が trait、impl、named type、callable link symbol のどれなのかを外部から
判断できなかった。`PublicSurfaceMaterializerBlockerReason::code()` と
`TypedPublicSignatureKind::code()` を追加し、root probe と edge probe の両方で次を出す。

- `last_pre_typecheck_probe_projection_blocker_reason_code`
- `last_pre_typecheck_probe_projection_blocker_entry_kind_code`
- `last_pre_typecheck_edge_probe_projection_blocker_reason_code`
- `last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code`

`std/prelude_base` edge artifact の 3 回目 compile では、最初の blocker が
`MissingTraitIdentity` (`reason_code=7`) かつ `Impl` surface (`entry_kind_code=5`) であることを
固定した。subagent review でも同じ結論で、`prelude_base -> core/traits/copy -> copy/primitive`
経由の `Clone` / `Copy` impl が private capability trait identity を要求している。

次の根本対応は、callable-only MVP を延命することではなく、trait table / impl table /
capability registration を `.neplmeta` から fail-closed に復元する
[ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1](./ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1.md)
である。`memo_call` や function value identity はこの materializer で bypass せず、`def_id=None`
の callable は引き続き direct call 専用に留める。

追加検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core neplmeta_store_pre_typecheck_probe_records_projection_reject_reason --lib -- --nocapture`
- `cargo test -p nepl-core materializer_mvp --lib -- --nocapture`
- `trunk build --release`
- `node tests\compiler\tree\run.js`
