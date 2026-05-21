---
id: ISS-20260521T222108027Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-A3B78241
title: "Drop traversal range certificate preserve check ignores Move output"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_summary_build_range_preserve.rs, nepl-core/src/resource/collection_slot_summary_build_range_certificate_tests.rs"
---

# ISS-20260521T222108027Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-A3B78241: Drop traversal range certificate preserve check ignores Move output

## 概要

full-range collection slot drop traversal certificate generation checks ResourceOp::Move source when proving that storage and initialized_count are preserved, but it does not check the Move output. A loop body can therefore write a new value into the storage/count anchor through Move output while still being treated as preserving the certificate anchor.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_range_preserve.rs, nepl-core/src/resource/collection_slot_summary_build_range_certificate_tests.rs`

## 根拠

- `collection_slot_summary_build_range_preserve.rs` は `body_preserves_place` で `ResourceOp::Move` を `source` のみで判定していた。
- `ResourceOp::Move { source, output }` は value transfer の result として `output` place を生成するため、`output` が storage / initialized_count anchor と一致する場合は loop body が full-range certificate の対象 state を保存していない。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) の Stage 6 方針では、full initialized range cleanup は loop induction と body witness に結び付いた typed certificate でなければならない。

## 問題

full-range collection slot drop traversal certificate generation checks ResourceOp::Move source when proving that storage and initialized_count are preserved, but it does not check the Move output. A loop body can therefore write a new value into the storage/count anchor through Move output while still being treated as preserving the certificate anchor.

## 影響

Resource IR may emit a ForallInitializedRange summary from a loop whose body changed the storage or initialized_count place. That can make caller-side non-Copy collection cleanup trust a stale traversal certificate, undermining the generic proof boundary required before non-Copy collection payload support.

## 修正方針

Treat ResourceOp::Move as preserving a protected anchor only when neither source nor output touches it. Add focused regression tests for storage/count Move-output overwrites and keep ordinary source-only moves rejected or preserved according to the existing typed place-touch rules.

## 検証

cargo test -p nepl-core --lib collection_slot_summary_loop_induction -- --nocapture; cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --nocapture; cargo check -p nepl-core; node nodesrc/issues.js check --dir issues

## 2026-05-22 修正

Agent 1 が修正した。

- `body_preserves_place` の `ResourceOp::Move` 判定を、`source` と `output` の両方が protected place に触れない場合だけ preserve とするようにした。
- full-range drop traversal certificate の unit regression に、loop body の induction step 後に `Move` output が `storage` / `initialized_count` を上書きするケースを追加した。
- これは stdlib helper 名や collection module allowlist ではなく、Resource IR の typed `ResourceOp::Move` semantics を certificate preservation に反映する修正である。

検証:

- `cargo test -p nepl-core --lib collection_slot_summary_loop_induction -- --nocapture`: passed
- `cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --nocapture`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed with CRLF normalization warnings only
