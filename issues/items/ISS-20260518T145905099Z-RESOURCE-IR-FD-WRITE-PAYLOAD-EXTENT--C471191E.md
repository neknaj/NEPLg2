---
id: ISS-20260518T145905099Z-RESOURCE-IR-FD-WRITE-PAYLOAD-EXTENT--C471191E
title: "Resource IR fd_write payload extent proof still requires free-obligation owner for readable views"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/owner_external_io.rs; nepl-core/src/resource/host_memory_contract.rs; stdlib/std/stdio/raw.nepl; tests/stdlib/neplg2_checker.n.md"
---

# ISS-20260518T145905099Z-RESOURCE-IR-FD-WRITE-PAYLOAD-EXTENT--C471191E: Resource IR fd_write payload extent proof still requires free-obligation owner for readable views

## 概要

After fixing diagnostic label owner classification and rebuilding web/dist, the focused selfhost checker suite passes the core checker doctests but tests/stdlib/neplg2_checker.n.md doctest#1/#2 still fail at stdlib/std/stdio/raw.nepl:32 with resource.owner.no_free_obligation on ResourceOwnerOperation::ExternalIoPayloadExtent inside stdio_fd_write_mem. The targeted stdio scratch owner regression passes, so the remaining failure is the payload/readable span proof path rather than the iovec/nwritten scratch owner cleanup.

## 対象

- `nepl-core/src/resource/owner_external_io.rs; nepl-core/src/resource/host_memory_contract.rs; stdlib/std/stdio/raw.nepl; tests/stdlib/neplg2_checker.n.md`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture` は pass しており、`stdio_fd_write_mem` の iovec / nwritten scratch owner cleanup は少なくとも最小 regression では証明できている。
- 同じ `trunk build` 後の selfhost checker focused suite では、core checker / module / pipeline doctest は pass するが、`tests/stdlib/neplg2_checker.n.md` の stdio 付き doctest だけが `ExternalIoPayloadExtent` で失敗する。
- 失敗箇所は `/stdlib/std/stdio/raw.nepl:32` の `fd_write` で、payload span 側が `NoFreeObligation` と判定されている。これは `fd_write` の read payload が「free obligation owner を消費するメモリ」ではなく「live かつ initialized な readable span」を必要とする、という責務分割がまだ Resource IR に表現しきれていないことを示す。
- 修正は stdlib/std/stdio の特定 helper を列挙して許可するのではなく、`HostMemorySpan` / raw alias / initialized range / storage extent をつなぐ汎用 proof path で行う必要がある。

## 問題

After fixing diagnostic label owner classification and rebuilding web/dist, the focused selfhost checker suite passes the core checker doctests but tests/stdlib/neplg2_checker.n.md doctest#1/#2 still fail at stdlib/std/stdio/raw.nepl:32 with resource.owner.no_free_obligation on ResourceOwnerOperation::ExternalIoPayloadExtent inside stdio_fd_write_mem. The targeted stdio scratch owner regression passes, so the remaining failure is the payload/readable span proof path rather than the iovec/nwritten scratch owner cleanup.

## 影響

Any stdio output path that writes from a readable non-owning view such as string-backed data can be rejected even when the memory is live and initialized. This blocks stdout-based selfhost doctests and indicates that host-visible read spans are still tied too closely to free-obligation ownership instead of a generic readable/live extent proof.

## 修正方針

Redesign the external IO owner extent check so fd_write payload spans consume a typed readable-memory extent proof, not only a transferable free-obligation owner. Reuse Resource IR raw alias, initialized range, and storage/live extent facts through a generic HostMemorySpan proof path; keep iovec/nwritten descriptor scratch as owner-token-backed RegionToken storage. Do not whitelist stdlib/std/stdio helpers or individual modules.

## 検証

Add Resource IR regressions for fd_write from a string/read-only non-owning view, fd_write from RegionToken-backed scratch, and missing/unbounded readable extent rejection. Re-run the focused selfhost checker suite including tests/stdlib/neplg2_checker.n.md after trunk build.
