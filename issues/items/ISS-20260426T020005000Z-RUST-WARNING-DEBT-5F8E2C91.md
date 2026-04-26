---
id: ISS-20260426T020005000Z-RUST-WARNING-DEBT-5F8E2C91
title: "Rust compiler crates emit warning debt that hides audit signal"
area: core
status: open
resolved: false
priority: P3
type: maintenance
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src, nepl-cli/src"
source: doc/neplg2/pre_selfhost_audit_20260426.md
---

# ISS-20260426T020005000Z-RUST-WARNING-DEBT-5F8E2C91: Rust compiler crates emit warning debt that hides audit signal

## 概要

`cargo check -p nepl-core -p nepl-cli` は成功するが、`nepl-core` で 66 件、`nepl-cli` で 1 件の warning を出す。
unused import / unused variable / dead code / crate-level attribute / unreachable pattern が混在している。

## 根拠

- `cargo check -p nepl-core -p nepl-cli` は `nepl-core` 66 warnings、`nepl-cli` 1 warning。
- 特に `nepl-core/src/types.rs:1370` の unreachable pattern は型統合ロジックの重複分岐を示している。
- `#![no_std]` crate-level attribute が複数 module に置かれており、no_std 境界の意図が読み取りにくい。

## 問題

警告が多い状態では、self-host 前提の review で新しい warning が追加されても差分が見えにくい。
また、unused code が実装途中の残骸なのか、今後必要な設計上の hook なのか判断しにくい。

## 影響

Rust 側参照実装を self-host の正とする期間に、監査と regression 判断のノイズが増える。
重大度は P3 だが、self-host の長期 branch 運用では warning の増減を checkpoint ごとに追える状態が望ましい。

## 修正方針

自動修正できる unused import / unused mut は `cargo fix` 相当の内容を個別に確認してから整理する。
dead code は単純削除せず、現在の計画で必要な hook かどうかを判定して、残す場合は用途を comment / issue に移す。
unreachable pattern は型統合の期待仕様を確認し、重複分岐を片付ける。

## 検証

- `cargo check -p nepl-core -p nepl-cli`
- `cargo test -p nepl-core -p nepl-cli`
