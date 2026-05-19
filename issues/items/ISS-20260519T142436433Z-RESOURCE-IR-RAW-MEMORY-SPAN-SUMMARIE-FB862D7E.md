---
id: ISS-20260519T142436433Z-RESOURCE-IR-RAW-MEMORY-SPAN-SUMMARIE-FB862D7E
title: "Resource IR raw memory span summaries miss MemPtr extent proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/owner_raw_memory.rs, nepl-core/src/resource/owner_raw_memory_cell.rs, nepl-core/src/resource/owner_raw_memory_span.rs, nepl-core/src/resource/owner_host_memory_summary.rs, nepl-core/src/resource/owner_host_direct_span.rs, nepl-core/src/resource/owner_host_payload_extent.rs, nepl-core/src/resource/effect_return_summary_filter.rs, nepl-core/src/resource/effect_checked_memptr.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260519T142436433Z-RESOURCE-IR-RAW-MEMORY-SPAN-SUMMARIE-FB862D7E: Resource IR raw memory span summaries miss MemPtr extent proof

## 概要

RawMemoryOp::LoadU8, StoreU8, BulkCopy, BulkMove, FillBytes, and Fill currently do not create caller-visible owner extent requirements. A compiler-owned stdlib callee can therefore read or write a MemPtr plus an arbitrary length while the caller only sees a pure function call, so direct imports such as string_from_mem_unchecked_result or cstr_to_str_bounded_result can hide an out-of-bounds RegionToken span from Resource IR.

## 対象

- `nepl-core/src/resource/owner_raw_memory.rs, nepl-core/src/resource/owner_raw_memory_cell.rs, nepl-core/src/resource/owner_raw_memory_span.rs, nepl-core/src/resource/owner_host_memory_summary.rs, nepl-core/src/resource/owner_host_direct_span.rs, nepl-core/src/resource/owner_host_payload_extent.rs, nepl-core/src/resource/effect_return_summary_filter.rs, nepl-core/src/resource/effect_checked_memptr.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `owner_raw_memory.rs` は修正前、`RawMemoryOp::LoadU8` / `StoreU8` / `BulkCopy` / `BulkMove` / `FillBytes` / `Fill` で owner extent を一切確認せず、pending summary requirement も作らなかった。
- そのため通常 source が `alloc_region<u8> 1` から得た `MemPtr<u8>` と `100` を `string_from_mem_unchecked_result` / `string_from_utf8_mem_result` に渡しても、caller 側では `RegionToken` の 1 byte extent と 100 byte 要求の矛盾が検査されなかった。
- 直接 `string_data_ptr` から `load_u8` / `store_u8` する場合は `resource.raw.memory_outside_boundary` で止まる一方、同じ raw operation が stdlib callee 内に隠れると関数 summary で再証明されないことが根本原因だった。

## 問題

RawMemoryOp::LoadU8, StoreU8, BulkCopy, BulkMove, FillBytes, and Fill currently do not create caller-visible owner extent requirements. A compiler-owned stdlib callee can therefore read or write a MemPtr plus an arbitrary length while the caller only sees a pure function call, so direct imports such as string_from_mem_unchecked_result or cstr_to_str_bounded_result can hide an out-of-bounds RegionToken span from Resource IR.

## 影響

Memory safety depends on stdlib API convention instead of Resource IR proof. Ordinary source can combine a one-byte RegionToken view with a larger length and compile through stdlib raw-span helpers, which violates the MemPtr = non-owning pointer / owner extent proof design and blocks self-host safety.

## 修正方針

Derive generic memory span requirements from RawMemoryOp and apply them through function summaries. The checker must prove that the backing owner extent covers every readable and writable byte span at call sites, using Resource IR alias/extent facts rather than stdlib module allowlists.

## 対応

- `OwnerReturnSummary` の `host_memory_span_requirements` を `memory_span_requirements` へ広げ、requirement ごとに `ResourceOwnerOperation` を保持するようにした。
- external IO と raw memory が同じ span proof machinery を使えるようにしつつ、diagnostic operation は `ExternalIoPayloadExtent` と `RawMemoryPayloadExtent` の enum で分けた。
- `RawMemoryOp::Load` / `Store` / `LoadU8` / `StoreU8` / `BulkCopy` / `BulkMove` / `FillBytes` / `Fill` から readable/writable byte span requirement を生成するようにした。
- raw allocator のように owner model より下層で plain raw address を扱う compiler-owned code は、所有者が見つからないだけでは誤診断しない。一方で caller 由来の `MemPtr` は summary requirement として記録し、call site の backing owner extent と照合する。
- `owner_raw_memory.rs` を dispatcher に戻し、raw cell transfer / raw byte span requirement / direct host span proof を `owner_raw_memory_cell.rs`、`owner_raw_memory_span.rs`、`owner_host_direct_span.rs` へ分離した。検査の意味単位を source policy の line budget と module declaration で監視し、静的検査プログラム自体の責務混在を見つけやすくした。
- focused verification 中に `Result<RegionToken<T>, E>` の enum root が raw identity summary から遮断され、checked `MemPtr` proof が空になる別問題を確認したため、[ISS-20260519T154609873Z-RESOURCE-RAW-IDENTITY-SUMMARY-BLOCKS-E4C8EDF4](./ISS-20260519T154609873Z-RESOURCE-RAW-IDENTITY-SUMMARY-BLOCKS-E4C8EDF4.md) として修正した。raw memory span proof は `RegionToken` provenance が caller へ届くことを前提にするため、同じ Stage 6 correctness boundary として検証した。
- `p + i` を 1 byte 読む bounded scanner のうち、loop condition から `p[0..max_len]` を summary 化する残件は [ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D](./ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D.md) に分離した。

## 検証

Add Resource IR regression tests for helper-call summaries that reject a one-byte RegionToken passed with length 100 and accept matching extents. Add stdlib memory-safety doctests for direct string/cstr raw-span imports.

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_string_from_mem_oversized_region_span -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_string_from_mem_string_source_span -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_fd_write_wrapper_iovec_payload_span_summary -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_fd_write_rejects_iovec_payload_extent_mismatch -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_args_get_accepts_host_size_return_summary -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_args_sizes_get_rejects_known_offset_beyond_owner -- --exact --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --lib effect_return_summary_filter -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_mem_ptr_wrapper_with_null_sentinel -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_region_pointer_from_region_provenance -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_region_ptr_at_from_region_provenance -- --exact --nocapture`: pass
- `cargo fmt -p nepl-core -- --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/issues.js check`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-raw-memory-span-summary-memory-safety.json -j 1 --dist web\dist`: total=62, passed=62
