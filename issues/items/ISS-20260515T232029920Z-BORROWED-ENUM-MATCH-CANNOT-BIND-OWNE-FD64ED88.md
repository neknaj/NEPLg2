---
id: ISS-20260515T232029920Z-BORROWED-ENUM-MATCH-CANNOT-BIND-OWNE-FD64ED88
title: "Borrowed enum match cannot bind owner payloads without moving them"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/typecheck/match_check.rs, nepl-core/src/resource/lower.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs"
---

# ISS-20260515T232029920Z-BORROWED-ENUM-MATCH-CANNOT-BIND-OWNE-FD64ED88: Borrowed enum match cannot bind owner payloads without moving them

## 概要

match currently accepts enum values but not references to enum values. That prevents source code from observing an enum payload through &Enum without moving or copying the payload. Owner-carrying enums such as VecStorage<T>::Owned(RegionToken<T>) therefore cannot be used as the structural invariant for storage ownership while still supporting borrowed observers.

## 対象

- `nepl-core/src/typecheck/match_check.rs, nepl-core/src/resource/lower.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs`

## 根拠

- `nepl-core/src/typecheck/match_check.rs` は `TypeKind::Enum` / enum `Apply` だけを enum match として扱い、`TypeKind::Reference` を unwrap していなかった。
- HIR の enum match payload binding は payload value 型だけを local に入れていたため、`&Enum` を許可しても payload owner を参照として束縛する型経路がなかった。
- Resource IR の `ResourceOp::Match` は payload binding を owner transfer として扱うため、owner payload を `&Payload` として読む設計には、match arm 冒頭に `Borrow` を seed し、既存の match payload move/copy 経路から外す必要があった。
- wasm/LLVM backend は enum payload binding で payload bytes/value を local へコピーしており、参照 binding では payload address を local に束縛する分岐が必要だった。

## 問題

match currently accepts enum values but not references to enum values. That prevents source code from observing an enum payload through &Enum without moving or copying the payload. Owner-carrying enums such as VecStorage<T>::Owned(RegionToken<T>) therefore cannot be used as the structural invariant for storage ownership while still supporting borrowed observers.

## 影響

Stdlib designs that tie ownership to an enum variant are forced either to split tag and owner again, or to rely on ad hoc helper assumptions. This blocks the Vec storage/RegionToken owner coupling issue and weakens static memory-safety proof from source.

## 修正方針

Add first-class borrowed enum match semantics: typecheck &Enum scrutinees as enum matches, bind payloads as &Payload or &mut Payload according to the scrutinee reference, lower Resource IR match scrutinees to the referenced enum place, seed borrowed payload bindings with Borrow operations instead of owner transfer, and teach wasm/LLVM codegen to bind payload addresses for reference payload bindings.

## 検証

Fixed.

- `&Enum` scrutinee を enum match として typecheck し、payload bind local は scrutinee の mutability に従って `&Payload` / `&mut Payload` になる。
- Resource IR lowering は borrowed enum match の scrutinee を deref 済み enum place として扱い、borrowed payload bind local には arm 冒頭で `ResourceOp::Borrow` を挿入する。
- 既存の match payload owner transfer / borrow token copy 経路は borrowed payload bind を処理しないため、owner payload を参照観測しただけで移動済みにしない。
- wasm/LLVM backend は borrowed payload bind に payload address を束縛し、owned payload bind のコピー・ロード経路とは分けた。

検証:

- `tests/compiler/reference_codegen.n.md` に、scalar payload の borrowed enum match と、`RegionToken` owner payload を borrowed match で観測したあと元 enum を消費して `dealloc_region` できる回帰テストを追加した。
- `cargo check -p nepl-core`: passed.
- `trunk build`: passed.
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --no-tree -o tmp/agent1-borrowed-enum-match-reference-codegen.json -j 1 --dist web/dist --assert-io`: 5/5 passed.
- `node nodesrc/test_resource_checker_responsibility.js`: passed.
- `node nodesrc/issues.js check --dir issues`: passed.
