---
id: ISS-20260428T112744824Z-RESOURCE-OWNER-CHECKER-LOSES-OWNERS--BB7D4D5F
title: "Resource owner checker loses owners returned by helpers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T112744824Z-RESOURCE-OWNER-CHECKER-LOSES-OWNERS--BB7D4D5F: Resource owner checker loses owners returned by helpers

## 概要

ResourceOwnerCheckEngine tracks RawMemory::Alloc and Dealloc inside a function, but ResourceOp::Call is ignored. A helper can allocate or receive an owned pointer and return it to the caller; the callee moves the owner out through Return, while the caller never receives a free obligation for the call output.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は owner token / free obligation を Resource IR 上で検査する計画である。
- `ResourceOwnerCheckEngine` は `RawMemory::Alloc` と `RawMemory::Dealloc` を同一関数内では追跡するが、`ResourceOp::Call` を無視していた。
- callee 側は `Return` で owner を move-out するため leak にならない。一方で caller 側の call output は `NoFreeObligation` のままになり、helper が返した owner の leak や transfer を検出できない。
- Resource IR には direct user call target、引数 place、output place があるため、function summary で owner return を caller に伝播できる。

## 問題

ResourceOwnerCheckEngine tracks RawMemory::Alloc and Dealloc inside a function, but ResourceOp::Call is ignored. A helper can allocate or receive an owned pointer and return it to the caller; the callee moves the owner out through Return, while the caller never receives a free obligation for the call output.

## 影響

Free-obligation diagnostics depend on inlining-like local allocation shapes. Self-host or stdlib helper functions can hide leaks and owner transfers across direct function boundaries, weakening Stage 4 owner/resource checking.

## 修正方針

Compute direct user function owner-return summaries for fresh owner returns and parameter-to-return owner transfers. Apply the summary at ResourceOp::Call so the caller receives a live owner for fresh returns and transfers live owner state from owner arguments to call outputs.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 owner return summary 対応

`ResourceOwnerCheckEngine` に direct user function の owner return summary を追加した。summary は、関数が fresh owner を返すか、どの owner 引数を戻り値へ transfer し得るかを固定点で計算する。

`ResourceOp::Call` では summary を適用し、fresh owner return なら call output を live owner として登録する。parameter-to-return owner transfer なら caller の live owner 引数を output へ移し、元の引数を moved にする。

`nepl-core/tests/resource_ir.rs` に、helper が fresh allocation を返したまま caller が解放しない場合の leak 検出回帰と、helper が owner 引数を返して caller が戻り値を dealloc する正常系を追加した。
