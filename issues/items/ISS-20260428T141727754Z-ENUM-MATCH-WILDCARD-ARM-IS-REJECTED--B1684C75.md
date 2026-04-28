---
id: ISS-20260428T141727754Z-ENUM-MATCH-WILDCARD-ARM-IS-REJECTED--B1684C75
title: "enum match wildcard arm is rejected as unsupported pattern"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/typecheck/match_check.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, tests/compiler/match_enum_wildcard_patterns.n.md"
---

# ISS-20260428T141727754Z-ENUM-MATCH-WILDCARD-ARM-IS-REJECTED--B1684C75: enum match wildcard arm is rejected as unsupported pattern

## 概要

enum scrutinee の match に _ wildcard arm を書くと D3097 unsupported match pattern for enum scrutinee と D3009 non-exhaustive match になり、仕様上の default arm として使えない。self-host import spec 実装で non-import module item を _ でまとめようとして再現した。

## 対象

- `nepl-core/src/typecheck/match_check.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, tests/compiler/match_enum_wildcard_patterns.n.md`

## 根拠

- `tests/compiler/match_literal_patterns.n.md` では i32 / char などの scalar scrutinee に対する `_` wildcard arm が回帰テスト済みで、言語機能として wildcard pattern は導入済みである。
- `stdlib/neplg2/core/module/import_spec.nepl` の `match item.kind: ... _:` で、enum scrutinee に対する wildcard arm が D3097 `unsupported match pattern for enum scrutinee` になり、同時に D3009 `non-exhaustive match` も出ることを確認した。
- `doc/neplg3/spec/patterns.md` は `_` を書いた時点で網羅されると定義しており、enum variant pattern も従来どおり利用できるものとして扱っている。

## 問題

enum scrutinee の match に _ wildcard arm を書くと D3097 unsupported match pattern for enum scrutinee と D3009 non-exhaustive match になり、仕様上の default arm として使えない。self-host import spec 実装で non-import module item を _ でまとめようとして再現した。

## 影響

enum variant が多い型で default 分岐を書けず、stdlib/self-host 側が全 variant を列挙する不自然なコードになる。variant 追加時の追従も局所的な default 処理として書けないため、match wildcard の仕様と実装がずれる。

## 修正方針

enum match の typecheck で Wildcard pattern を enum scrutinee の網羅 arm として扱い、wildcard 非末尾・重複 wildcard・到達不能 arm の既存検査と整合させる。wasm/LLVM lowering は enum tag dispatch の default 分岐として wildcard arm へ落とす。

## 検証

tests/compiler/match_enum_wildcard_patterns.n.md に enum wildcard が default armとして動作するケース、wildcard 非末尾が D3098 になるケース、重複 wildcard が診断されるケースを追加する。
