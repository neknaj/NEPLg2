---
id: ISS-20260430T004118434Z-RESOURCE-IR-LACKS-EXPLICIT-NON-OWNIN-D546F9CD
title: "Resource IR lacks explicit non-owning raw address view and fallible realloc state"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource, stdlib/core/mem.nepl"
---

# ISS-20260430T004118434Z-RESOURCE-IR-LACKS-EXPLICIT-NON-OWNIN-D546F9CD: Resource IR lacks explicit non-owning raw address view and fallible realloc state

## 概要

Resource IR currently represents owning raw addresses and non-owning address views through the same i32 alias mechanism. A plain read of an owning raw address and an address view such as add buf 0 can become indistinguishable at owner-check time, while realloc_raw returning 0 cannot express that the old allocation remains live on the failure path.

## 対象

- `nepl-core/src/resource, stdlib/core/mem.nepl`

## 根拠

- KP scanner / fd_read fixture の修正中、`store_i32 iov buf` は所有権移動として扱われるべき raw address と、iovec に渡すだけの非所有 address view を区別できないことが分かった。
- 一時的な heuristic で「exact owner を持たない raw alias」を非所有 view とみなすと、raw cell に格納した所有 address を enum / aggregate へ移す既存の owner transfer regression tests を壊した。
- `add buf 0` を明示的な view として lowering で残し、owner check で raw address view table を導入すると KP fixture は通せるが、これはまだ IR の正式な型/効果として設計された view ではない。
- `LocalScanner` の元実装に近い loop/realloc 構造では、`realloc_raw` が 0 を返す失敗経路で旧 allocation が引き続き live であることを Resource IR が表現しにくい。

## 問題

Resource IR currently represents owning raw addresses and non-owning address views through the same i32 alias mechanism. A plain read of an owning raw address and an address view such as add buf 0 can become indistinguishable at owner-check time, while realloc_raw returning 0 cannot express that the old allocation remains live on the failure path.

## 影響

Self-host scanner and collection code must either hide ownership in raw cells or add ad hoc source rewrites. This weakens memory-safety verification and makes loop/realloc based buffers hard to check without false leaks or false moves.

## 修正方針

Add an explicit Resource IR concept for raw address view/provenance separate from transferable raw ownership, and model fallible realloc as a Result-style operation or a branch-sensitive effect that preserves the old allocation on failure. Update core/mem APIs and stdlib users to avoid nullable raw ownership sentinels.

## 検証

Add Resource IR tests for iovec non-owning views, owning raw address stores into returned headers, and realloc failure branches preserving the old buffer owner without leaks or double frees.
