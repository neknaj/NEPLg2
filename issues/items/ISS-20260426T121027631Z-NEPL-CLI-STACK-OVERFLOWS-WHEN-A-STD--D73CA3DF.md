---
id: ISS-20260426T121027631Z-NEPL-CLI-STACK-OVERFLOWS-WHEN-A-STD--D73CA3DF
title: "nepl-cli stack overflows when a std program calls std/fs fs_read_dir"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/compiler.rs, nepl-core/src/codegen_wasm.rs, nepl-cli/src/main.rs"
---

# ISS-20260426T121027631Z-NEPL-CLI-STACK-OVERFLOWS-WHEN-A-STD--D73CA3DF: nepl-cli stack overflows when a std program calls std/fs fs_read_dir

## 概要

`nepl-cli --run` で `std/fs` を import し、`fs_read_dir` を呼ぶと Windows 上で Rust stack overflow によりプロセスが abort していた。
raw WASI `fd_readdir` は通っていたため、host syscall shim ではなく compiler pipeline を CLI main thread の既定 stack で実行していることが原因だった。

## 対象

- `nepl-cli/src/main.rs`
- `nepl-cli/tests/cli_output.rs`

## 根拠

- `cargo test -p nepl-cli run_wasi_std_fs_read_dir_returns_stable_directory_entries -- --ignored --nocapture` が `status=Some(-1073741571)` と `thread 'main' has overflowed its stack` で失敗していた。
- `--verbose` の末尾は `std/fs` の typecheck 中で、`fs_read_dir_fd` 内の generic call 解決中だった。
- `std/fs` を import するだけの `nepl-cli --check` でも再現し、raw `fd_readdir` test は通るため WASI runtime ではなく compiler/typecheck 側の stack 消費が原因と判断した。

## 問題

`nepl-cli` が loader/typecheck/codegen/run を OS 既定の main thread stack で実行していたため、正当な stdlib module の再帰的な compiler traversal が native stack 上限に依存していた。

## 影響

self-host CLI code が `std/fs` の directory traversal facade を nepl-cli 経由で安全に使えず、stdlib discovery の end-to-end 検証を Rust CLI runner で固定できない。

## 修正方針

`nepl-cli` の実処理を明示的に大きい stack の worker thread で実行し、compiler pipeline が OS 既定の main thread stack に依存しないようにする。
`std/fs` を import して `fs_read_dir` を実行する回帰テストの ignored を外す。

## 検証

Run cargo test -p nepl-cli run_wasi_std_fs_read_dir_returns_stable_directory_entries -- --nocapture without stack overflow, then run the full nepl-cli run_wasi_ filter.

## 解決内容

- `nepl-cli` の `execute` を `32 MiB` stack の worker thread で実行する構造へ分離した。
- worker thread が panic した場合は panic payload を `anyhow` error に変換し、通常の `Result` 経路で呼び出し側へ返すようにした。
- `run_wasi_std_fs_read_dir_returns_stable_directory_entries` の ignored を外し、`std/fs` facade の `fs_read_dir` を nepl-cli の通常実行経路で回帰固定した。

## 検証結果

- `cargo fmt --check`: pass
- `cargo test -p nepl-cli run_wasi_std_fs_read_dir_returns_stable_directory_entries -- --nocapture`: 1 passed
- `target/debug/nepl-cli.exe --check -i import.nepl --target wasi`: exit 0
- `target/debug/nepl-cli.exe --run -i std_readdir.nepl --target wasi`: exit 0
- `cargo test -p nepl-cli run_wasi_ -- --nocapture`: 9 passed
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-stack-worker.json`: 13/13 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
