---
id: ISS-20260516T031913953Z-SOURCE-CAPABILITY-PROOF-TRAVERSAL-IS-9E939884
title: "Source capability proof traversal is duplicated per capability domain"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/source_capability/**, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260516T031913953Z-SOURCE-CAPABILITY-PROOF-TRAVERSAL-IS-9E939884: Source capability proof traversal is duplicated per capability domain

## 概要

SourceCapabilities no longer depend on path-only stdlib allowlists, but raw memory and owner aggregate still each implement their own AST traversal, scope update, prefix call-head detection, and raw body visit logic. This is not a per-module allowlist, but it is still a per-capability proof engine split. Future capability additions can miss match arm scopes, initializer call-head positions, raw body evidence, or shadowing behavior independently.

## 対象

- `nepl-core/src/source_capability/**, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/source_capability/raw_memory.rs` と `nepl-core/src/source_capability/owner_aggregate.rs` が、それぞれ `Block` / `Stmt` / `FnBody` / `PrefixExpr` / `PrefixItem` を直接走査していた。
- どちらも `SourceCapabilityScope` の block local / match arm binding 更新と `PrefixCallHead` による call-head 判定を独自に呼んでおり、片方だけ initializer call-head や shadowing 更新を取り逃す余地があった。
- `nodesrc/test_static_check_boundary_responsibility.js` も旧構造の関数名を監視していたため、共通 proof traversal の存在そのものを退行防止できていなかった。

## 問題

SourceCapabilities no longer depend on path-only stdlib allowlists, but raw memory and owner aggregate still each implement their own AST traversal, scope update, prefix call-head detection, and raw body visit logic. This is not a per-module allowlist, but it is still a per-capability proof engine split. Future capability additions can miss match arm scopes, initializer call-head positions, raw body evidence, or shadowing behavior independently.

## 影響

Static-check proof code becomes hard to audit: the compiler cannot rely on one source-level proof traversal invariant, and mistakes in the checker itself are less likely to be caught by enum/match coverage or source policy. This conflicts with the policy that source properties should be proven by a generic compiler mechanism rather than local one-off proof engines.

## 修正方針

Introduce a shared source-capability proof walker that owns AST traversal, SourceCapabilityScope updates, prefix call-head recognition, intrinsic descent, and raw body observation. Raw-memory and owner-aggregate modules should become observers/classifiers over that shared traversal. Update source policy to require the shared walker and reject renewed direct traversal duplication.

## 解決内容

- `nepl-core/src/source_capability/walk.rs` を追加し、module / block / stmt / function body / prefix expression / intrinsic / match arm / raw body の走査を `SourceCapabilityObserver` による共通 proof traversal に集約した。
- raw memory proof は `RawMemoryCollector` として shared walker の `observe_call_head_symbol` / `observe_intrinsic` / `observe_raw_body` / function start/end callback を受け取り、raw helper definition の「body に evidence がある場合だけ関数名 operation を許可する」規則を維持した。
- owner aggregate proof は `OwnerAggregateCollector` として同じ walker の call-head / intrinsic / alias callback を受け取り、field import provenance と constructor evidence の分類だけを担当するようにした。
- source policy を更新し、`walk.rs` の存在、prefix call-head tracking、scope update、match arm binding、raw body callback を監視する。raw memory / owner aggregate module が `PrefixCallHead` や `PrefixItem::Match` を再実装する退行も拒否する。

## 検証

- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `cargo fmt -p nepl-core -- --check`
- `git diff --check`
- `trunk build`
