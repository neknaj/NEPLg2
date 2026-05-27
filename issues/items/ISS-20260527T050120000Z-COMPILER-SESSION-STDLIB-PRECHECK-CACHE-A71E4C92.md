---
id: ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92
title: "CompilerSession needs prechecked stdlib artifact and incremental query cache"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-27
updated: 2026-05-27
target: "nepl-core, nepl-web, nodesrc/run_test.js, stdlib"
---

# ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92: CompilerSession needs prechecked stdlib artifact and incremental query cache

## 概要

2026-05-27 の performance fix により、release WASM の最小 program は cold `compile_ms=231`、warm `compile_ms=5` まで下がった。一方で aggregate / generic / stdlib-heavy case を微小変更時に常に 10ms 未満へ収めるには、単発 compile API の warmup だけでは不足する。

## 対象

- `nepl-core`
- `nepl-web`
- `nodesrc/run_test.js`
- `stdlib`

## 根拠

- `target/debug/nepl-cli.exe --check -i tmp/minimal_perf.nepl --target wasm --stdlib-root stdlib`: elapsed 160ms、`resource_typecheck=5ms`、`resource_static_check=1ms`。
- `target/debug/nepl-cli.exe --check -i tmp/perf_alloc_probe.nepl --target wasm --stdlib-root stdlib`: elapsed 166ms、`resource_typecheck=6ms`、`resource_static_check=1ms`。
- `trunk build --release` 後の `nodesrc/run_test.js` minimal cold は `compile_ms=231`, `total_ms=257`, `stdlib_vfs_mode=bundled`。
- 同じ release artifact の minimal warm は `compile_ms=5`。
- aggregate warm は `compile_ms=22` で、10ms 未満にはまだ届いていない。
- direct WASM API の同一 source 連続 compile は初回 127ms、以後 3-5ms であり、release artifact と warm process の効果は確認できた。
- first checkpoint で `nepl-web` に `CompilerSession` を公開し、Node runner は session API を優先するようにした。release WASM session smoke では minimal warm `compile_ms=3`、aggregate warm `compile_ms=16`、cold minimal `compile_ms=160` だった。
- 同 checkpoint で bundled stdlib content hash を artifact に埋め込み、Node runner は hash が一致する場合にだけ bundled stdlib を使う。hash API のない旧 artifact では mtime fallback を維持する。
- Web playground worker と tutorial runtime も method 単位で `CompilerSession` を優先するようにし、full stdlib VFS object を通常の compile path から外した。
- Web playground の workspace compile request は read-only stdlib files と runtime data files を overlay VFS へ含めず、editable `.nepl` user source だけを送るようにした。WASI 実行用の full VFS snapshot は `runtimeVfsData` として compile overlay から分離した。
- Web terminal は compile 用 Worker を artifact URL 単位で保持するようにし、連続 build では同じ WASM instance / `CompilerSession` を再利用する。`neplg2 run` は compile だけ persistent Worker を使い、生成 wasm の実行は一回限りの runtime Worker に分離した。
- second checkpoint で `nepl-core::loader::LoaderSessionCache` を追加し、`CompilerSession` から bundled stdlib の raw parsed module cache を使うようにした。
- parsed stdlib cache は `cache version + stdlib namespace hash + canonical path + source hash + imported type arity hint hash` を key にし、cached AST は中立 `FileId` へ正規化してから保持する。cache hit 時は現在 compile の fresh `SourceMap` が割り当てた `FileId` へ再投影する。
- `SourceMap`、merged module 全体、`ImportResolution`、typed HIR、`TypeCtx` / `TypeId`、Resource IR summary、codegen fragment はまだ cache しない。これは stale span / stale source capability / stale `TypeId` を避けるためである。
- stdlib override または overlay が `/stdlib` 以下を差し替える場合は parsed module cache を bypass し、bundled stdlib artifact を local override へ混ぜない。
- `CompilerSession.loader_cache_stats_json()` と `nodesrc/run_test.js` の `timing.compiler_session_cache_before` / `timing.compiler_session_cache_after` により、warm compile が cache hit したかを JSON output から確認できるようにした。
- `trunk build --release` 後の release WASM で、minimal warm は `compile_ms=2`、aggregate first after warmup は `compile_ms=19`、同一 process / 同一 `CompilerSession` で aggregate を再compileした場合は `compile_ms=3` だった。aggregate 2 回目では cache hits が 4 から 9 へ増え、stdlib parsed module cache が実際に効いていることを確認した。
- subagent review 後、同一 path の source hash 変更、imported type arity hint 変更、forced stdlib VFS / stdlib overlay bypass、空 namespace cache の禁止を regression と API 境界で補強した。
- third checkpoint で `LoaderSessionCache` に source arity surface cache を追加した。これは local type arity hints、prelude/import/include/public re-export path、root-only default prelude 判定だけを持つ未型付け artifact であり、`FileId` / `Span` / `ImportResolution` / typed HIR / `TypeId` は保存しない。
- source arity surface cache は `cache version + stdlib namespace hash + stdlib root + canonical path + source hash` を key にし、public re-export 先の arity result は親 surface へ畳み込まない。依存先 source hash が変わった場合は、facade source が cache hit しても依存先 surface を再評価する。
- `CompilerSession.loader_cache_stats_json()` は `arity_surface_hits` / `arity_surface_misses` / `arity_surface_stores` も返すようになった。
- `trunk build --release` 後の same preloaded `CompilerSession` 実測では、minimal は `compile_ms=2`、aggregate first は `compile_ms=15`、aggregate second は `compile_ms=4` だった。aggregate second では parsed module hits が 8 から 13、arity surface hits が 8 から 14 に増えた。
- fourth checkpoint で source-directed loader prewarm を追加した。全 bundled stdlib file list の総なめは subagent review で却下し、root source の default prelude / prelude / import / include から到達する configured stdlib closure だけを prewarm する。
- 同 checkpoint で arity surface cache を configured stdlib path に限定した。user entry source や user VFS module は `arity_surface_bypasses` として観測し、長寿命 `CompilerSession` の cache value には残さない。
- Node runner は `selectStdlibVfsMode(meta) == "bundled"` の場合だけ `CompilerSession.prewarm_loader_cache_for_source()` を呼ぶ。forced stdlib VFS / fs override では skip reason を timing に残し、bundled stdlib prewarm は実行しない。
- prewarm 中の loader error は `compiler_session_prewarm_error` として観測し、通常 compile path は続行する。これにより、prewarm 専用の失敗が本来の compile diagnostic を隠さない。
- `trunk build --release` 後の source-directed prewarm 実測では、minimal は `compile_ms=3` / `prewarm_ms=1` / `wasm_call_ms=2`、aggregate first は `compile_ms=15` / `prewarm_ms=3` / `wasm_call_ms=11-12`、aggregate second は `compile_ms=4-5` / `prewarm_ms=1` / `wasm_call_ms=3-4` だった。aggregate first の total `compile_ms` はまだ 10ms 未満に固定できていないため、logical import graph / typed public surface / Resource IR summary cache が次の根本対応である。

