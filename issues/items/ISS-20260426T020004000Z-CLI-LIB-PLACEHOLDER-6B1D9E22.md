---
id: ISS-20260426T020004000Z-CLI-LIB-PLACEHOLDER-6B1D9E22
title: "nepl-cli --lib is accepted but only prints a placeholder warning"
area: cli
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: nepl-cli/src/main.rs
source: doc/neplg2/pre_selfhost_audit_20260426.md
---

# ISS-20260426T020004000Z-CLI-LIB-PLACEHOLDER-6B1D9E22: nepl-cli --lib is accepted but only prints a placeholder warning

## 概要

`nepl-cli` は `--lib` を option として受け取るが、compile pipeline では処理せず placeholder warning を stderr に出すだけである。

## 根拠

- `nepl-cli/src/main.rs:571` は `--lib is acknowledged but not yet implemented in the placeholder pipeline` を出力する。
- self-host compiler の配布形態では、compiler core を library として build し、WASI CLI から呼ぶ構成を予定している。

## 問題

option が成功パスで受理されるため、利用者や self-host parity runner が library artifact を生成したと誤認しやすい。
未実装なら明確な unsupported diagnostic と non-zero exit にするか、実際に library mode の artifact 契約を実装する必要がある。

## 影響

self-host の core / CLI 分離を検証する段階で、Rust 参照 CLI と self-host CLI の artifact set が揃わない。
特に `nepl-core` 相当の no-WASI WASM と `nepl-cli` 相当の WASI wrapper を分ける計画に影響する。

## 修正方針

`--lib` の仕様を `doc/neplg2/self_host_plan.md` と CLI help に合わせて決める。
短期的には unsupported を structured diagnostic と exit failure にする。
実装する場合は、entry requirement、export set、stdlib prelude、output file naming、test fixture を同じ commit で固定する。

## 検証

- `cargo test -p nepl-cli --test cli`
- `nepl-cli --lib` の成功 / 失敗契約を JSON または snapshot fixture で確認する。
