---
id: ISS-20260518T095629679Z-RESOURCE-IR-EXTERNAL-IO-IOVEC-PAYLOA-EBED3E34
title: "Resource IR external IO iovec payload extent is not owner-proven"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/owner_external_io.rs; nepl-core/src/resource/external_io_iov_layout.rs; nepl-core/tests/resource_ir.rs; doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260518T095629679Z-RESOURCE-IR-EXTERNAL-IO-IOVEC-PAYLOA-EBED3E34: Resource IR external IO iovec payload extent is not owner-proven

## 概要

Resource owner checking does not prove that fd_read/fd_write iovec payload buffers are backed by a live owner whose extent covers the iovec length. Initialized-state checking can bound reads/writes by nread/nwritten, but a host call can still be modeled as writing or reading beyond the allocation extent.

## 対象

- `nepl-core/src/resource/owner_external_io.rs; nepl-core/src/resource/external_io_iov_layout.rs; nepl-core/tests/resource_ir.rs; doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `check_resource_initialized_moves` は `fd_read` の `nread` 境界や `fd_write` の payload initialized state を扱っていたが、`check_resource_owner_obligations` は `EffectOp::ExternalIo` を owner span proof として扱っていなかった。
- `stdio` / `fs` 側は `RegionToken` owner と `MemPtr` non-owning view へ移行しているため、Resource IR 側でも host ABI へ渡る iovec payload span が owner extent に収まることを証明する必要がある。
- 調査中、非所有 raw address view を iovec descriptor に保存する経路で、view が live owner に alias するだけで free obligation transfer として扱われ、descriptor cell に backing alias が残らないことも確認した。

## 問題

Resource owner checking does not prove that fd_read/fd_write iovec payload buffers are backed by a live owner whose extent covers the iovec length. Initialized-state checking can bound reads/writes by nread/nwritten, but a host call can still be modeled as writing or reading beyond the allocation extent.

## 影響

Memory-safety proof for external IO remains incomplete: stdlib can migrate to RegionToken/MemPtr separation while Resource IR still fails to prove the host-visible raw span against the storage owner. This also leaves future external IO handling dependent on individual helper discipline instead of enum/match-driven Resource IR proof.

## 修正方針

Add an owner-side ExternalIo checker that consumes typed ExternalIoOp, derives iovec payload spans from shared iov layout helpers, resolves non-owning raw views back to their live storage owner, and proves the iovec length against OwnerStorageExtent. Reject missing or mismatched owner extents with a ResourceOwnerOperation dedicated to external IO payload spans.

## 検証

Add Resource IR owner regression tests for fd_read/fd_write iovec payload extent mismatch and matching extent; run focused nepl-core resource_ir tests, resource responsibility policy, issue validation, cargo fmt/check, and git diff --check.

## 対応内容

- iovec layout helper を `external_io_iov_layout.rs` へ移し、initialized checker と owner checker が同じ descriptor 解釈を使うようにした。
- `external_io_iov_contract.rs` を追加し、`ExternalIoOp` の exhaustive match で iovec payload を持つ operation を分類した。
- `owner_external_io.rs` を追加し、`fd_read` / `fd_write` / `fd_pread` / `fd_pwrite` の iovec payload buffer を raw alias から backing owner へ解決し、iovec length と `OwnerStorageExtent` を照合するようにした。
- `ResourceOwnerOperation::ExternalIoPayloadExtent` を追加し、host IO span の owner extent mismatch を通常の dealloc/realloc mismatch と混ぜずに診断できるようにした。
- 非所有 raw address view を raw memory cell へ保存する場合、live owner へ alias していても free obligation を移動せず、descriptor cell に non-owning alias として残すようにした。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_fd_ -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir scratch_cleanup -- --nocapture`: pass。
- `cargo check -p nepl-core --tests`: pass。
- `cargo fmt --all -- --check`: pass。
- `node nodesrc/test_resource_checker_responsibility.js`: pass。
- `node nodesrc/issues.js check --dir issues`: pass。
