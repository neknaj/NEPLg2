---
id: ISS-20260428T110657405Z-RESOURCE-BORROW-CHECKER-ALLOWS-BORRO-EC3BEA97
title: "Resource borrow checker allows borrow tokens to escape through returns"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T110657405Z-RESOURCE-BORROW-CHECKER-ALLOWS-BORRO-EC3BEA97: Resource borrow checker allows borrow tokens to escape through returns

## 概要

ResourceBorrowCheckEngine checks ResourceOp borrow conflicts but ignores ResourceTerminator::Return. A function can return an active borrow token, leaving a borrow lifetime tied to a local/source place to escape the function boundary in the Resource IR shadow checker.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は borrow / lifetime を Resource IR 上で検査し、borrow token の release と escape を形だけでなく実際に扱う方針である。
- `nepl-core/src/resource/check.rs` の `ResourceBorrowCheckEngine::check_block` は `block.ops` だけを検査し、`ResourceTerminator::Return` を見ていなかった。
- `BorrowTable` は active borrow token と source place の binding を保持しているため、return value が active token かどうかを判定できる状態は既に存在していた。
- active borrow token を戻り値として外へ出すと、その token の lifetime が source place の scope を越える可能性があり、Resource IR enforcement へ進む前に診断として固定する必要がある。

## 問題

ResourceBorrowCheckEngine checks ResourceOp borrow conflicts but ignores ResourceTerminator::Return. A function can return an active borrow token, leaving a borrow lifetime tied to a local/source place to escape the function boundary in the Resource IR shadow checker.

## 影響

Borrow/lifetime diagnostics remain incomplete at function boundaries. If Resource IR checks are later enforced without this case, references or borrow capabilities can outlive their source, weakening memory safety for self-host lowering and resource analysis.

## 修正方針

Inspect return terminators in ResourceBorrowCheckEngine. If the returned place is an active borrow token, report a BorrowConflict with ReturnValue and the active source borrow state. Preserve normal return of non-token values and released tokens.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; node nodesrc/issues.js check

## 2026-04-28 Stage 4 borrow return escape 対応

`ResourceBorrowCheckEngine::check_block` が return terminator を検査するようにした。戻り値 place が `BorrowTable` 上の active borrow token で、source が `Shared` または `Unique` の active borrow 状態を持つ場合、`ResourceBorrowOperation::ReturnValue` の `BorrowConflict` を出す。

released token は `BorrowTable` の binding から取り除かれるため、borrow release 後に通常値を返す経路は従来通り許可される。`nepl-core/tests/resource_ir.rs` に active shared borrow token return の回帰と、release 後 return の正常系を追加した。
