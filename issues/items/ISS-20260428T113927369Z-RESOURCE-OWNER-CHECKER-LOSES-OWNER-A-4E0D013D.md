---
id: ISS-20260428T113927369Z-RESOURCE-OWNER-CHECKER-LOSES-OWNER-A-4E0D013D
title: "Resource owner checker loses owner arguments returned by unknown callbacks"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T113927369Z-RESOURCE-OWNER-CHECKER-LOSES-OWNER-A-4E0D013D: Resource owner checker loses owner arguments returned by unknown callbacks

## 概要

ResourceOwnerCheckEngine applies owner return summaries to direct calls and known function values, but an IndirectCall with no known callee alias does not conservatively transfer owner arguments to the output. An unknown callback can return an owner argument of the same result type, while the caller keeps the original argument live and the output has no free obligation.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T112744824Z-RESOURCE-OWNER-CHECKER-LOSES-OWNERS--BB7D4D5F` で direct call、`ISS-20260428T113234640Z-RESOURCE-OWNER-CHECKER-LOSES-OWNER-R-AE831E7E` で known function value の owner return summary は追加された。
- しかし callee alias がない `ResourceOp::IndirectCall` は owner checker で保守的に扱われておらず、callback parameter が owner 引数を戻り値にする可能性を反映できなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は callback 境界でも owner/free obligation を落とさない Resource IR 化を求めている。
- unknown callback に fresh allocation を仮定すると過剰になるが、同型の live owner 引数を返す可能性は caller の既存 obligation から保守的に transfer できる。

## 問題

ResourceOwnerCheckEngine applies owner return summaries to direct calls and known function values, but an IndirectCall with no known callee alias does not conservatively transfer owner arguments to the output. An unknown callback can return an owner argument of the same result type, while the caller keeps the original argument live and the output has no free obligation.

## 影響

Owner/free-obligation checks remain dependent on knowing the exact callback target. Higher-order self-host or stdlib code can hide owner transfer behind callback parameters, producing missed leaks or incorrect deallocation diagnostics.

## 修正方針

For IndirectCall with no known callee aliases, conservatively transfer the first live owner argument whose type matches the output type to the output. Keep known aliases on precise owner return summaries.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 unknown callback owner argument return 対応

`ResourceOwnerCheckEngine::apply_indirect_call_return_owner` で、callee が known function alias を持たない場合の fallback を追加した。unknown callback は output と型が一致する live owner 引数を返し得るため、その owner を output へ transfer する。

known function value の場合は従来通り computed owner return summary を使う。fresh allocation は unknown callback に対して無条件に仮定せず、既存 owner 引数の transfer に限定して過剰診断を避ける。

`nepl-core/tests/resource_ir.rs` に unknown callback が owner 引数を返し、caller が戻り値を dealloc する正常系を追加した。
