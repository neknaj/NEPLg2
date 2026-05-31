---
id: ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649
title: "binary intermediate artifacts needed for incremental compile"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/compiler.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_performance_cache_design.md"
---

# ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649: binary intermediate artifacts needed for incremental compile

## 概要

現状の NEPLg2 は typed HIR、Resource summary、compiled output cache を主に process-local memory に保持している。JVM の `.class` や C 系の `.o` に相当する、session をまたいで再利用できる永続 binary intermediate artifact はまだ持っていない。

## 対象

- `nepl-core/src/compiler.rs; nepl-web/src/lib.rs; doc/neplg2/compiler_performance_cache_design.md`

## 根拠

- Zenn 方針では、純粋性、依存関係の DAG 化、静的検査、cache により探索範囲と計算量を削減することが要求されている。
- `.class` は verifier が検査できる typed bytecode、`.o` は linker が再接続できる target-specific relocatable fragment である。NEPLg2 ではこの 2 つを混ぜず、platform-neutral typed/proof artifact と target-specific codegen fragment を分離する必要がある。
- stdlib や selfhost compiler は大きくなっており、毎回 source から全 pipeline を再構築すると 0.5 秒未満 compile / 0.1 秒未満 warm recompile / 10ms 級 expression edit に届かない。

## 問題

現行の compiled-output cache は source 全体 key が同一の場合には強いが、小さな式枝差し替えでは miss し、typecheck、Resource IR、summary replay、codegen を広く再実行する。Resource summary value cache は function-level proof の一部を保持できるが、typed public surface、typed HIR、Resource proof、codegen fragment を統一した永続 artifact boundary にはなっていない。

## 影響

安定した中間 artifact 境界がないままだと、リテラル変更や小さな式枝差し替えで source 全体 key が変わり、変更されていない module / function / stdlib proof / codegen fragment まで再処理される。この状態では 0.1 秒 warm recompile と 10ms 級の微小差分 compile target が構造的に達成しにくい。

## 修正方針

NEPL object artifact stack を `.nepl...` 形式の artifact として設計し、段階的に実装する。
短い `.nei` / `.nehir` / `.ners` / `.neo` 形式は採用しない。NEPL 固有の artifact で
あることを拡張子から確認できるようにし、役割名も読み取れる形にする。

- `.neplmeta`: import graph、exported type/function/trait impl surface、effect signature、typed public signature、source capability policy surface を保持する。
- `.neplhir`: stable lexical path id、typed HIR、typed diagnostics enum、local binding shape、expected type boundary を保持する。永続化は stable typed id 導入後に行い、MVP では same-session cache に限定する。
- `.neplproof`: Resource IR summary、private effect mask proof、drop/borrow/owner/initialized proof summary を stable mirror として保持する。
- `.neplobj`: wasm / LLVM の function fragment、signature table entry、function table entry、data segment、relocation metadata を保持する。
- `.nepllink`: fragment の symbol / relocation / table index / data offset を再接続し、final wasm / LLVM artifact を生成する。

cache key には compiler version、artifact schema version、target/profile、stdlib content hash、module public surface hash、dependency public surface hash、source capability policy hash、type/effect boundary hash、generic type arguments、backend feature set を含める。どれかが再投影できない場合は stale hit を避けるため fail-closed に再計算する。

実装順序は `.neplmeta`、`.neplproof`、same-session `.neplhir` query cache、`.neplobj` /
`.nepllink`、persistent `.neplhir` とする。`.neplobj` を先に作る案は採用しない。
NEPLg2 の prefix call boundary は依存 module の callable candidate / arity / effect /
generic surface を必要とするため、interface artifact なしでは `.o` 相当を持っても
再型検査の支配コストを削れないからである。

## 検証

same-session と cross-session の RPN expression edit を測定し、次を確認する。

- public surface が不変の edit で、stdlib / dependency module artifact が再利用される。
- changed function と dependent Resource summary だけが再計算される。
- unchanged codegen fragment は relocation/link だけで再接続される。
- final wasm は full compile と同一の挙動を持つ。
- source capability、generic substitution、diagnostic span、private effect mask proof を再投影できない場合は cache hit せず安全側に再計算する。
