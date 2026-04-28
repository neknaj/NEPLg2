---
id: ISS-20260428T113234640Z-RESOURCE-OWNER-CHECKER-LOSES-OWNER-R-AE831E7E
title: "Resource owner checker loses owner returns through function values"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T113234640Z-RESOURCE-OWNER-CHECKER-LOSES-OWNER-R-AE831E7E: Resource owner checker loses owner returns through function values

## 概要

ResourceOwnerCheckEngine now applies owner return summaries to direct ResourceOp::Call, but ResourceOp::FunctionValue and ResourceOp::IndirectCall are still ignored by the owner checker. A known function value can call a helper that returns a fresh owner or transfers an owner argument, while the caller output remains without a free obligation.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T112744824Z-RESOURCE-OWNER-CHECKER-LOSES-OWNERS--BB7D4D5F` で direct call の owner return summary は追加された。
- しかし `ResourceOwnerCheckEngine` は `ResourceOp::FunctionValue` と `ResourceOp::IndirectCall` を owner 境界として扱わず、known function value 経由では summary が適用されなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は owner token / free obligation を function / callback 境界でも落とさない Resource IR 化を要求している。
- Resource IR には `FunctionValue` output と `IndirectCall` callee place が残っているため、known alias には direct call と同じ owner return summary を適用できる。

## 問題

ResourceOwnerCheckEngine now applies owner return summaries to direct ResourceOp::Call, but ResourceOp::FunctionValue and ResourceOp::IndirectCall are still ignored by the owner checker. A known function value can call a helper that returns a fresh owner or transfers an owner argument, while the caller output remains without a free obligation.

## 影響

Owner/free-obligation diagnostics still depend on syntactic direct calls. Higher-order helper code in self-host or stdlib can hide leaks and owner transfers behind first-class functions, leaving Stage 4 owner checking incomplete.

## 修正方針

Track known FunctionValue aliases in the owner checker, merge aliases through local copies and branches, and apply direct owner return summaries to ResourceOp::IndirectCall when the callee has a known alias.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 owner function value summary 対応

borrow checker 用に導入していた function alias tracking を汎用 `FunctionAliasTable` として整理し、owner checker でも利用するようにした。`FunctionValue`、local copy / move / assign、branch / loop / match merge で known callee alias を保持する。

`ResourceOp::IndirectCall` の callee が known function alias を持つ場合は、direct call と同じ owner return summary を適用する。fresh owner return は call output を live owner にし、parameter-to-return owner transfer は caller の live owner 引数を output へ移す。

`nepl-core/tests/resource_ir.rs` に function value 経由で fresh allocation owner が leak する経路の検出回帰と、owner 引数を function value 経由で戻して caller が dealloc する正常系を追加した。
