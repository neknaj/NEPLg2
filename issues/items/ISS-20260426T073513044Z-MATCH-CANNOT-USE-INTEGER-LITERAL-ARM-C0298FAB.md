---
id: ISS-20260426T073513044Z-MATCH-CANNOT-USE-INTEGER-LITERAL-ARM-C0298FAB
title: "match cannot use integer literal arms for finite value dispatch"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/parser.rs
---

# ISS-20260426T073513044Z-MATCH-CANNOT-USE-INTEGER-LITERAL-ARM-C0298FAB: match cannot use integer literal arms for finite value dispatch

## 概要

match ch: の arm に 92: や 34: のような整数 literal pattern を書くと parser が expected identifier で失敗する。stdlib/alloc/encoding/json.nepl の json_escape で JSON escape byte を match で表現しようとした際に再現した。

## 対象

- `nepl-core/src/parser.rs`

## 根拠

- `nepl-core/src/parser.rs` の `parse_match_arms` は arm 見出しを identifier / path separator の列として読むため、整数 literal arm を構文として受け付けない。
- `nepl-core/src/typecheck.rs` の `check_match_expr` は enum variant 集合（bool は `true` / `false` 相当の疑似 variant）を対象に duplicate / unknown variant / non-exhaustive を検査している。
- `_` は payload bind 名として使われることはあるが、現在の match arm 見出しでは wildcard / default pattern として扱われない。

## 問題

match ch: の arm に 92: や 34: のような整数 literal pattern を書くと parser が expected identifier で失敗する。stdlib/alloc/encoding/json.nepl の json_escape で JSON escape byte を match で表現しようとした際に再現した。

## 影響

固定 byte、token kind、small integer code の対応表を match で直接書けず、stdlib 側が enum への分類関数や if 連鎖を挟む必要がある。match で表現すべき有限分岐が不自然な if / else if として残りやすく、compiler bug 回避の痕跡を stdlib に温存する原因になる。

## 修正方針

parser / AST / typecheck / lowering の pattern 対応範囲を確認し、整数 literal arm を match pattern として受理する。
enum / bool match では既存の duplicate / unknown / non-exhaustive 検査を維持し、literal match では閉じた集合ではない型を扱うため wildcard / default arm の仕様を明確化する。
`_` を単なる payload bind 名として扱う経路と、arm 見出しの wildcard pattern として扱う経路を混同しないように AST 上で分離する。

## 検証

compiler regression test に `match i32` の整数 literal arm、wildcard / default arm、duplicate literal、wildcard なしの non-exhaustive 相当の診断方針を追加する。
既存 enum match の duplicate / unknown / non-exhaustive 回帰テストも維持し、stdlib の `json_escape` など byte dispatch を literal match で書けることを確認する。
