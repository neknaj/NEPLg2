# Tools Review: Rust CLI

対象 commit: `f108cebd`

## 対象

- `nepl-cli/src/main.rs`
- `nepl-cli/src/codegen_llvm.rs`

## 概要

Rust CLI は compile / check / run / artifact output / WASI host / LLVM runner を担っており、実用上の入口として機能している。`--emit wasm,wat,wat-min` の multi output は CI の `wasi-test` job に smoke check があり、LLVM compile-only は `llvm-test` job に分離されている。

一方で `main.rs` は約 77KB で、WASI preview1 host、filesystem preopen、stdio buffering、tty、argument parsing、compile orchestration が 1 file に集中している。現時点では動く実装だが、selfhost CLI の設計参考としては分割が必要である。

## Actions 根拠

GitHub Actions run `25157230630` では次の状態。

- `compile-test`: success。Rust native / wasm32 の compile は通っている。
- `llvm-test`: success。`tests/compiler/llvm_target.n.md` の LLVM compile-only は通っている。
- `wasi-test`: failure。`nodesrc/tests.js -i tests` が `1034 total / 812 passed / 185 failed / 37 errored`。
- `nm-compile`: failure。`examples/nm.nepl` compile が stdlib string/nm owner failure の影響を受ける。

## 良い点

- compile-only と runtime doctest が CI job として分離されている。
- WASI host は fd read/write/readdir/path_open など selfhost に必要な host capability を広く持つ。
- `codegen_llvm.rs` が別 file になっており、LLVM runner の一部は分離済み。
- verbose log は `cli_verbose!` に集約されている。

## 問題

- `main.rs` が大きく、WASI host と CLI orchestration が混在している。
- `AllocState` が stdin/args/preopens/files/tty/stdout buffer をまとめて持ち、責務境界が広い。
- CLI の runtime host 実装は Rust 側に厚く、selfhost S6 で同等機能を実装する際の仕様文書化が不足している。
- CI の runtime failure が stdlib / Resource IR 起因で多く、CLI 自体の failure と区別しにくい。

## 必要な設計

- Rust CLI は `args`, `compile_driver`, `wasi_host`, `artifact_writer`, `llvm_runner` に分割する。
- WASI host の fd/path/tty contract は doc 化し、selfhost CLI が追従できる仕様にする。
- CLI test は exit code だけでなく stdout/stderr/artifact path を明示して検証する。

## 進捗状況

- `nepl-cli`: 実用段階だが大規模リファクタリング対象。
- `nepl-cli/src/codegen_llvm.rs`: 分離済みだが runner contract の doc 化が必要。
- CI compile gate: 通過。
- CI runtime gate: stdlib/Resource IR 起因で failure。
