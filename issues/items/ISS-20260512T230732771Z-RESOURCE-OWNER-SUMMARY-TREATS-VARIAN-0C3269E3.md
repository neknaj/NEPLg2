---
id: ISS-20260512T230732771Z-RESOURCE-OWNER-SUMMARY-TREATS-VARIAN-0C3269E3
title: "Resource owner summary treats variant path conditions as global unreachable facts"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/resource/owner_variant.rs; nepl-core/src/resource/owner_summary_variant_conditions.rs; nepl-core/src/resource/owner_summary_variant_paths.rs; stdlib/alloc/io/bytebuilder.nepl; tests/stdlib/byte_builder.n.md"
source: "Agent 1 ResourceIR owner summary review 2026-05-13"
---

# ISS-20260512T230732771Z-RESOURCE-OWNER-SUMMARY-TREATS-VARIAN-0C3269E3: Resource owner summary treats variant path conditions as global unreachable facts

## 概要

Owner summary variant_conditions recorded multiple path-local facts for the same variant, but caller-side reachability treated any false fact as making the whole variant unreachable. A reachable Result arm could therefore be skipped, so the checker never observed deallocation paths such as free_src src and reported a stale source owner leak.

## 対象

- `nepl-core/src/resource/owner_variant.rs; nepl-core/src/resource/owner_summary_variant_conditions.rs; nepl-core/src/resource/owner_summary_variant_paths.rs; stdlib/alloc/io/bytebuilder.nepl; tests/stdlib/byte_builder.n.md`

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 根拠

ByteBuilder LEB32 loop and nested Result owner focused tests reproduced false maybe_leak before the variant path condition fix.

## 問題

Owner summary variant_conditions recorded multiple path-local facts for the same variant, but caller-side reachability treated any false fact as making the whole variant unreachable. A reachable Result arm could therefore be skipped, so the checker never observed deallocation paths such as free_src src and reported a stale source owner leak.

## 影響

Reachable match arms in ResourceIR summaries could be silently ignored. This affected memory safety diagnostics because owner obligations were checked against an incomplete control-flow approximation.

## 修正方針

Represent conditionless variant paths explicitly, combine facts per path, and treat variants as unreachable only when every recorded path alternative is definitely false. Share inactive enum payload owner retirement between direct checking and summary path entry, and keep ByteBuilder source copies borrowed while owned ByteBuf sources are deallocated centrally.

## 実装結果

- `OwnerValueCondition::Always` を追加し、条件を持たない variant return path を暗黙条件ではなく enum variant として表現した。
- branch / match / known fact は path 単位で `All(...)` に畳み、同一 variant の複数 path は OR alternatives として保持するようにした。
- caller 側の `record_unreachable_variants` は、同一 variant の全 alternatives が `Some(false)` の場合だけ unreachable と見なす。未知条件または true alternative が残る場合は arm を落とさない。
- inactive enum payload owner の retirement を `owner_match_payload.rs` に分離し、通常の match check と summary path entry の状態遷移を共有した。
- `byte_builder_push_bytes_ref` は source `MemPtr` を borrow として扱い、owned `ByteBuf` source は copy 後に専用 helper で deallocation obligation を閉じるようにした。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_byte_builder_source_ref_deallocatable -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_byte_builder_owner_through_leb32_loop -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_nested_byte_builder_result_owner -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_byte_builder_owner_through_text_result_mapping -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_stdlib_builder_owner_boundary.js`
- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/agent1-byte-builder-owner-after-variant-conditions.json -j 1 --dist web/dist`
