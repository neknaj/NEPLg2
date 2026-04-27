---
id: ISS-20260426T213057127Z-WASM32-COMPILE-TEST-BUILDS-UNIX-ONLY-95E8BF55
title: "wasm32 compile-test builds unix-only cli_output fixture"
area: cli
status: verified
resolved: true
priority: P1
type: test
created: 2026-04-26
updated: 2026-04-27
target: "nepl-cli/tests/cli_output.rs, .github/workflows/ci.yml"
---

# ISS-20260426T213057127Z-WASM32-COMPILE-TEST-BUILDS-UNIX-ONLY-95E8BF55: wasm32 compile-test builds unix-only cli_output fixture

## 概要

GitHub Actions run 24967172989 compile-test fails under wasm32-unknown-unknown because nepl-cli/tests/cli_output.rs imports std::os::unix::fs::PermissionsExt and calls Permissions::set_mode.

## 対象

- `nepl-cli/tests/cli_output.rs, .github/workflows/ci.yml`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 compile-test fails under wasm32-unknown-unknown because nepl-cli/tests/cli_output.rs imports std::os::unix::fs::PermissionsExt and calls Permissions::set_mode.

## 影響

The workspace cannot pass cargo test --target wasm32-unknown-unknown --no-run, so CI cannot guard the Rust wasm32 compile boundary.

## 修正方針

Gate the unix-only executable permission test with cfg(unix) and keep a wasm32-compatible assertion path, or move the host permission mutation behind a helper that is not compiled for wasm32.

## 検証

cargo test --target wasm32-unknown-unknown --no-run --all --all-features --locked passes in GitHub Actions compile-test.

## 対応記録

- 原因: `write_fake_clang` の Unix shell fixture が `#[cfg(not(windows))]` で守られており、Unix ではない `wasm32-unknown-unknown` target でも `std::os::unix::fs::PermissionsExt` と `Permissions::set_mode` を compile していた。
- 対応: Unix fixture を `#[cfg(unix)]` に狭め、Windows でも Unix でもない compile target には実行権限変更を含まない fallback fixture を用意した。host 実行用の Windows `.cmd` と Unix executable shell script は維持した。
- 検証:
  - `cargo fmt --all --check`: pass
  - `cargo test -p nepl-cli --target wasm32-unknown-unknown --test cli_output --no-run`: pass
  - `cargo test -p nepl-cli --test cli_output llvm_target_default_accepts_available_clang_without_fixed_linux_version -- --nocapture`: pass
