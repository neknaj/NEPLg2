---
id: ISS-20260604T034255819Z-SELFHOST-SOURCESPAN-CAN-REPRESENT-NE-644AA655
title: "selfhost SourceSpan can represent negative or inverted spans"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/neplg2/core/infra/span.nepl
---

# ISS-20260604T034255819Z-SELFHOST-SOURCESPAN-CAN-REPRESENT-NE-644AA655: selfhost SourceSpan can represent negative or inverted spans

## 概要

Subagent audit found source_span_new accepting raw start/end values without validation and source_span_len able to return negative lengths. This keeps source range validity as a caller convention instead of a static/data contract.

## 対象

- `stdlib/neplg2/core/infra/span.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found source_span_new accepting raw start/end values without validation and source_span_len able to return negative lengths. This keeps source range validity as a caller convention instead of a static/data contract.

## 影響

Parser, diagnostic, and syntax highlight ranges can carry impossible spans, making later range-based diagnostics and editor services harder to trust.

## 修正方針

Add SourceSpanValid or source_span_new_result, make invalid spans a typed diagnostic/result, and reserve unchecked construction for parser-internal proof boundaries only.

## 検証

Add regular tests for inverted spans, negative starts, zero-length spans, valid spans, and diagnostic rendering with invalid input rejected.

## 対応

- `source_span_new_result` / `source_span_empty_result` を追加し、負の `file_id`、負の `start`、逆転 range を `SelfhostSourceSpanBuildError` として返すようにした。
- 既存の直接構築 API は `source_span_new_unchecked` / `source_span_empty_unchecked` へ明示的に改名し、通常 caller が unchecked 境界を安全 API と誤認しないようにした。
- `source_span_len` は invalid span で負の長さを返さず、`Option::None` を返すようにした。
- `source_text_line_span` は checked constructor を通し、line map 由来の span 生成でも内部不整合を `None` に落とす。
- `tests/stdlib/neplg2_span.n.md` と `nodesrc/test_selfhost_source_span_contract.js` を追加し、checked constructor と source policy の両方で回帰を検出する。