## 問題

現在の API は compile call ごとに loader / source map / parse / import / typecheck / Resource IR / codegen を新規に構築する。stdlib source は bundled になっても、stdlib の parse/import/typecheck artifact と Resource IR summary template は session 間で再利用されない。

このため、同一 process 内であっても entry source の微小変更に対し、変更されていない stdlib と unchanged user functions の query result を再利用する構造が不足している。

## 影響

Web playground、Node doctest runner、selfhost compiler 開発で、実行時間ではなく compile phase が feedback loop を支配する。静的検査を強化するほど同じ stdlib graph の再検査が増え、Zenn 方針の「純粋性と静的検査を活かした performance 追求」に反する。

## 修正方針

[NEPLg2.1 compiler performance / cache design 2026-05-27](../../doc/neplg2/compiler_performance_cache_design.md) に沿って、`CompilerSession` と stdlib prechecked artifact を導入する。

MVP は次の順に進める。

1. `nepl-web` に `CompilerSession` wasm-bindgen class を公開し、Node runner が session API を優先する状態にする。
2. `nepl-core` に source text / lex / parse / import graph / type arity を query として分離する session API を追加する。現在は source arity surface cache と parsed stdlib module cache まで実装済みで、typed public surface cache は未実装。
3. Web terminal の worker を compile ごとに破棄せず、同一 WASM instance / `CompilerSession` が複数 compile にまたがって warm state を保持するようにする。これは実装済みなので、次は `CompilerSession` 側へ semantic cache を載せる。
4. `CompilerSession` に bundled stdlib の parsed module / import graph / type arity を warm state として保持する。raw parsed module、stdlib-only source arity surface、source-directed loader prewarm は実装済み。次 checkpoint は typed public surface へ進む前に、logical import graph と dependency public surface hash の安定表現を設計する。
5. stdlib artifact に public signature table、trait impl index、source capability tableを持たせ、通常 compile では entry source と overlay source だけを新規処理する。
6. Resource IR summary を function hash + source capability hash + type argument hash で cache し、entry から到達する changed functions だけを再計算する。
7. codegen fragment cache を function hash 単位にし、unchanged fragments を signature/index table へ再接続する。

## 完了条件

- release WASM + warm `CompilerSession` で、最小 entry source の同一 compile と 1 行変更 compile が 10ms 未満になる。
- aggregate/generic の小規模 program でも、stdlib artifact が unchanged の場合は 10ms 台を安定して維持する。
- local stdlib が release artifact より新しい場合は cache を使わず、FS stdlib override / artifact refresh に戻る。
- local stdlib content hash が release artifact の bundled stdlib hash と一致しない場合は、mtime に関係なく FS stdlib override / artifact refresh に戻る。
- raw LLVM、raw wasm direct call、indirect call、曖昧な function reference は conservative-all で検査漏れしない。
- stale diagnostic span や stale source capability が別 source へ流用されないことを regression test で固定する。

## 検証

- `trunk build --release`
- `node nodesrc/run_test.js` minimal / aggregate timing
- session API の unit test
- loader parsed stdlib cache の `FileId` 再投影 unit test
- loader parsed stdlib cache の source hash / imported type arity hint invalidation test
- source arity surface cache の source hash / root default prelude / lexer error no-preload / public re-export dependency invalidation test
- source-directed prewarm が bundled mode だけで実行され、forced / fs override では skip されることの Node runner regression test
- prewarm error が本来の compile diagnostic を置き換えないことの Node runner regression test
- user source arity surface が long-lived `LoaderSessionCache` に保存されないことの unit test
- forced stdlib VFS path が session cache を使わないことの Node runner regression test
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/test_playground_compiler_session_policy.js`
- stdlib artifact invalidation test
- Resource IR summary cache invalidation test
- `node nodesrc/issues.js check --dir issues`
