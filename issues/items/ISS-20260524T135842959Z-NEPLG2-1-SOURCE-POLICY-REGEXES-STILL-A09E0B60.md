---
id: ISS-20260524T135842959Z-NEPLG2-1-SOURCE-POLICY-REGEXES-STILL-A09E0B60
title: "NEPLg2.1 source policy regexes still expect old type syntax"
area: tests
status: open
resolved: false
priority: P1
type: maintenance
created: 2026-05-24
updated: 2026-05-24
target: "nodesrc/test_stdlib_*.js; nodesrc/source_policy/**"
---

# ISS-20260524T135842959Z-NEPLG2-1-SOURCE-POLICY-REGEXES-STILL-A09E0B60: NEPLg2.1 source policy regexes still expect old type syntax

## 概要

NEPLg2.1 syntax migration changed type annotations and function signatures to %/prefix form, but many source policy regexes still expect NEPLg2.0 angle-bracket signatures.

## 対象

- `nodesrc/test_stdlib_*.js; nodesrc/source_policy/**`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` が 90 件の stale policy failure を報告した。
- 失敗例は `let text <str>`、`fn ... <(...)->...>`、`struct ... field <Type>` などの NEPLg2.0 記法を期待しており、実 source は `let text %str`、`%fn ...`、`field %Type` へ移行済みである。
- builder owner boundary 系は `nodesrc/source_policy/stdlib_builder_owner.js` と `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` の一部を NEPLg2.1 記法へ更新して pass へ戻したが、同種の regex が他の source policy に残っている。

## 問題

NEPLg2.1 syntax migration changed type annotations and function signatures to %/prefix form, but many source policy regexes still expect NEPLg2.0 angle-bracket signatures.

## 影響

run_source_policy_regressions --warn-only reports many stale policy failures, reducing static inspection signal during the migration.

## 修正方針

Migrate source policy regexes to NEPLg2.1 syntax or introduce explicit syntax-aware helpers, without weakening owner-boundary and API-boundary assertions.

## 検証

node nodesrc/run_source_policy_regressions.js without stale NEPLg2.0 syntax failures
