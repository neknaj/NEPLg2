---
id: ISS-20260520T165035484Z-BULK-RAW-COPY-DOES-NOT-TRANSFER-INIT-C34AC2CD
title: "Bulk raw copy does not transfer initialized raw range evidence"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T165035484Z-BULK-RAW-COPY-DOES-NOT-TRANSFER-INIT-C34AC2CD: Bulk raw copy does not transfer initialized raw range evidence

## 概要

RawMemoryOp::BulkCopy/BulkMove lifecycle handling copies initialized Copy cell entries but does not transfer initialized raw byte/element range evidence. A raw range initialized by fill or external proof can therefore be lost across a byte copy even when extent proof should preserve it.

## 対象

- `nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

RawMemoryOp::BulkCopy/BulkMove lifecycle handling copies initialized Copy cell entries but does not transfer initialized raw byte/element range evidence. A raw range initialized by fill or external proof can therefore be lost across a byte copy even when extent proof should preserve it.

## 影響

Buffer and collection code that uses raw copy as a storage operation cannot rely on Resource IR to preserve initialized prefix facts. Fixing this with stdlib allowlists would violate the generic proof-boundary design; leaving it unfixed causes false positives or pressures weakening RawMemoryLoadCell.

## 修正方針

Extend the typed raw lifecycle transition model so bulk copy/move consumes an explicit extent/count proof and transfers only range evidence covered by that proof, or emits a precise diagnostic when range transfer cannot be proven.

## 検証

Add Resource IR regressions for initialized byte ranges and element ranges crossing bulk copy/move, plus negative tests where count/extent proof is missing.
