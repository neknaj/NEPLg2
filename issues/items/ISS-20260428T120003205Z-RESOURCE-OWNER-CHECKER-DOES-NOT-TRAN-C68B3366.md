---
id: ISS-20260428T120003205Z-RESOURCE-OWNER-CHECKER-DOES-NOT-TRAN-C68B3366
title: "Resource owner checker does not transfer aggregate owner descendants returned by helpers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T120003205Z-RESOURCE-OWNER-CHECKER-DOES-NOT-TRAN-C68B3366: Resource owner checker does not transfer aggregate owner descendants returned by helpers

## 概要

Owner return summaries can mark a parameter as returned, and transfer_owner can move descendant owner projections, but summary application only calls transfer_owner when the exact argument place has a live owner. Aggregate values usually carry owners under field projections, so id_wrapper(wrapper) leaves the owner on the old wrapper field and the returned wrapper output has no free obligation.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T114942018Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-B551A456` で `transfer_owner` は source 配下の owner projection を target 配下へ移せるようになった。
- しかし owner return summary の適用側は `owners.state(arg) == Live` の exact owner だけを条件にしていたため、aggregate parameter の field projection にある owner を transfer しなかった。
- `ISS-20260428T115405922Z-RESOURCE-OWNER-CHECKER-LOSES-AGGREGA-DE066CEF` で aggregate return projection は caller に伝播できるようになったが、`fn id_wrapper(w): w` のような parameter-to-return aggregate identity では exact summary に descendant owner transfer を適用する必要がある。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は function boundary でも owner/free obligation と value movement を一致させることを要求している。

## 問題

Owner return summaries can mark a parameter as returned, and transfer_owner can move descendant owner projections, but summary application only calls transfer_owner when the exact argument place has a live owner. Aggregate values usually carry owners under field projections, so id_wrapper(wrapper) leaves the owner on the old wrapper field and the returned wrapper output has no free obligation.

## 影響

Wrapper/helper APIs can lose owner obligations for aggregate values. A self-host or stdlib function that returns an aggregate parameter can detach initialized storage ownership from the returned value, causing false leaks or missed invalid old-field reuse.

## 修正方針

Treat a call argument as transferable when either the exact place or one of its descendant owner projections is live. Apply this to direct/known owner return summaries and unknown callback fallback, so aggregate parameter returns move descendant obligations with the value.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 aggregate parameter return descendant owner transfer 対応

`OwnerTable::has_transferable_owner` を追加し、exact place が live owner である場合だけでなく、source 配下に live owner projection がある場合も owner transfer を開始するようにした。

direct call / known function value の owner return summary 適用と unknown callback fallback は、この判定を使って `transfer_owner` を呼ぶ。`transfer_owner` 自体は source prefix 配下の descendant owner を target prefix 配下へ移すため、aggregate parameter return でも field owner obligation が caller output へ移る。

`nepl-core/tests/resource_ir.rs` に、owner 入り wrapper を helper `id_wrapper` に渡して戻り値 field を dealloc する正常系を追加した。
