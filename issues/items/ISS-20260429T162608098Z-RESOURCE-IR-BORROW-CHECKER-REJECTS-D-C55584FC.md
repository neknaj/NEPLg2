---
id: ISS-20260429T162608098Z-RESOURCE-IR-BORROW-CHECKER-REJECTS-D-C55584FC
title: "Resource IR borrow checker rejects drop overwrite fixture"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource/borrow_check.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/drop_overwrite.rs, tests/compiler/drop_overwrite.n.md"
---

# ISS-20260429T162608098Z-RESOURCE-IR-BORROW-CHECKER-REJECTS-D-C55584FC: Resource IR borrow checker rejects drop overwrite fixture

## 概要

On current main after MemPtr explicit Clone merge, node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/memptr-explicit-clone-drop-overwrite-after-merge.json -j 1 --dist web/dist reports total=1 failed=1 with resource.borrow.assign_during_shared. The conflict is Assign on local g while a Shared borrow is counted as active. This is outside the MemPtr explicit Clone path and appears to be a Resource IR borrow-lifetime regression.

## 対象

- `nepl-core/src/resource/borrow_check.rs`
- `nepl-core/tests/resource_ir.rs`
- `nepl-core/tests/drop_overwrite.rs`
- `tests/compiler/drop_overwrite.n.md`

## 根拠

- `tests/compiler/drop_overwrite.n.md` は `set g Guard 1` の上書き前に旧 `g` を `Drop::drop(&g)` で破棄する。
- Resource IR lowering では `&g` が `ResourceOp::Borrow` になり、その token が `Drop::drop` の call 引数に渡される。
- `Drop::drop` は borrow token を返さないにもかかわらず、borrow checker は call 後も引数 token を保持し続けていた。
- そのため後続の `Assign` が `g` に対する active shared borrow と誤判定され、`resource.borrow.assign_during_shared` で valid な drop overwrite fixture を拒否していた。

## 問題

On current main after MemPtr explicit Clone merge, node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/memptr-explicit-clone-drop-overwrite-after-merge.json -j 1 --dist web/dist reports total=1 failed=1 with resource.borrow.assign_during_shared. The conflict is Assign on local g while a Shared borrow is counted as active. This is outside the MemPtr explicit Clone path and appears to be a Resource IR borrow-lifetime regression.

## 影響

The drop overwrite fixture is a focused regression guard for overwrite/drop behavior. Rejecting it means the Resource IR borrow checker may be retaining shared borrows too long, which can block valid code and undermine confidence in the static memory safety gate.

## 修正方針

Trace Resource IR borrow activation and release around addr-of/deref and overwrite in the drop_overwrite fixture. Keep assign_during_shared strict for real aliasing, but ensure borrow scopes end at the correct expression boundary before a later assignment is checked.

## 修正結果

- `ResourceOp::Call` / `ResourceOp::IndirectCall` の borrow token 伝播は、まず callee の返り値 summary に従って返却 token を output へ移す。
- その後で call 引数に残る borrow token を解放し、返り値に含まれない一時 borrow が式境界を越えて生存しないようにした。
- 返り値として返された borrow token は output 側に残るため、`borrow_id(&x)` のように token を返す関数の後で `x` へ assign すると従来どおり borrow conflict になる。
- direct call と indirect call の両方に、非返却 borrow token 解放の Resource IR 回帰テストを追加した。
- token を返す direct call の回帰テストも追加し、過剰解放でメモリ安全検査が抜けないことを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check_releases_non_returned_call_argument_borrow_token -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check_keeps_returned_call_argument_borrow_token_live -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check_releases_non_returned_indirect_call_argument_borrow_token -- --nocapture`: passed
- `cargo test -p nepl-core --test drop_overwrite -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/drop-overwrite-borrow-regression-fixed.json -j 1 --dist web/dist`: total=1, passed=1
