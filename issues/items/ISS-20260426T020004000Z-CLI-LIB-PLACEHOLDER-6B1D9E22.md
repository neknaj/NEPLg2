---
id: ISS-20260426T020004000Z-CLI-LIB-PLACEHOLDER-6B1D9E22
title: "nepl-cli --lib is accepted but only prints a placeholder warning"
area: cli
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "nepl-cli/src/main.rs, nepl-cli/tests/cli_output.rs"
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

## 対応結果

短期契約として `--lib` は未実装を明示する failure に固定した。
`nepl-cli/src/main.rs` では `--lib` を parse 後すぐに検出し、compile pipeline や output 書き込みへ進む前に `--lib is not supported yet: library artifact contract is not implemented` で non-zero exit する。
CLI help も「currently unsupported; exits with an error」として、実装済み library compile と誤認しない表現にした。

`nepl-cli/tests/cli_output.rs` に `lib_mode_fails_until_artifact_contract_exists` を追加し、stderr の unsupported diagnostic、placeholder warning の非出力、wasm artifact を書かないことを固定した。
実際の library artifact 契約は、entry requirement / export set / output naming を決める別設計で扱う。

## 検証

- `cargo fmt --all --check`
- `cargo test -p nepl-cli --test cli_output lib_mode_fails_until_artifact_contract_exists` (`1 passed`)
- `cargo test -p nepl-cli --test cli_output` (`13 passed`)
- `cargo test -p nepl-cli` (`9 unit passed`, `13 cli_output passed`, `2 deploy_script ignored`)
- `node nodesrc/issues.js check`
