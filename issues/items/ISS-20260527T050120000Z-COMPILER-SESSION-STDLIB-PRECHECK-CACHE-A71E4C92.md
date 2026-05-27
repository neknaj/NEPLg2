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

## 問題

現在の API は compile call ごとに loader / source map / parse / import / typecheck / Resource IR / codegen を新規に構築する。stdlib source は bundled になっても、stdlib の parse/import/typecheck artifact と Resource IR summary template は session 間で再利用されない。

このため、同一 process 内であっても entry source の微小変更に対し、変更されていない stdlib と unchanged user functions の query result を再利用する構造が不足している。

## 影響

Web playground、Node doctest runner、selfhost compiler 開発で、実行時間ではなく compile phase が feedback loop を支配する。静的検査を強化するほど同じ stdlib graph の再検査が増え、Zenn 方針の「純粋性と静的検査を活かした performance 追求」に反する。

## 修正方針

[NEPLg2.1 compiler performance / cache design 2026-05-27](../../doc/neplg2/compiler_performance_cache_design.md) に沿って、`CompilerSession` と stdlib prechecked artifact を導入する。

MVP は次の順に進める。

1. `nepl-core` に source text / lex / parse / import graph / type arity を query として分離する session API を追加する。
2. `nepl-web` に `CompilerSession` wasm-bindgen class を公開し、bundled stdlib の parsed module / import graph / type arity を warm state として保持する。
3. stdlib artifact に public signature table、trait impl index、source capability tableを持たせ、通常 compile では entry source と overlay source だけを新規処理する。
4. Resource IR summary を function hash + source capability hash + type argument hash で cache し、entry から到達する changed functions だけを再計算する。
5. codegen fragment cache を function hash 単位にし、unchanged fragments を signature/index table へ再接続する。

## 完了条件

- release WASM + warm `CompilerSession` で、最小 entry source の同一 compile と 1 行変更 compile が 10ms 未満になる。
- aggregate/generic の小規模 program でも、stdlib artifact が unchanged の場合は 10ms 台を安定して維持する。
- local stdlib が release artifact より新しい場合は cache を使わず、FS stdlib override / artifact refresh に戻る。
- raw LLVM、raw wasm direct call、indirect call、曖昧な function reference は conservative-all で検査漏れしない。
- stale diagnostic span や stale source capability が別 source へ流用されないことを regression test で固定する。

## 検証

- `trunk build --release`
- `node nodesrc/run_test.js` minimal / aggregate timing
- session API の unit test
- stdlib artifact invalidation test
- Resource IR summary cache invalidation test
- `node nodesrc/issues.js check --dir issues`
