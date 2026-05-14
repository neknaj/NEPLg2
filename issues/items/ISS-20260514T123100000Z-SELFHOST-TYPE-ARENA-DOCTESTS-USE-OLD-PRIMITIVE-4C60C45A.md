---
id: ISS-20260514T123100000Z-SELFHOST-TYPE-ARENA-DOCTESTS-USE-OLD-PRIMITIVE-4C60C45A
title: "selfhost type arena doctests use old primitive kind API"
area: selfhost
status: open
resolved: false
priority: P2
type: test
created: 2026-05-14
updated: 2026-05-14
target: "tests/stdlib/neplg2_type_arena.n.md, stdlib/neplg2/core/ty/ty.nepl"
---

# ISS-20260514T123100000Z-SELFHOST-TYPE-ARENA-DOCTESTS-USE-OLD-PRIMITIVE-4C60C45A: selfhost type arena doctests use old primitive kind API

## 概要

`SelfhostTypeRecord` が `Primitive <SelfhostPrimitiveTypeKind>` / `Function <SelfhostFunctionTypeRecord>` に分離された後も、`tests/stdlib/neplg2_type_arena.n.md` は `selfhost_type_arena_add_primitive` に旧 `SelfhostTypeKind::*` を渡している。

## 対象

- `tests/stdlib/neplg2_type_arena.n.md`
- `stdlib/neplg2/core/ty/ty.nepl`

## 根拠

- current `stdlib/neplg2/core/ty/ty.nepl` の `selfhost_type_arena_add_primitive` は `(SelfhostTypeArena, SelfhostPrimitiveTypeKind)*>Result<SelfhostTypeArenaAlloc, StdErrorKind>` を要求する。
- `tests/stdlib/neplg2_type_arena.n.md` の focused run は全 5 doctest で `type.overload.no_match` を報告し、失敗位置はいずれも `selfhost_type_arena_add_primitive arena0 SelfhostTypeKind::...` である。
- enum payload 分離は静的検査の網羅性を高める正しい設計変更なので、compiler 側で旧 enum を暗黙変換して後方互換を残してはいけない。

## 問題

Type arena の regression doctest が現在の enum-first API に追従しておらず、selfhost type stage の CI signal が旧設計の呼び出しで失敗している。

## 影響

`tests/stdlib/neplg2_type_arena.n.md` が 0/5 で失敗し、TypeId / TypeArena / function type equality の回帰を検証できない。静的検査の大規模修正で type/lifetime/effect の土台を確認する際に、テストの stale failure が実装バグとの切り分けを妨げる。

## 修正方針

doctest を現在の `SelfhostPrimitiveTypeKind::*` API に更新し、期待値側は公開される `SelfhostTypeKind::*` と比較する。必要なら doc comment も primitive payload と public kind の責務分離が分かるように補強する。旧 `SelfhostTypeKind` を primitive input として受ける overload や adapter は追加しない。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/agent1-vec-push-owner-error-neplg2-type-arena-final.json -j 1 --dist web/dist --assert-io`: total=5, failed=5。top issue は `type.overload.no_match`。
