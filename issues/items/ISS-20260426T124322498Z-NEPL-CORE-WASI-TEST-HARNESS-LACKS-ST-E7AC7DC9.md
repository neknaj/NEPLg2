---
id: ISS-20260426T124322498Z-NEPL-CORE-WASI-TEST-HARNESS-LACKS-ST-E7AC7DC9
title: "nepl-core WASI test harness lacks std/fs directory imports after fs dir API"
area: core
status: verified
resolved: true
priority: P2
type: test
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/tests/harness.rs
---

# ISS-20260426T124322498Z-NEPL-CORE-WASI-TEST-HARNESS-LACKS-ST-E7AC7DC9: nepl-core WASI test harness lacks std/fs directory imports after fs dir API

## 概要

After syncing origin/main at b801a12, cargo test -p nepl-core -p nepl-cli fails in nepl-core tests/kp.rs because the shared WASI harness does not register path_filestat_get imported by std/fs.

## 対象

- `nepl-core/tests/harness.rs`

## 根拠

- `origin/main` の `b801a12` 取り込み後に `cargo test -p nepl-core -p nepl-cli` を実行すると、`nepl-core/tests/kp.rs` の WASI 実行系テストが instantiate 時点で `wasi_snapshot_preview1::path_filestat_get` の未登録により失敗した。
- stdlib 側に `std/fs` directory API が追加されたことで、directory API を直接使わない core integration test でも module import として `path_filestat_get` / `fd_readdir` が現れるようになった。

## 問題

After syncing origin/main at b801a12, cargo test -p nepl-core -p nepl-cli fails in nepl-core tests/kp.rs because the shared WASI harness does not register path_filestat_get imported by std/fs.

## 影響

Core integration tests that import stdlib can fail at module instantiation even when they do not exercise directory APIs, blocking issue verification after main sync.

## 修正方針

Register minimal path_filestat_get and fd_readdir stubs in the nepl-core WASI test harnesses so modules importing std/fs instantiate; leave full directory behavior covered by nepl-cli raw WASI tests.

## 解決内容

- `nepl-core/tests/harness.rs` に WASI directory import stub の登録 helper を追加した。
- `run_main_wasi_i32`、`run_main_capture_stdout`、`run_main_capture_stdout_with_stdin` の各 WASI harness で同じ stub を登録するようにした。
- `path_filestat_get` は harness の仮想入力で使う `test.nepl` の file stat を返し、それ以外は `NOENT` を返す。
- `fd_readdir` は core harness では directory traversal を実行対象にせず、instantiate を妨げない最小 stub として `BADF` を返す。directory traversal の実挙動は `nepl-cli` の raw WASI integration test で固定する。

## 検証

- `cargo test -p nepl-core --test kp`: pass
- `cargo test -p nepl-core -p nepl-cli`: pass
- `cargo check --workspace`: pass
- `cargo fmt --all --check`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-warning-debt-after-rebase.json`: `13/13 passed`
