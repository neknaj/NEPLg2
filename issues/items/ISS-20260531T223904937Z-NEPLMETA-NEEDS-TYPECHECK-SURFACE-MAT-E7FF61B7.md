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
