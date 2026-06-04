---
id: ISS-20260604T033643338Z-VEC-CONSTRUCTOR-CAPABILITY-REJECTION-463D3E88
title: "Vec constructor capability rejection doctests lost PlainPayload coverage"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/alloc/collections/vec/storage/api.nepl
---

# ISS-20260604T033643338Z-VEC-CONSTRUCTOR-CAPABILITY-REJECTION-463D3E88: Vec constructor capability rejection doctests lost PlainPayload coverage

## 概要

node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js reports that Vec allocation constructors must still reject payloads with neither Copy nor Drop capability in doctests. The expected new<PlainPayload> and with_capacity<PlainPayload> compile-fail coverage is no longer present in the source policy view. This conflicts with the Zenn policy of using static checking and enum/trait capability constraints instead of runtime failure.

## 対象

- `stdlib/alloc/collections/vec/storage/api.nepl`

## 根拠

- 未記入

## 問題

node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js reports that Vec allocation constructors must still reject payloads with neither Copy nor Drop capability in doctests. The expected new<PlainPayload> and with_capacity<PlainPayload> compile-fail coverage is no longer present in the source policy view. This conflicts with the Zenn policy of using static checking and enum/trait capability constraints instead of runtime failure.

## 影響

Collection APIs may regress toward accepting payload types whose lifetime cannot be statically cleaned up, reopening non-Copy owner/drop holes that previous Resource IR work tried to close.

## 修正方針

Restore explicit compile_fail doctests for new<PlainPayload> and with_capacity<PlainPayload>, keep constructor bounds as static capability requirements, and add regular cfg-test-style coverage for Copy, Drop, and neither-Copy-nor-Drop payload categories once the mechanism is available.

## 検証

Run node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, Vec focused doctests, and collection cleanup contract tests.
