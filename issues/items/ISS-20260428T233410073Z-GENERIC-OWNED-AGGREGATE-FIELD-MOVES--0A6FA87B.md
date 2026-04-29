---
id: ISS-20260428T233410073Z-GENERIC-OWNED-AGGREGATE-FIELD-MOVES--0A6FA87B
title: "generic owned aggregate field moves still reject SelfhostOutcome direct Result cleanup"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/typecheck/field_apply.rs, nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/passes/move_check, nepl-core/tests/move_check.rs, tests/stdlib/neplg2_diag_outcome.n.md"
---

# ISS-20260428T233410073Z-GENERIC-OWNED-AGGREGATE-FIELD-MOVES--0A6FA87B: generic owned aggregate field moves still reject SelfhostOutcome direct Result cleanup

## 概要

After SelfhostOutcome stores Result<T,E> directly, selfhost_outcome_result/selfhost_outcome_free must move both result and diagnostics fields from SelfhostOutcome<T,E>. For Result<DropCounter,str>, move_check reports D3053 use of moved value on the second field extraction even though the fields are disjoint.

## 対象

- `nepl-core/src/typecheck/field_apply.rs, nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/passes/move_check, nepl-core/tests/move_check.rs, tests/stdlib/neplg2_diag_outcome.n.md`

## 根拠

- `SelfhostOutcome<T,E>` を `result <Result<T,E>>` / `diagnostics <SelfhostDiagnostics>` の直接 owned field へ変更した後、`node nodesrc/tests.js -i tests\stdlib\neplg2_diag_outcome.n.md --no-tree -o tmp\outcome-direct-result-fixture-4.json -j 1` を実行した。
- `tests\stdlib\neplg2_diag_outcome.n.md::doctest#3` の `selfhost_outcome_free<DropCounter,str>` 経路で、`selfhost_outcome_result` 内の 2 つ目の field extraction が `error[D3053]: use of moved value: outcome` になった。
- 同じ issue family の `ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE` は verified だが、今回の再現は `SelfhostOutcome<.T,.E>` の generic field `Result<.T,.E>` が実体化後に non-Copy になるケースで、既存修正の対象から漏れている。

## 問題

After SelfhostOutcome stores Result<T,E> directly, selfhost_outcome_result/selfhost_outcome_free must move both result and diagnostics fields from SelfhostOutcome<T,E>. For Result<DropCounter,str>, move_check reports D3053 use of moved value on the second field extraction even though the fields are disjoint.

## 影響

SelfhostOutcome can remove the raw result cell for Copy payload smoke tests, but non-Copy payload cleanup remains blocked by a compiler partial-move regression. Keeping the raw pointer workaround would hide this compiler issue and retain unsafe stage storage.

## 修正方針

`field::get` / `get_field` を型検査段階で `load(add(...))` に早期 lowering すると、move_check が field selector を復元できず、generic 実体化後に offset/type が衝突する field を同一 move と誤判定する。`RawMemoryLoadCell` や move check を弱めず、HIR 上に `get_field(base, selector)` を残して selector identity を静的検査へ渡す。

move_check 側は selector から field index を取得できる場合に `field_index` を move identity として保持し、raw address 由来で index が一意に復元できない場合だけ従来の offset/type fallback を使う。これにより disjoint field は一度ずつ move でき、同一 field の二重 move、partial move 後の owner use、borrow 中 move は引き続き拒否する。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 検証

- `cargo test -p nepl-core --test move_check move_generic_ -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test move_check -- --nocapture`: 55 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-generic-aggregate-field-move-compiler-move-check.json -j 1`: total=52, passed=52, failed=0
- `node nodesrc\tests.js -i tests\stdlib\neplg2_diag_outcome.n.md --no-tree -o tmp\agent1-generic-aggregate-field-move-outcome-2.json -j 1`: total=3, passed=3, failed=0
- `rustfmt --check nepl-core\src\passes\move_check\state.rs nepl-core\src\passes\move_check\context_state.rs nepl-core\src\passes\move_check\alias.rs nepl-core\src\passes\move_check\provenance.rs nepl-core\src\passes\move_check\visitor.rs nepl-core\src\typecheck\field_apply.rs nepl-core\src\typecheck\prefix_check.rs nepl-core\src\typecheck\hir_finalize.rs nepl-core\tests\move_check.rs`: pass
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

`SelfhostOutcome<DropCounter,str>` の `result` / `diagnostics` field move は、field selector を保持したまま move_check へ渡すことで正しく disjoint move と判定されるようになった。generic struct の non-Copy field 2 個を別々に move できる positive regression と、同じ field を 2 回 move した場合の negative regression を追加し、検査の形骸化ではなく field identity の復元として修正した。
