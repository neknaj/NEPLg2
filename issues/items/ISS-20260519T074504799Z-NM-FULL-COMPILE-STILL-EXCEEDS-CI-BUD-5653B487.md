---
id: ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487
title: "NM full compile still exceeds CI budget in Resource IR summary stages"
area: core
status: open
resolved: false
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
- したがって timeout の根本は TypeCtx 欠落単独ではなく、Resource IR summary stage 全体の再計算量 / summary propagation / effect boundary proof の設計に残っている。

## 問題

TypeCtx を raw identity summary replay に通しても examples/nm.nepl は 8分の local probe で完了しない。修正後 stage timing では resource_initialized_moves が約104秒で、その大半は resource_initialized_raw_init_summaries 約86秒。さらに resource_effect_boundaries はその後 140秒以上戻らない。

## 影響

NM CLI の native compile smoke と examples doctest が CI で失敗し、静的検査の正確性検証と deploy status が不安定になる。timeout や skip ではなく、Resource IR summary の計算量を型証明・依存グラフ・関数責務境界から削減する必要がある。

## 修正方針

raw init summary と effect boundary summary を個別モジュール allowlist ではなく型・ResourceOp・call dependency に基づいて pruning する。NM parser/html のような大きい source で stage timing を比較し、default CI budget 内へ戻す。

2026-05-19 時点で、variant-param summary 側の concrete non-enum return pruning は完了。残る主因は raw initialization summary の release requirement collection と、その後の effect boundary proof である。引き続き stdlib/nm 名の allowlist ではなく、ResourceOp と TypeCtx から導ける汎用 proof / dependency pruning として対応する。

## 検証

NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output tmp/... が CI 10分枠未満で完了すること。examples doctest と affected Resource IR unit tests を通すこと。

## 関連

- split from fixed correctness issue: `ISS-20260519T073007560Z-RESOURCEEFFECT-RAW-IDENTITY-SUMMARY--1691BDDC`
- partial fixed performance sub-issue: `ISS-20260519T081602763Z-RESOURCE-VARIANT-INIT-SUMMARY-SCANS--5F80D6A9`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
