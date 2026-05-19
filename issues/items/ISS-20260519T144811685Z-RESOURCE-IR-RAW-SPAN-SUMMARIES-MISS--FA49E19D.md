---
id: ISS-20260519T144811685Z-RESOURCE-IR-RAW-SPAN-SUMMARIES-MISS--FA49E19D
title: "Resource IR raw span summaries miss loop-guarded symbolic byte loads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/lower_condition.rs, nepl-core/src/resource/owner_external_io_payload.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/initialized_alias_scalar.rs, nepl-core/tests/resource_ir.rs, stdlib/std/env/cliarg/cstr.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
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

## 解決内容

- 根本原因は `owner_host_memory_summary` 固有ではなく、Resource IR の condition fact lowering が `and flag (lt i max_len)` のような複合条件で片側の boolean conjunct を解釈できない場合、解釈できる `i < max_len` まで丸ごと捨てていたことだった。
- `lower_condition` は `and` / `or` を、認識できる child fact を持つ `All` / `Any` として保持するようにした。`and` の真 branch と `or` の偽 branch でだけ使うため、未認識 conjunct/disjunct を許可条件として扱わず、既知 fact だけを sound に利用できる。
- `i = 0` のような scalar value assignment から `i >= 0` / `i == 0` / `i != 0` などの alias fact を導出し、loop 内 symbolic offset の下限証明が初期値から失われないようにした。
- checked `MemPtr` wrapper 経由の関数 summary でも `memory_span_requirements` は caller 側へ適用するようにし、raw owner transfer summary を抑制する場合でも byte span requirement を落とさないようにした。
- non-owning input payload の no-free-obligation 判定より先に deferred direct memory span requirement を記録し、owner が live でない view でも証明すべき span obligation が握りつぶされないようにした。
- `cstr_len_bounded_result` は source 上で `0 <= i < max_len` を明示する loop guard にし、stdlib 名の allowlist ではなく、source から導出される Resource IR condition fact で bounded scan を証明する形にした。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` も旧 `i < max_len` だけの loop guard ではなく、`0 <= i < max_len` の source proof を要求するように更新した。

## 検証

Add compile_fail regression for direct cstr_to_str_bounded_result import with a one-byte RegionToken and max_len 100, plus positive regressions for matching bounds and for ordinary cliarg_get paths.

## 回帰テスト

- `resource_ir_lowering_keeps_known_conjuncts_in_partial_loop_condition_fact` で、未認識 boolean conjunct と既知 `i < len` fact が混在しても Resource IR が既知 fact を保持することを固定した。
- `resource_ir_owner_check_rejects_cstr_bounded_oversized_region_span` で、1 byte `RegionToken<u8>` 由来の `MemPtr<u8>` に `cstr_to_str_bounded_result p 100` を渡す direct import が `resource.owner.unavailable` で拒否されることを固定した。
- `tests/stdlib/memory_safety.n.md` に同じ compile-fail doctest を追加し、stdlib doctest 側でも bounded C string scan の owner extent proof を監視する。

## 対応 stage

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: `MemPtr = non-owning pointer` / `RegionToken = owner` 分離後の Resource IR owner extent proof を、loop/path condition 付き symbolic byte scan へ広げる修正として扱う。
