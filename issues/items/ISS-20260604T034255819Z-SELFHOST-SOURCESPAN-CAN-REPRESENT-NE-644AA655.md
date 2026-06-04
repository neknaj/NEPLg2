---
id: ISS-20260604T034255819Z-SELFHOST-SOURCESPAN-CAN-REPRESENT-NE-644AA655
title: "selfhost SourceSpan can represent negative or inverted spans"
area: selfhost
status: open
resolved: false
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
