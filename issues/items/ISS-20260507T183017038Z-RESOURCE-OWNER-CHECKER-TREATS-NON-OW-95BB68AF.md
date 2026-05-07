---
id: ISS-20260507T183017038Z-RESOURCE-OWNER-CHECKER-TREATS-NON-OW-95BB68AF
title: "Resource owner checker treats non-owning callback arguments as owner candidates"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_return_view.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T183017038Z-RESOURCE-OWNER-CHECKER-TREATS-NON-OW-95BB68AF: Resource owner checker treats non-owning callback arguments as owner candidates

## 概要

Unknown indirect-call owner handling could treat a non-owning raw address view argument as an owner candidate or consume the owner behind its alias. When another same-type owner argument was present, the callback result could be classified as a definite owner even though the callback may return the non-owning view.

## 対象

- `nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_return_view.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 では、Resource IR 上で non-owning pointer view と free obligation owner を分離し、関数境界でも owner/provenance state を保つ必要がある。
- `MemPtr` は non-owning pointer、`OwnedRegion` / storage は free obligation owner、initialized/moved/drop state は Resource IR の cell state として分ける方針であり、unknown callback return でもこの区別を失ってはならない。
- `nepl-core/src/resource/owner_return.rs` の unknown indirect-call return handling は、same-type argument の transferable owner を先に選び、non-owning raw address view 候補を owner return より強い不確実性として扱っていなかった。
- unknown callback argument の owner consumption でも、non-owning raw view の alias 先を free obligation owner のように消費し得るため、valid borrowed pointer flow の false positive と owner provenance の false negative の両方につながる。

## 問題

Unknown indirect-call owner handling could treat a non-owning raw address view argument as an owner candidate or consume the owner behind its alias. When another same-type owner argument was present, the callback result could be classified as a definite owner even though the callback may return the non-owning view.

## 影響

Resource IR owner/provenance checking becomes dependent on callback precision and can either reject valid borrowed MemPtr flows or lose the distinction between MemPtr non-owning views and free-obligation owners at higher-order boundaries.

## 修正方針

Make non-owning raw address view facts authoritative over alias-based owner transfer for unknown callback returns. If any same-type non-owning candidate may be returned, propagate the non-owning view to the output instead of transferring a definite owner, and skip owner consumption for non-owning arguments.

## 2026-05-08 Stage 4 unknown callback non-owning view 対応

根本原因は、unknown indirect-call return が「同じ型の戻り値候補」を definite owner transfer として先に確定し、同じ型の non-owning view 引数が返り得る可能性を出力の owner state に反映していなかったことだった。

対応では `apply_unknown_indirect_call_return_owner` の判定順を変更し、unknown callback の same-type arguments に non-owning raw address view が含まれる場合は、出力を definite owner ではなく non-owning raw view として扱うようにした。出力側の raw alias / storage origin は clear し、free obligation owner の alias が non-owning view に混入しないようにしている。

また、unknown callback argument consumption では non-owning raw view 引数を owner consumption 対象から外した。これにより、callback が借用 pointer を読むだけの経路で元 owner の dealloc が妨げられず、逆に callback result が non-owning candidate を返し得る経路では `dealloc returned` が `OwnerState::NoFreeObligation` として拒否される。

`owner_return_view.rs` の global non-owning 判定は拡張しなかった。raw address load には「owner raw alias を保持したまま non-owning query view として扱う」既存仕様があり、そこを広げると aggregate raw cell owner transfer の正常系まで崩れるため、unknown callback 境界だけで保守的に処理する設計にした。

この修正は `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 の Resource IR owner/provenance 分離に含まれる。`MemPtr` / `OwnedRegion` / initialized cell の最終分離は親 issue `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` の残件として継続する。

## 検証

- `cargo fmt -p nepl-core`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir unknown_callback -- --nocapture`: 5 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_raw_address_load_as_nonowning_view -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 240 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-unknown-callback-non-owning-memory-safety.json -j 1 --dist web/dist`: 21 passed
- `node nodesrc/issues.js check`: passed
