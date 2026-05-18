---
id: ISS-20260518T165601592Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-7475D872
title: "resource checker responsibility policy omits host memory modules"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: nodesrc/test_resource_checker_responsibility.js
---

# ISS-20260518T165601592Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-7475D872: resource checker responsibility policy omits host memory modules

## 概要

Stage 6 Resource IR の fd_write payload extent proof で host_memory_address.rs と owner_host_memory_summary.rs を追加したが、resource checker responsibility source policy の監視対象に入っていない。run_source_policy_regressions --warn-only は host_memory_address.rs must be monitored と警告し、以後の責務分割 regression を見落とす。

## 対象

- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` が `host_memory_address.rs must be monitored by resource responsibility line limits` を報告した。
- `host_memory_address.rs` / `owner_host_memory_summary.rs` / payload extent 分離 module が `nepl-core/src/resource/mod.rs` に存在する一方で、`nodesrc/test_resource_checker_responsibility.js` の required module list と line-limit map に入っていなかった。

## 問題

Stage 6 Resource IR の fd_write payload extent proof で host_memory_address.rs と owner_host_memory_summary.rs を追加したが、resource checker responsibility source policy の監視対象に入っていない。run_source_policy_regressions --warn-only は host_memory_address.rs must be monitored と警告し、以後の責務分割 regression を見落とす。

## 影響

Resource IR の host memory proof helper が行数上限と module 宣言監視から外れ、静的検査本体の肥大化や責務再集中を source policy で検出できない。

## 修正方針

host_memory_address.rs と owner_host_memory_summary.rs を required module list、module declaration list、line-limit map に追加し、source policy 自体が新規 Resource IR module を網羅的に監視する状態へ戻す。

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only

## 2026-05-18 Agent 1 修正

Stage 6 の host memory proof module を resource checker responsibility policy の required module list、module declaration list、line-limit map に追加した。

単に監視対象を足しただけでなく、policy を有効化したことで露出した責務肥大も整理した。`owner_external_io.rs` から iovec payload / payload extent proof を `owner_external_io_payload.rs` へ分離し、`owner_host_memory_span.rs` から iovec descriptor extent proof を `owner_host_iov_descriptor.rs` へ分離した。また raw owner alias branch merge を `owner_summary_raw_use_branch.rs` へ分離し、walk 本体と分岐合流の責務を切り分けた。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check`
- `git diff --check`
