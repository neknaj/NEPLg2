---
id: ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D
title: "core mem exposes raw address escape hatches as safe API"
area: stdlib
status: open
resolved: false
priority: P1
type: security
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/core/mem.nepl, stdlib/core/traits/copy.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D: core mem exposes raw address escape hatches as safe API

## 概要

`stdlib/core/mem.nepl` は `MemPtr<T>` を導入しているが、同時に raw `i32` address への unwrap / wrap と raw load/store を safe public API として公開している。結果として compiler が pointer provenance、bounds、ownership、effect を管理する前に、stdlib と利用者 code が raw address へ降りられる。

## 対象

- `stdlib/core/mem.nepl, stdlib/core/traits/copy.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- `stdlib/core/mem.nepl:97` で `MemPtr<T>`、`100` で `RegionToken<T>` を定義している。
- `stdlib/core/mem.nepl:104` の `mem_ptr_wrap` と `107` の `mem_ptr_addr` が raw `i32` と `MemPtr<T>` を双方向変換できる。
- `stdlib/core/mem.nepl:278` / `386` / `450` の `alloc_raw` / `dealloc_raw` / `realloc_raw` は raw `i32` address を公開する。
- `stdlib/core/mem.nepl:558` / `591` の raw `load_i32(i32)` / `store_i32(i32,i32)` と、`1101` / `1117` の generic raw `load<T>(i32)` / `store<T>(i32,T)` が public に見える。
- `stdlib/core/traits/copy.nepl:151` / `155` で `MemPtr<T>` は `Clone` / `Copy` になっており、コメント上も non-owning address とされている。
- `doc/compare/memory_model.md:47` は Phase 1 で `mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` / `realloc_raw` を公開面から除く計画を明記している。

## 問題

`MemPtr<T>` を safe pointer wrapper として扱うには、raw address への変換、任意型 load/store、allocator primitive は unsafe または compiler-owned boundary に閉じる必要がある。現状では safe source code から raw `i32` を作り、pointer arithmetic 後に任意型として読み書きできるため、`MemPtr<T>` の型引数は provenance や ownership の証明になっていない。

## 影響

型安全上は `MemPtr<T>` から別 `U` の pointer を作る、所有値を raw memory から浅く複製する、dealloc 済み address を再利用する、といった経路を compiler が根本的に遮断できない。メモリ安全上は double free / use-after-free / uninitialized read を safe API から構成できる。self-host stdlib の collection と diagnostic storage が増えるほど被害範囲が広がる。

## 修正方針

public `core/mem` は checked allocation、typed pointer arithmetic、copy-only load/store、owned move in/out のような safe operation に限定する。raw `i32` address 変換と generic raw load/store は non-public または明示 unsafe module へ分離し、compiler 側の Resource IR / effect model と同期して移行する。`MemPtr<T>` は non-owning pointer と明示し、owner token / storage handle は別型へ分ける。

## 検証

safe import だけでは `mem_ptr_addr` / `mem_ptr_wrap` / raw `load<T>` / raw `store<T>` / raw allocator primitive を呼べない compile_fail を追加する。safe wrapper は bounds error を `Result` / `Option` で返す正常系を維持する。raw escape が必要な既存 stdlib は unsafe/internal module へ寄せ、使用箇所を source policy で追跡する。
