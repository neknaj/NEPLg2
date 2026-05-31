---
id: ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C
title: "memo_call phase1 needs compiler-known primitive boundary"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/typecheck; nepl-core/src/resource; stdlib"
---

# ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C: memo_call phase1 needs compiler-known primitive boundary

## 概要

memo_call を pure public API として提供するには、現状の関数値 string identity と Pure/Impure 二値だけでは private cache の非 escape と高階関数境界を表現できない。

## 対象

- `nepl-core/src/typecheck; nepl-core/src/resource; stdlib`

## 根拠

- 未記入

## 問題

memo_call を pure public API として提供するには、現状の関数値 string identity と Pure/Impure 二値だけでは private cache の非 escape と高階関数境界を表現できない。

## 影響

primitive 境界を固定しないまま memo_call を通常ライブラリとして実装すると、impure/capturing/unresolved generic function や observable cache identity を pure と誤認する危険がある。

## 修正方針

Phase 1 は compiler-known primitive とし、memo_call @pure_named_func だけを受け入れる。typed function identity、MemoKey/MemoValue の保守的構造制約、private cache SourceCapability、sealed backend representation を依存条件として明示する。

## 検証

memo_call @pure_named_func が pure に通り、impure function、capturing function、generic unresolved function、reference/raw pointer key/value、cache stats/clear/ref exposure が拒否される regression matrix を追加する。
