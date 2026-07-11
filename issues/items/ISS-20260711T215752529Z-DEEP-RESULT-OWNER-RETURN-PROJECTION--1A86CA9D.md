---
id: ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D
title: "Deep Result owner return projection reuses moved payload"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-11
updated: 2026-07-12
target: nepl-core/src/resource/owner_return_apply_projection.rs
---

# ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D: Deep Result owner return projection reuses moved payload

## 概要

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 対象

- `nepl-core/src/resource/owner_return_apply_projection.rs`

## 根拠

- GUI production chainでは`writer_step_budget(owner, 1)`の`Result<BudgetStep, StepError>`適用時に、同じregistered source authority由来の複数deep owner leafが順番に`ReturnValue` transferされ、2件目以降が`Moved`として拒否される。
- outer/inner enum pathをpayload bindへ遅延する試作は単純な`Result<Step, E>`では元の誤報を消したが、GUIではstruct fieldと複数の内部owner enumを挟むleaf群が同じsourceへ解決されるため誤報が残った。
- moved sourceを単にskipする方法は、別variantが選択された場合にownerを返せなくなる。outer variant名だけの平坦化や最初のleaf採用では正当なowner transferを証明できない。
- 同一`OwnerProjectionSource`だけを排他的deep targetの共通prefixへaggregate化する試作はunit対照を通過したが、productionではsource suffixが異なるleaf同士がruntime alias解決で同じaggregate authorityへ収束するため対象にならず、誤報が残った。authority groupはsource path完全一致ではなくcallee `StorageId`由来の抽象groupとしてsummaryへ保存する必要がある。
- callee `StorageId`ごとのabstract group schemaも試作したが、production parameter seedはdeep owner leafごとに新しいStorageIdを割り当てるため、StorageId partitionはaggregate authority relationを表さなかった。人工的に同じStorageIdを与えるunitだけではproduction契約を証明できない。

## 問題

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 影響

Valid production owner chains cannot be exercised by runtime fixtures; F5nxj integration is blocked despite normal compile and source-policy gates passing.

## 修正方針

GUIから抽出したstruct fieldと内部owner enumを含む複数leaf最小再現を追加する。まずparameter aggregate rootと各seed leafのauthority relationを型構造・owner-token境界・alias contractから明示し、leafごとのStorageIdとは別のstable function-local abstract groupを発行する。source leafと排他的return target leafを同じgroupへ束ね、適用側はgroupを論理予約して選択pathへ1回だけmoveする。同じaggregate moveをleafごとの独立moveとして再適用せず、genuine use-after-move診断、異なるauthority groupを持つ複数leaf transfer、非排他的複製拒否を維持する。

## 検証

Run the minimized Resource IR regression and tests/stdlib/gui_font_registered_face.n.md with the F5nxj controlled 8-command runtime contract: read retry, zero/negative budget, partial seal, eight writes, terminal completion, checked seal, and cleanup-only push failure.
