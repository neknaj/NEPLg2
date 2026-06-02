---
id: ISS-20260602T134118244Z-NATIVE-CHECK-SHOULD-USE-PRE-TYPECHEC-31F9C9CD
title: "Native check should use pre-typecheck interface artifacts before loading stdlib bodies"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-06-02
updated: 2026-06-02
target: "nepl-core/src/loader.rs; nepl-core/src/compiler.rs; nepl-cli/src/main.rs; nepl-core/src/artifact.rs"
---

# ISS-20260602T134118244Z-NATIVE-CHECK-SHOULD-USE-PRE-TYPECHEC-31F9C9CD: Native check should use pre-typecheck interface artifacts before loading stdlib bodies

## 概要

RPN cold base は `.neplproof` preseed 後も `loader_load` と `resource_typecheck` に支配されている。
native CLI `--check` は、依存先の public interface だけで足りる場面でも stdlib source body を読み込み、
親 module へ merge してから typecheck へ進む。

## 対象

- `nepl-core/src/loader.rs; nepl-core/src/compiler.rs; nepl-cli/src/main.rs; nepl-core/src/artifact.rs`

## 根拠

- `examples/rpn.nepl` を代表 workload として、release `target\release\nepl-cli.exe --check -i examples\rpn.nepl --target std --stdlib-root stdlib` を測定した。
- `.neplproof` preseed により `resource_static_check` はおおむね `0.36-0.40s` まで下がったが、全体 wall-clock はまだ約 `0.89s` である。
- 2026-06-02 の direct import / root source single-read checkpoint では、no-stage 10 run が `953.505ms / 896.248ms / 894.861ms / 876.222ms / 865.402ms / 924.871ms / 862.790ms / 872.495ms / 928.304ms / 867.538ms`、中央値 `885.542ms` だった。
- 同 checkpoint の stage timing 中央値は `loader_load=357.711ms`、`check_pipeline=569.216ms`、`resource_typecheck=141ms`、`resource_static_check=396ms` だった。
- `NEPL_LOADER_STAGE_TIMING=1` の詳細計測では、loader の支配点は `read_file` ではなく recursive `process_directives` と dependency body merge である。
- Web / provider-backed loader には `.neplmeta` store と `MaterializedPublicSurfaceInput` により dependency body merge を skip する経路がある。一方、native CLI `--check` の通常 path はまだ `PublicInterfaceArtifactInputs::new(None, None, &[])` 相当であり、typed interface artifact を使っていない。
- 現行 prepare path は selected materialized callable の body が無い場合に codegen fallback diagnostic を出す。`--check` は codegen body availability ではなく型検査と Resource proof が authority なので、check 専用 boundary と codegen boundary を分ける必要がある。

## 問題

`.neplproof` は Resource summary / proof の再利用境界であり、dependency module の source body load、
directive merge、依存先 public callable / type / trait / impl surface の再 materialize までは削れない。
そのため Resource static check の一部が artifact 化されても、stdlib-heavy program では loader と
dependency typecheck の固定費が残る。

## 影響

RPN proof-backed cold base は約 `0.9s` に残り、base compile `0.5s` 未満目標へ届かない。さらに
stdlib / selfhost compiler が大きくなるほど、source body merge と依存先再 typecheck の固定費が
増え、`.neplproof` や expression subtree cache だけでは初回 compile を軽量化できない。

## 修正方針

native CLI `--check` を `.neplmeta` または同等の typed public interface artifact に接続する。

実装方針:

- import / prelude edge について、dependency body load 前に pre-typecheck envelope を作り、保存済み public interface artifact を fail-closed に probe する。
- artifact が source key、module surface、dependency public surface、target/profile、source capability policy、private effect policy に一致した場合だけ、`MaterializedPublicSurfaceInput` として typecheck に渡す。
- materialized surface が成功した edge は dependency body を root AST へ merge しない。
- `--check` では selected materialized callable body が無くても、codegen 用 body-missing diagnostic にしない。Resource proof に必要な summary がない場合だけ source fallback または proof fallback へ戻る。
- codegen / run では `.neplobj` body fragment が揃うまで source fallback を維持する。check-only public interface boundary と backend body availability を混同しない。
- artifact hit できない場合、または再投影に失敗した場合は通常 source load / typecheck / Resource check へ戻る。

## 検証

- `examples/rpn.nepl` を release `nepl-cli --check` で測定する。
- exact `.neplcheck` hit を避けるため `NEPL_DISABLE_CHECK_CACHE=1` を使う。
- `.neplproof` preseed の影響を固定するため、専用 `NEPL_PROOF_CACHE_DIR` を bootstrap してから測定する。
- `NEPL_CLI_STAGE_TIMING=1` と `NEPL_COMPILE_STAGE_TIMING=1` により `loader_load`、`resource_typecheck`、`resource_static_check` の中央値を記録する。
- `loader_load` と `resource_typecheck` が下がり、`resource_static_check` の diagnostic / proof coverage が弱まらないことを確認する。
- artifact mismatch、source edit、capability policy edit、target/profile edit で fail-closed に source fallback へ戻る regression を追加する。
