---
id: ISS-20260426T060311796Z-SHA256-MODULE-PUBLISHES-BUFFERING-SC-F4601536
title: "sha256 module publishes buffering scaffold instead of SHA-256 digest"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/hash/sha256.nepl
---

# ISS-20260426T060311796Z-SHA256-MODULE-PUBLISHES-BUFFERING-SC-F4601536: sha256 module publishes buffering scaffold instead of SHA-256 digest

## 概要

stdlib/alloc/hash/sha256.nepl は new_sha256、sha256_update、sha256_finalize を公開しているが、finalize は 32-byte digest ではなく入力 buffer をそのまま返す。コメントでも draft state / scaffold と明記されている。

## 対象

- `stdlib/alloc/hash/sha256.nepl`

## 根拠

- 未記入

## 問題

stdlib/alloc/hash/sha256.nepl は new_sha256、sha256_update、sha256_finalize を公開しているが、finalize は 32-byte digest ではなく入力 buffer をそのまま返す。コメントでも draft state / scaffold と明記されている。

## 影響

sha256 という module 名を信じた呼び出し側が、cache key、integrity check、artifact fingerprint に実 digest ではない値を使う危険がある。self-host compiler の file hash 設計にも誤用され得る。

## 修正方針

SHA-256 padding と compression を実装して known vector を通す。短期的に実装しない場合は public sha256 API から外すか scaffold 名へ隔離し、digest と誤認できない名前にする。

## 検証

empty string、abc、multi-block 入力の known SHA-256 vector doctestを追加し、finalize が 32 byte の digest を返すことを確認する。
