---
id: ISS-20260513T151511232Z-RESOURCE-OWNER-ALIAS-RESOLUTION-CAN--CB5B7B73
title: "Resource variant owner return can reconsume moved source after payload owner is materialized"
area: RESOURCE
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-14
target: "nepl-core/src/resource/owner_alias.rs; nepl-core/src/resource/owner_variant.rs; nepl-core/tests/resource_ir.rs"
---

# ISS-20260513T151511232Z-RESOURCE-OWNER-ALIAS-RESOLUTION-CAN--CB5B7B73: Resource variant owner return can reconsume moved source after payload owner is materialized

## 概要

When a helper returns an owner inside a `Result::Ok` payload and consumes the same argument on `Err` paths, the caller can return that `Result` through a branch/local binding. Resource IR correctly derives a variant owner-return summary from the helper body, but later can also transfer the concrete payload owner through branch/local/result moves. If the pending variant owner-return effect is applied again at function return, it tries to reconsume the already-moved source handle and reports a false `resource.owner.use_after_move`.

## 対象

- `nepl-core/src/resource/owner_alias.rs`
- `nepl-core/src/resource/owner_variant.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `tests/stdlib/stdio_read_all.n.md` rejected valid `stdio_read_all_bytes_result` with `ResourceOwnerOperation::ReturnValue` on `buf.field0` in `Moved` state.
- The existing `resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup` regression rejected valid `fs_read_fd_bytes` and `stdio_read_all_bytes_result` owner cleanup paths for the same reason.
- Debugging the owner table showed the returned `Result::Ok` payload already had a live owner under the result place while the original `buf.field0` source was moved. The pending variant return was therefore a duplicate proof effect, not a missing source owner.

## 問題

`PendingVariantOwnerEffects::materialize_result_owner_effects` applied every pending owner return unconditionally. That is correct for opaque call results that have not yet materialized the selected payload owner, but incorrect after ordinary Resource IR operations have already moved the owner into the same result payload. The checker then treated the source handle as if it must be moved again.

The alias resolver also selected moved/freed aliases too early when raw-address aliases contained both available and unavailable owner states. That made stale moved sources mask valid live aliases in related returned-owner paths.

## 影響

Valid `ByteBuf`-producing APIs were rejected by the memory-safety checker. This blocked stdio/fs read doctests and made Resource IR ownership checking unstable around `Result` payload owner returns, because a proved live returned owner could be masked by a stale moved source.

## 修正方針

- `apply_pending_variant_owner_return` now treats a target payload that already has a transferable owner as an already-materialized variant owner return. It does not reconsume the source in that case, but still records the source as handled so paired variant consumption does not run for the same source.
- Fresh/maybe pending variant returns also avoid overwriting an already-materialized target owner.
- Owner alias fallback now prefers available owner states (`Live` / `MaybeFreed`, then `Reserved`) before considering moved/freed aliases. Direct use of a moved place still reports moved state, so the diagnostic is not weakened.
- Added a Resource IR regression where a helper returns a `Result::Ok` owner on success, consumes the same source on error, and the caller returns that helper result through a local binding.

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_prefers_live_return_owner_over_moved_source_alias -- --nocapture`: passed
- `cargo fmt --package nepl-core --check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdio_read_all.n.md --no-tree -o tmp/agent1-stdio-read-all-owner-alias-fix.json -j 1 --assert-io --dist web/dist`: total=2, passed=2
