---
id: ISS-20260520T014008212Z-SELF-HOST-OWNER-OBLIGATION-IS-NOT-RE-EB1CB46B
title: "self-host owner obligation is not represented in generic proof solver"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/resource/owner.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_owner_proof.n.md"
---

# ISS-20260520T014008212Z-SELF-HOST-OWNER-OBLIGATION-IS-NOT-RE-EB1CB46B: self-host owner obligation is not represented in generic proof solver

## 概要

The self-host proof architecture models Resource cell state, borrow access, and lifetime outlives, but it still has no typed model for free-obligation ownership. Without an owner state/event fact, future self-host Resource IR lowering can fall back to checker-local rules or MemPtr-shaped owner assumptions instead of proving storage ownership through the generic solver.

## 対象

- `stdlib/neplg2/core/resource/owner.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_owner_proof.n.md`

## 根拠

- `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` という分離方針では、free obligation owner の acquire / move / release / non-owning view を compiler が別責務として証明できる必要がある。
- 修正前の self-host proof architecture は Resource cell、borrow access、lifetime outlives を扱えていたが、owner obligation を表す typed fact / obligation / evidence / refutation が存在しなかった。
- この欠落を放置すると、後続の Resource IR lowering / checker が `MemPtr` を owner として扱う古い設計へ戻るか、checker-local の ad hoc owner state machine を持つことになる。

## 問題

The self-host proof architecture models Resource cell state, borrow access, and lifetime outlives, but it still has no typed model for free-obligation ownership. Without an owner state/event fact, future self-host Resource IR lowering can fall back to checker-local rules or MemPtr-shaped owner assumptions instead of proving storage ownership through the generic solver.

## 影響

This leaves the MemPtr=non-owning pointer and OwnedRegion/Storage=free obligation owner split incomplete on the self-host side, weakening the Stage 6 memory-safety architecture and making it easier to reintroduce ad hoc owner checks.

## 修正方針

Add a typed owner obligation model under core/resource, add Owner domain facts/obligations/evidence/refutations to the generic proof solver, keep public helpers intentional, and add regression tests for acquire, move, release, double release, move-after-release, and invalid storage id cases.

## 検証

Run the selfhost proof entry contract, focused owner model/proof doctests, existing proof doctests, issues check, and diff checks.

## 対応内容

- `stdlib/neplg2/core/resource/owner.nepl` を追加し、`SelfhostOwnerStorageId`、`SelfhostOwnerState`、`SelfhostOwnerEventKind` を typed model として定義した。
- `core/proof` に `SelfhostProofDomain::Owner`、`SelfhostOwnerEventFact`、`SelfhostProofObligation::OwnerTransition`、`SelfhostProofEvidence::OwnerTransition`、`SelfhostProofRefutation::OwnerTransitionInvalid` を追加し、owner transition を generic proof solver に接続した。
- `Acquire`、`MoveOut`、`Release`、`BorrowView` の遷移と、invalid storage id、storage id mismatch、double release、moved/released 後の不正操作を `SelfhostOwnerTransitionError` enum として保持した。
- `tests/stdlib/neplg2_owner_proof.n.md` と `nodesrc/test_selfhost_proof_entry_contract.js` で、owner proof が typed enum / exhaustive match / public solver surface から外れないことを固定した。

## 完了条件

- self-host checker / Resource IR lowering は、owner obligation を個別 checker-local proof で扱わず、source-derived owner event fact producer として `core/proof` の Owner domain に接続する。
- `MemPtr` は owner ではなく non-owning pointer view として扱い、free obligation owner は owner state / storage id 側で証明する。
