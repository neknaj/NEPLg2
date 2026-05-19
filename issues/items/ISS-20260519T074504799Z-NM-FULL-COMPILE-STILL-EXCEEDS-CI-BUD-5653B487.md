---
id: ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487
title: "NM full compile still exceeds CI budget in Resource IR summary stages"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-19
updated: 2026-05-19
target: "nepl-core/src/resource/**, examples/nm.nepl, stdlib/nm/**"
---

# ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487: NM full compile still exceeds CI budget in Resource IR summary stages

## 概要

TypeCtx を raw identity summary replay に通しても examples/nm.nepl は 8分の local probe で完了しない。修正後 stage timing では resource_initialized_moves が約104秒で、その大半は resource_initialized_raw_init_summaries 約86秒。さらに resource_effect_boundaries はその後 140秒以上戻らない。

## 対象

- `nepl-core/src/resource/**, examples/nm.nepl, stdlib/nm/**`

## 根拠

- 修正前 probe では `resource_initialized_raw_init_summaries=84656ms`、`resource_initialized_function_checks=14985ms`、`resource_initialized_moves=102500ms` の後、`resource_effect_boundaries` が数分戻らなかった。
- `ISS-20260519T073007560Z-RESOURCEEFFECT-RAW-IDENTITY-SUMMARY--1691BDDC` で raw identity summary replay の `TypeCtx` 欠落は修正した。
- 修正後 probe でも `resource_initialized_raw_init_summaries=86269ms`、`resource_initialized_function_checks=15228ms`、`resource_initialized_moves=104483ms` で、さらに `resource_effect_boundaries` が 140 秒以上戻らなかった。
- `ISS-20260519T081602763Z-RESOURCE-VARIANT-INIT-SUMMARY-SCANS--5F80D6A9` で concrete non-enum return の variant summary replay を型で止めた後、`resource_initialized_raw_init_summaries=76306ms`、`resource_initialized_moves=95503ms` まで改善した。
- `ISS-20260519T090154240Z-RESOURCE-INIT-SUMMARY-RELEASE-REQUIR-58379116` で release requirement の parameter seed を raw-address carrier 型に絞った後、`resource_initialized_raw_init_summaries=43194ms`、`resource_initialized_moves=65689ms` まで改善した。
- `ISS-20260519T092414550Z-RESOURCE-RAW-POINTER-SUMMARY-TREATS--CFB63B46` で enum-backed owner storage を structural owner carrier として認識するようにした後、`resource_effect_boundaries=786ms` まで改善した。
- 同 probe は timeout ではなく `resource.owner.no_free_obligation` / `resource.owner.unavailable` に到達した。残る blocking issue は `ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977` の cliarg dependent host-span owner proof である。
- したがって timeout の根本は TypeCtx 欠落単独ではなく、Resource IR summary stage 全体の再計算量 / summary propagation / effect boundary proof の設計に残っている。

## 問題

TypeCtx を raw identity summary replay に通しても examples/nm.nepl は 8分の local probe で完了しない。修正後 stage timing では resource_initialized_moves が約104秒で、その大半は resource_initialized_raw_init_summaries 約86秒。さらに resource_effect_boundaries はその後 140秒以上戻らない。

## 影響

NM CLI の native compile smoke と examples doctest が CI で失敗し、静的検査の正確性検証と deploy status が不安定になる。timeout や skip ではなく、Resource IR summary の計算量を型証明・依存グラフ・関数責務境界から削減する必要がある。

## 修正方針

raw init summary と effect boundary summary を個別モジュール allowlist ではなく型・ResourceOp・call dependency に基づいて pruning する。NM parser/html のような大きい source で stage timing を比較し、default CI budget 内へ戻す。

2026-05-19 時点で、variant-param summary 側の concrete non-enum return pruning、release requirement の plain string view over-seed、enum-backed owner storage の raw pointer summary carrier 誤分類は完了。`resource_effect_boundaries` は timeout 主因から外れた。さらに `ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977` で cliarg dependent host-span owner proof を解決した後、`examples/nm.nepl` は stage timing 付き local probe で完了した。

## 検証

- `NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output tmp\agent1-nm-stage6-after-cliarg.wasm`: pass
  - cargo build 込み wall time: 105.4s
  - `resource_static_check=68386ms`
  - `resource_initialized_raw_init_summaries=43546ms`
  - `resource_initialized_moves=65099ms`
  - `resource_effect_boundaries=422ms`
  - `resource_owner_obligations=2309ms`
  - output wasm: 88364 bytes

## 解決内容

- この performance issue の最後の blocker だった cliarg dependent host-span owner proof は `ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977` で解決した。
- その後の `examples/nm.nepl` full compile probe は timeout ではなく正常完了したため、「8分でも完了しない」「CI 10分枠を超える」という issue 本体は解消した。
- raw init summary はまだ約43.5秒を占めるため今後の最適化余地は残るが、今回の P1 performance issue は CI budget blocker としては解決済みとする。追加の最適化が必要になった場合は、別 issue として raw init summary の再計算量を測定し直す。

## 関連

- split from fixed correctness issue: `ISS-20260519T073007560Z-RESOURCEEFFECT-RAW-IDENTITY-SUMMARY--1691BDDC`
- partial fixed performance sub-issue: `ISS-20260519T081602763Z-RESOURCE-VARIANT-INIT-SUMMARY-SCANS--5F80D6A9`
- partial fixed performance sub-issue: `ISS-20260519T090154240Z-RESOURCE-INIT-SUMMARY-RELEASE-REQUIR-58379116`
- fixed performance sub-issue: `ISS-20260519T092414550Z-RESOURCE-RAW-POINTER-SUMMARY-TREATS--CFB63B46`
- fixed blocking issue: `ISS-20260519T092458711Z-CLIARG-ARGS-GET-DEPENDENT-OWNER-PROO-3D72A977`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
