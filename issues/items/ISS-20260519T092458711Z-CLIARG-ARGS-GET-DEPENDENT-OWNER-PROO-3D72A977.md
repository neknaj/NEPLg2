---
id: ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977
title: "cliarg args_get dependent owner proof fails after NM effect summary unblocks"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/owner_host_dependent_span.rs; nepl-core/src/resource/host_dependent_length.rs; stdlib/std/env/cliarg/raw.nepl"
---

# ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977: cliarg args_get dependent owner proof fails after NM effect summary unblocks

## 概要

After enum-backed owner storage is no longer misclassified as a raw pointer carrier, examples/nm.nepl reaches owner obligation checking and fails in std/env/cliarg/raw.nepl. The diagnostics are resource.owner.no_free_obligation for cli_args_sizes_result(meta) and resource.owner.unavailable / ExternalIoPayloadExtent for args_get(argv_raw, argv_buf_raw), even though the cliarg scratch is intended to be RegionToken-backed and the prior args_sizes_get proof should justify the dependent args_get spans.

## 対象

- `nepl-core/src/resource/owner_host_dependent_span.rs; nepl-core/src/resource/host_dependent_length.rs; stdlib/std/env/cliarg/raw.nepl`

## 根拠

- `ISS-20260519T092414550Z-RESOURCE-RAW-POINTER-SUMMARY-TREATS--CFB63B46` の修正後、`examples/nm.nepl` は `resource_effect_boundaries=786ms` で通過し、次の `resource_owner_obligations` まで進んだ。
- 同じ probe で `cliarg_count_result__unit__Result_T_E_i32_i32__imp` の `cli_args_sizes_result meta` に `resource.owner.no_free_obligation` が出た。
- `cliarg_get_checked__i32__Option_T_str__imp` では、`args_get argv_raw argv_buf_raw` に対して `argv_region` / `argv_buf_region` の `ExternalIoPayloadExtent` が `Live { storage, extent }` のまま `resource.owner.unavailable` として報告された。
- 既存の `ISS-20260518T104225390Z-ARGS-GET-AND-ENVIRON-GET-NEED-DEPEND-64A7F146` は dependent host-span proof の基本形を修正済みだが、nm の full compile で raw cliarg helper 境界を跨いだ RegionToken owner extent と prior size proof の接続がまだ不足している。

## 問題

After enum-backed owner storage is no longer misclassified as a raw pointer carrier, examples/nm.nepl reaches owner obligation checking and fails in std/env/cliarg/raw.nepl. The diagnostics are resource.owner.no_free_obligation for cli_args_sizes_result(meta) and resource.owner.unavailable / ExternalIoPayloadExtent for args_get(argv_raw, argv_buf_raw), even though the cliarg scratch is intended to be RegionToken-backed and the prior args_sizes_get proof should justify the dependent args_get spans.

## 影響

NM full compile still cannot complete after the effect-boundary timeout is removed. More importantly, dependent host-span proof for WASI argv is not robust enough across the raw cliarg helper boundary, so Resource IR may reject valid owner-backed scratch while the exact proof gap remains hidden behind an examples failure.

## 修正方針

Trace HostSize proof and owner storage extent propagation from cli_args_sizes_result through cliarg_count_result and cliarg_get_checked, then fix the generic dependent host-span proof path so args_get/environ_get are accepted only when the source-derived RegionToken extents and prior size facts match. Do not add stdlib allowlists or operation-name exceptions.

## 検証

Run focused Resource IR args_get/environ_get owner tests, cliarg source policy, cliarg doctests, examples/nm.nepl stage-timing compile probe, issue check, and diff check.

## 関連

- exposed by fixed performance issue: `ISS-20260519T092414550Z-RESOURCE-RAW-POINTER-SUMMARY-TREATS--CFB63B46`
- parent performance issue: `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487`
- prior dependent span issue: `ISS-20260518T104225390Z-ARGS-GET-AND-ENVIRON-GET-NEED-DEPEND-64A7F146`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
