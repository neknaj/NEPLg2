---
id: ISS-20260524T085928137Z-README-AND-DOCS-MUST-DISTINGUISH-NEP-20719BBC
title: "README and docs must distinguish NEPLg2.1 from unstable NEPLg3"
area: doc
status: fixed
resolved: true
priority: P0
type: doc
created: 2026-05-24
updated: 2026-05-25
target: "README.md, doc/README.md, doc/neplg2/**, doc/neplg3/**"
---

# ISS-20260524T085928137Z-README-AND-DOCS-MUST-DISTINGUISH-NEP-20719BBC: README and docs must distinguish NEPLg2.1 from unstable NEPLg3

## 概要

Repository docs currently present parts of NEPLg3 as the positive current direction, but the active migration is NEPLg2 to NEPLg2.1 and NEPLg3 remains unstarted and unstable.

## 対象

- `README.md, doc/README.md, doc/neplg2/**, doc/neplg3/**`

## 根拠

- ユーザー指示により、現在の対象は NEPLg2 であり、この変更で NEPLg2.1 へ切り替える。NEPLg3 はまだ仕様すら確定していない未着手段階である。
- `README.md` と `doc/README.md` は NEPLg3 や Zenn #1 / #2 を現在の正方向のように扱っており、NEPLg2.1 移行と混同しやすい。
- `doc/neplg3/README.md` と `doc/neplg3/spec/index.md` は、NEPLg3 文書を正仕様のように説明している。
- `doc/migration/index.md` は `stdlib/` / `tests/` / `tutorials/` を凍結して `*-g3/` を並行作成する計画を記しているが、NEPLg2.1 では既存ディレクトリ内で移行する。

## 問題

Repository docs currently present parts of NEPLg3 as the positive current direction, but the active migration is NEPLg2 to NEPLg2.1 and NEPLg3 remains unstarted and unstable.

## 影響

Agents and readers can treat NEPLg3 draft documents as authoritative for the current implementation, causing wrong syntax, wrong selfhost planning, and wrong migration work.

## 修正方針

Document NEPLg2.1 as the active current syntax migration, mark NEPLg3 docs as unstable reference material, and update README/doc indexes accordingly.

### 2026-05-24 checkpoint

- `README.md` に NEPLg2.1 が現在の表層構文移行対象であることを追記した。
- `doc/neplg2/neplg21_syntax_migration_plan.md` を追加した。
- `doc/README.md` と `doc/neplg2/README.md` は NEPLg2.1 を優先し、NEPLg3 を参考扱いに更新した。
- `doc/neplg3/README.md`、`doc/neplg3/spec/index.md`、`doc/migration/index.md`、`doc/compare/index.md` は draft / 参考資料として明記した。

### 2026-05-25 final checkpoint

- `README.md` は、現在の主作業が NEPLg2 から NEPLg2.1 への表層構文移行であり、NEPLg3 が未着手・未確定の将来設計であることを明示している。
- `doc/neplg2/README.md` と `doc/neplg2/neplg21_syntax_migration_plan.md` は、NEPLg2.1 を現行 `nepl-core/` / `stdlib/` / `tests/` の同一ライン移行として説明している。
- `doc/neplg3/README.md`、`doc/neplg3/spec/index.md`、`doc/migration/index.md`、`doc/compare/index.md` は、NEPLg3 文書を現在の正仕様ではなく draft / 参考資料として扱うよう更新済みである。
- 残る NEPLg2.1 実装・corpus・kind resolver の課題は、専用 issue で追跡しており、この doc issue の範囲から分離した。

## 検証

- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/neplg21_syntax_migrate.js --check`
- `rg -n "NEPLg2.1|NEPLg3" README.md doc/README.md doc/neplg2 doc/neplg3 doc/migration doc/compare`
