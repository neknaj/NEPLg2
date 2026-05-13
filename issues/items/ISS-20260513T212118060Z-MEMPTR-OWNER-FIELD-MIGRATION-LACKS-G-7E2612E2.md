---
id: ISS-20260513T212118060Z-MEMPTR-OWNER-FIELD-MIGRATION-LACKS-G-7E2612E2
title: "MemPtr owner field migration lacks global source policy"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nodesrc/test_stdlib_memptr_owner_field_policy.js, nodesrc/run_source_policy_regressions.js, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md"
---

# ISS-20260513T212118060Z-MEMPTR-OWNER-FIELD-MIGRATION-LACKS-G-7E2612E2: MemPtr owner field migration lacks global source policy

## 概要

Stage A says new public APIs must not use MemPtr as an owner field, but existing source policies only check individual modules such as Vec, ByteBuf, builders, and streamio. A new stdlib struct can add a MemPtr or Option<MemPtr> storage field without being classified as a transitional owner-model debt.

## 対象

- `nodesrc/test_stdlib_memptr_owner_field_policy.js, nodesrc/run_source_policy_regressions.js, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`

## 根拠

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の Stage A は、新規 public API が `MemPtr` を owner field として使うことを禁止する source policy を要求している。
- 既存 policy は `Vec`、`ByteBuf`、builder、streamio など個別 module の境界を検査しているが、stdlib 全体の `struct` field 走査はなかった。
- そのため新しい stdlib module が `data <MemPtr<T>>` や `ptr <Option<MemPtr<u8>>>` を追加しても、個別 policy の監視範囲外なら Stage A 違反として検出されない。

## 問題

Stage A says new public APIs must not use MemPtr as an owner field, but existing source policies only check individual modules such as Vec, ByteBuf, builders, and streamio. A new stdlib struct can add a MemPtr or Option<MemPtr> storage field without being classified as a transitional owner-model debt.

## 影響

Raw-memory-backed storage can spread while Resource IR and stdlib are migrating to MemPtr as non-owning pointer plus OwnedRegion/OwnedBuffer owner tokens. That weakens the static-check complexity reduction plan and makes later compiler proof work chase newly introduced debt.

## 修正方針

Add a stdlib-wide source policy that scans struct fields for direct MemPtr storage fields, permits only explicitly documented transitional fields, and treats every permitted field as migration debt rather than semantic proof. Add the policy to the source regression runner and update the Stage A documentation.

## 検証

Run the new source policy, the full source policy regression runner, issue index check, and focused related policies.

## 対応内容

- `nodesrc/test_stdlib_memptr_owner_field_policy.js` を追加し、stdlib 配下の `struct` field を走査して直接 `MemPtr` / `Option<MemPtr>` を保持する field を検出するようにした。
- 現在残っている 8 field は migration debt として明示した。これは semantic safety proof ではなく、`OwnedRegion` / `OwnedBuffer` / typed scanner state へ移すまでの既知残件を固定するための監視である。
- policy は新しい `MemPtr` owner-like field を拒否し、逆に既存 transitional field が解消された場合は stale exception として削除を求める。
- `nodesrc/run_source_policy_regressions.js` に追加し、aggregate source policy で常時実行されるようにした。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の Stage A に、今回の監視対象と設計上の位置付けを追記した。

## 検証結果

- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: passed
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
