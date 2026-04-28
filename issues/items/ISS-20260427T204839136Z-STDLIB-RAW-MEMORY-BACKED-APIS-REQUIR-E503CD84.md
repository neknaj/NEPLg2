---
id: ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84
title: "stdlib raw-memory-backed APIs require staged effect migration"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/core/mem.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, nepl-core/src/typecheck.rs"
---

# ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84: stdlib raw-memory-backed APIs require staged effect migration

## 概要

core/mem raw primitives are still exposed as Pure because large parts of stdlib call alloc_raw/dealloc_raw/load/store from functions whose public signatures are currently pure. Marking those primitives impure in the compiler immediately causes widespread D3025 failures in Vec, string, IO, fs, diag, and stdio helpers.

## 対象

- `stdlib/core/mem.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/std/fs.nepl, stdlib/std/stdio.nepl, stdlib/std/streamio.nepl, nepl-core/src/typecheck.rs`

## 根拠

- `tests/compiler/move_effect.n.md` の既存正常系「pure からメモリ操作を呼べる」は、pure `compute` から `alloc_raw` / `store_i32` / `load_i32` / `dealloc_raw` を呼び、`ret: 123` で通る。
- compiler 側で raw memory boundary 内の raw primitive を `Effect::Impure` として登録する試作を行うと、`tests/compiler/move_effect.n.md` の stdlib import ケースで `stdlib/alloc/collections/vec.nepl:278` の `store<.T>`、`vec.nepl:648` の `load<.T>`、`stdlib/alloc/string.nepl`、`stdlib/std/fs.nepl`、`stdlib/std/stdio.nepl`、`stdlib/std/streamio.nepl` などが一斉に D3025 になる。
- これは compiler の effect 判定だけの問題ではなく、stdlib が raw memory backed helper を pure API として公開・利用している設計移行の問題である。

## 問題

core/mem raw primitives are still exposed as Pure because large parts of stdlib call alloc_raw/dealloc_raw/load/store from functions whose public signatures are currently pure. Marking those primitives impure in the compiler immediately causes widespread D3025 failures in Vec, string, IO, fs, diag, and stdio helpers.

## 影響

The compiler cannot close the raw memory effect boundary issue without either breaking current stdlib APIs or introducing a richer internal memory effect. Pure source can still reach allocation or raw storage through stdlib wrappers, so the effect model remains unsound for self-host planning.

## 修正方針

Stage the migration: introduce a compiler-level internal/unsafe memory effect or explicit stdlib unsafe boundary, update raw-memory-backed stdlib APIs to either be impure or wrap an internal effect safely, and only then make core/mem raw primitives externally impure by default.

## 検証

Add compile_fail tests for user pure calls to raw primitives and stdlib migration tests that show intended safe wrappers either require impure context or are proven pure through the new internal effect boundary.

## 2026-04-28 memory model 方針レビュー追記

現在の stdlib は `Vec`、`string`、`io`、`fs`、`stdio`、`streamio` などの実装で raw memory backed helper を pure API の内部から呼んでいる。内部 mutation を使うこと自体は問題ではないが、raw address や storage identity が safe surface へ漏れる場合は referential transparency を compiler が証明できない。

したがって stdlib 側の方向は「raw memory を使わない」ではなく、「raw memory を public API discipline として利用者へ押し出さない」に修正する。具体的には、内部 builder / collection storage は `InternalAlloc` と owner token に閉じ、公開 API は Copy read、move-out、drop/free obligation を型と Resource IR で区別する。

self-host 実装では、S1/S2 の文字列走査・token 配列・diagnostic からこの方針を適用する。短期的には既存 `Vec` / `StringBuilder` を使って開始できるが、compiler 本体へ raw `MemPtr` 操作を直接持ち込む実装は避ける。
