---
id: ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04
title: "core/mem raw memory operations bypass effect and ownership checks"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/core/mem.nepl, tests/compiler/move_effect.n.md"
---

# ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04: core/mem raw memory operations bypass effect and ownership checks

## 概要

`stdlib/core/mem.nepl` は `alloc_raw` / `dealloc_raw` / `realloc_raw` / `load` / `store` を pure function signature として公開している。一方、`nepl-core` の effect 判定は既知 WASI call だけを impure とするため、raw memory 操作が pure 文脈から観測可能なまま呼べる。

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, stdlib/core/mem.nepl, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/ast.rs` の `Effect` は `Pure` / `Impure` の 2 値だけで、`InternalAlloc` や `UnsafeMemory` の内部効果を表現できない。
- `nepl-core/src/effects.rs` の `intrinsic_effect` は既知 WASI marker だけを `Impure` とし、それ以外の intrinsic を `Pure` とする。
- `stdlib/core/mem.nepl` の `alloc_raw` / `dealloc_raw` / `realloc_raw` / `load<T>` / `store<T>` は `*` なしの pure signature で公開されている。
- `nepl-core/src/passes/move_check.rs` と `nepl-core/src/passes/drop_insertion.rs` は intrinsic `load` / `store` を field move などの局所 pattern として扱うが、任意 raw address が所有 place かどうかは追跡しない。
- `doc/compare/memory_model.md` は Phase 0 で `alloc/dealloc/realloc/load/store` を `Effect::Pure` から `Effect::InternalAlloc` へ移す計画を明記しているが、実装側 issue としては未分離だった。

## 問題

`move_check` と `drop_insertion` は intrinsic `load` / `store` を field move などの局所 pattern として special-case しているが、任意の `MemPtr` / raw address がどの owning place に属するかは追跡しない。そのため、raw memory から non-Copy 値を浅く読み出す経路や、pure 関数内で raw address identity を観測しながら allocate/free する経路を、effect / ownership 検査が正しく表現できない。

## 影響

pure source code が observable raw address を allocate / free / load / store でき、non-Copy 値を owned place 外の raw memory から浅く複製できる。self-host compiler の AST / diagnostic / buffer が owning value を増やすほど、effect、borrow、type safety の前提が崩れる。

## 2026-04-27 部分対応

`move_check` に raw memory place の所有権状態を追加し、`load<T>` / `store<T>` および lowered intrinsic `load` / `store` が non-Copy 値を扱う場合は、raw address を owning place として検査するようにした。

- non-Copy `load<T>` は raw place からの move として扱い、同じ place からの二重 load を `D3100` で拒否する。
- non-Copy `store<T>` は raw place の初期化として扱い、未 move の non-Copy 値を含む place への上書きを `D3100` で拒否する。
- `let q p` や `let q add p 4` のような i32 raw address alias を scope / branch snapshot に追従して正規化し、alias 経由の二重 load を拒否する。
- branch 間で raw place 状態が分岐する場合は `PossiblyMoved` として合流し、後続の non-Copy load / store を安全側で拒否する。

この対応は ownership 検査の穴を塞ぐもので、effect model の不足はまだ残る。`alloc_raw` / `dealloc_raw` / `realloc_raw` / `load` / `store` の pure API、`InternalAlloc` / `UnsafeMemory` 相当の effect 導入、stdlib API 移行が必要になる場合の stdlib 側修正は、この issue の残件または別 issue として扱う。

## 2026-04-28 compiler / mem 責務分割レビュー追記

今回の責務分割レビューでは、この issue はまだ閉じられないと判断した。`move_check` の raw place state は non-Copy raw load/store の二重 move をかなり塞いだが、根本の境界はまだ `core/mem.nepl` の public raw API と compiler の effect / provenance model に残っている。

- `stdlib/core/mem.nepl:104` / `107` の `mem_ptr_wrap` / `mem_ptr_addr` により safe source code が raw `i32` address と `MemPtr<T>` を相互変換できる。
- `stdlib/core/mem.nepl:278` / `386` / `450` の allocator primitive と、`1101` / `1117` の generic raw `load<T>` / `store<T>` は pure signature のまま公開されている。
- `nepl-core/src/typecheck.rs:2491` の raw body effect validation は direct callee だけを確認し、memory instruction 自体を分類しない。
- `nepl-core/src/runtime_helpers.rs:8` 以降は compiler 内部 allocator helper discovery を public `alloc_raw` / `dealloc_raw` / `realloc_raw` 名に依存している。

この issue は raw memory operation 全体の親 issue とし、今回のレビューで不足していた追跡単位を次の issue に分割した。

- `ISS-20260427T152947135Z-RAW-BODY-MEMORY-INSTRUCTIONS-BYPASS--162A8C00`: raw body memory instruction が pure effect validation を通らない。
- `ISS-20260427T152951013Z-RUNTIME-ALLOCATOR-HELPER-LOOKUP-DEPE-D070168E`: compiler runtime helper lookup が public `core/mem` 名に依存している。
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`: `core/mem` が safe API として raw address escape hatch を公開している。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: `MemPtr` / `RegionToken` に compiler-owned provenance model がない。

## 2026-04-28 raw memory intrinsic effect 部分対応

`#intrinsic "load"` / `#intrinsic "store"` が `intrinsic_effect` で pure 扱いになっていたため、user source が `core/mem` の wrapper を通さず raw memory を直接読み書きできる穴を `ISS-20260427T160936494Z-RAW-MEMORY-INTRINSICS-ARE-TREATED-AS-C0657AB6` として分離し、修正した。これにより direct raw memory intrinsic は pure context で `D3025` になる。移行中の `stdlib/core/mem.nepl` は SourceMap path による compiler-owned memory boundary として限定許可している。

## 修正方針

`InternalAlloc` / `UnsafeMemory` のような内部 memory effect を導入し、raw identity が観測できない場合だけ surface `Pure` へ畳み込む。raw `load` / `store` / `alloc` / `dealloc` は unsafe 層または compiler-owned boundary に閉じ込める。Resource IR では memory token / place を表現し、non-Copy raw load は unrestricted copy ではなく owning place からの move として扱う。

## 検証

raw identity が観測可能な public raw memory operation を pure function から呼ぶ compile_fail を追加する。同じ raw place から non-Copy 値を繰り返し `load` する case も、将来の明示 unsafe escape がない限り拒否する ownership test を追加する。`MemPtr` safe overload の正常系は別途維持する。

2026-04-27 の部分対応では、`tests/compiler/move_effect.n.md` に non-Copy raw load の二重 move、raw address alias 経由の二重 move、未 move raw place への store overwrite、load 後の再初期化の回帰テストを追加した。
