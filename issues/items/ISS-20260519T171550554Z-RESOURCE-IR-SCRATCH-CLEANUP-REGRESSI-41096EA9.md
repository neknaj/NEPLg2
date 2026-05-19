---
id: ISS-20260519T171550554Z-RESOURCE-IR-SCRATCH-CLEANUP-REGRESSI-41096EA9
title: "Resource IR scratch cleanup regressions call private raw span helper"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260519T171550554Z-RESOURCE-IR-SCRATCH-CLEANUP-REGRESSI-41096EA9: Resource IR scratch cleanup regressions call private raw span helper

## 概要

Resource IR fd scratch cleanup regressions still called stdio_write_fd_mem_result directly after the stdio raw span writer was made private. The tests no longer matched the public API boundary that ordinary source must use.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --exact --nocapture` が、`stdio_write_fd_mem_result` の未定義と literal match payload binding の副作用で失敗した。
- `stdio_write_fd_mem_result` は `ISS-20260518T210549005Z-STD-STDIO-WRITE-FACADE-EXPOSES-RAW-M-11591E6E` で private helper に戻しており、通常 source が直接呼べないこと自体は正しい。
- したがって、回帰テスト側が現在の public typed boundary を通らず、過去の raw span writer surface を前提にしていたことが原因である。

## 問題

Resource IR fd scratch cleanup regressions still called stdio_write_fd_mem_result directly after the stdio raw span writer was made private. The tests no longer matched the public API boundary that ordinary source must use.

## 影響

Focused Resource IR tests fail at typecheck time and no longer prove that fd_write scratch cleanup remains safe through the typed public wrapper. This weakens Stage 6 regression coverage for the MemPtr non-owning pointer discipline.

## 修正方針

Update the regressions to enter through stdio_write_fd_byte_result so monomorphization still includes the private fd write loop and raw ABI helper, while the test source respects the public typed boundary.

## 解決内容

- `resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup` は `stdio_write_fd_mem_result` 直呼びをやめ、public API の `stdio_write_fd_byte_result` から入るようにした。
- `resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup` も同じ public wrapper 経由にし、検査対象 prefix へ wrapper 自身を追加した。
- private `stdio_write_fd_mem_result` と `stdio_fd_write_from_result` は monomorphization 後の Resource IR に残るため、raw ABI scratch cleanup と host span proof は引き続き内部実装として検査される。
- stdlib module 名や raw helper 名の allowlist は追加していない。テスト source の入口だけを、現在の public API shape に合わせた。

## 対応 stage

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: `MemPtr = non-owning pointer` / typed owner boundary 分離後の Resource IR 回帰テストを、private raw span helper ではなく public typed wrapper 経由で固定する整備。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --exact --nocapture`: passed
