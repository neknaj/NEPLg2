---
id: ISS-20260516T021926423Z-RAW-MEMORY-SOURCE-EVIDENCE-ACCEPTS-N-88427FD2
title: "raw memory source evidence accepts non-call helper symbols"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/raw_memory.rs
---

# ISS-20260516T021926423Z-RAW-MEMORY-SOURCE-EVIDENCE-ACCEPTS-N-88427FD2: raw memory source evidence accepts non-call helper symbols

## 概要

Raw memory source capability scans every symbol inside a prefix expression. A raw helper name used as a non-call argument can grant raw memory operation or structural boundary authority even though no raw helper call was observed.

## 対象

- `nepl-core/src/source_capability/raw_memory.rs`

## 根拠

- `nepl-core/src/source_capability/raw_memory.rs` は修正前、prefix expression 内の全 `PrefixItem::Symbol` を走査し、raw helper / raw operation 名に一致すると source capability evidence としていた。
- そのため `consume load_i32` や `consume mem_ptr_addr` のように raw helper 名が値・引数位置に現れるだけでも、compiler-owned stdlib source では raw operation / structural boundary authority が付与され得た。
- 一方で `let cur <i32> load_i32 0` のような NEPL prefix では raw helper call が `let` / type annotation の後ろに現れるため、単純な先頭 item 限定では正当な raw helper 実装を壊す。
- したがって、source proof は「expression の全 symbol」ではなく、「prefix 構文上 call head になり得る位置」の証拠として扱う必要があった。

## 問題

Raw memory source capability scans every symbol inside a prefix expression. A raw helper name used as a non-call argument can grant raw memory operation or structural boundary authority even though no raw helper call was observed.

## 影響

The static check authority gate remains weaker than the intended source proof and can authorize a compiler-owned source file from stale names or value references. This conflicts with the generic proof direction for ResourceIR and makes checker mistakes harder to catch with Rust exhaustiveness.

## 修正方針

Restrict raw memory symbol evidence to prefix call-head syntax, keep explicit intrinsic/raw-body evidence, add focused loader regressions for non-call raw operation and structural helper symbols, and update source-policy checks.

## 解決

2026-05-16 に修正した。

- `collect_expr_raw_memory_evidence` に prefix call-position scanner を追加し、raw memory symbol evidence は call head になり得る位置でだけ収集するようにした。
- expression 先頭に加え、`let` / `set` / `if` / `while` / `addr-of` / `deref` / type annotation / pipe の直後は、NEPL prefix 構文上の nested call head として扱う。これにより `let cur <i32> load_i32 0` のような正当な raw helper implementation evidence は維持する。
- 通常の後続 symbol は raw helper value / argument として扱い、`consume load_i32` や `consume mem_ptr_addr` からは raw operation / structural boundary authority を出さない。
- raw body と intrinsic evidence は explicit low-level evidence として維持した。
- loader regression に raw operation helper の非 call-head と raw structural helper の非 call-head を追加した。
- `nodesrc/test_static_check_boundary_responsibility.js` に call-head evidence と prefix initializer call-position scanner の監視を追加した。

この修正は raw memory helper を stdlib module ごとに許可するものではない。compiler-owned source の authority gate を、構文上の raw helper call evidence に近づけるための修正であり、semantic proof は typed effect / Resource IR 側で行う方針を維持する。

## 検証

cargo test -p nepl-core raw_memory_boundary -- --nocapture; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues

実施済み:

- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
