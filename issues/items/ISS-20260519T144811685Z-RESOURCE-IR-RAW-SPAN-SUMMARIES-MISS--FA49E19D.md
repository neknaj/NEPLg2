---
id: ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D
title: "Resource IR raw span summaries miss loop-guarded symbolic byte loads"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/owner_host_memory_summary.rs, nepl-core/src/resource/owner_control.rs, nepl-core/src/resource/owner_summary.rs, stdlib/std/env/cliarg/cstr.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D: Resource IR raw span summaries miss loop-guarded symbolic byte loads

## 概要

RawMemoryOp::LoadU8 on mem_ptr_add(p, i) inside loops guarded by i < max_len does not currently summarize the required base span p[0..max_len]. Direct import of cstr_to_str_bounded_result can therefore read through a one-byte RegionToken with max_len 100 without a caller-side owner extent diagnostic.

## 対象

- `nepl-core/src/resource/owner_host_memory_summary.rs, nepl-core/src/resource/owner_control.rs, nepl-core/src/resource/owner_summary.rs, stdlib/std/env/cliarg/cstr.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- `tmp/agent1-cstr-span-probe.nepl` で、`alloc_region<u8> 1` から得た `MemPtr<u8>` を `cstr_to_str_bounded_result p 100` に渡しても compile が通ることを確認した。
- 同じ `MemPtr + 明示 length` を `string_from_utf8_mem_result p 100` に渡す場合は `ISS-20260519T142436433Z-RESOURCE-IR-RAW-MEMORY-SPAN-SUMMARIE-FB862D7E` の修正後に `resource.owner.unavailable` で拒否される。
- 差分は `cstr_len_bounded_result` が `while i < max_len` の条件下で `load_u8 (mem_ptr_add p i)` を行い、raw operation の直接引数には `max_len` ではなく `p+i` と 1 byte access だけが現れる点にある。
- したがって、単なる raw operation span summary ではなく、loop/path condition を span proof として summary 化する機構が必要である。

## 問題

RawMemoryOp::LoadU8 on mem_ptr_add(p, i) inside loops guarded by i < max_len does not currently summarize the required base span p[0..max_len]. Direct import of cstr_to_str_bounded_result can therefore read through a one-byte RegionToken with max_len 100 without a caller-side owner extent diagnostic.

## 影響

Bounded byte scanners rely on loop guard convention rather than a Resource IR proof that the backing owner covers the full searched span. This leaves C string conversion and similar byte-scanning helpers weaker than the MemPtr = non-owning pointer design requires.

## 修正方針

Teach owner summary generation to preserve loop/path conditions as a generic span proof: when a raw byte load uses base + symbolic offset and the active condition proves 0 <= offset < bound, record a base pointer/bound extent requirement in the callee summary. Apply the same mechanism to all bounded byte scanners rather than special-casing cliarg/cstr.

## 検証

Add compile_fail regression for direct cstr_to_str_bounded_result import with a one-byte RegionToken and max_len 100, plus positive regressions for matching bounds and for ordinary cliarg_get paths.
