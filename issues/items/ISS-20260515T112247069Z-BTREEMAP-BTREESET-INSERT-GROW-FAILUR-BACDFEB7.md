---
id: ISS-20260515T112247069Z-BTREEMAP-BTREESET-INSERT-GROW-FAILUR-BACDFEB7
title: "BTreeMap/BTreeSet insert grow failure drops collection owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/alloc/collections/btreemap/**, stdlib/alloc/collections/btreeset/**, nepl-core/tests/pipe_operator.rs"
---

# ISS-20260515T112247069Z-BTREEMAP-BTREESET-INSERT-GROW-FAILUR-BACDFEB7: BTreeMap/BTreeSet insert grow failure drops collection owner

## 概要

BTreeMap.insert and BTreeSet.insert are owner-consuming fallible updates, but their grow failure path returns Result<Collection, Diag> and btreemap_grow/btreeset_grow free or discard the input storage internally. Stage D requires fallible collection updates to return the consumed owner in the error payload so Resource IR can prove ownership from source-level types instead of trusting implementation discipline.

## 対象

- `stdlib/alloc/collections/btreemap/**, stdlib/alloc/collections/btreeset/**, nepl-core/tests/pipe_operator.rs`

## 根拠

- `BTreeMap` / `BTreeSet` の public `insert` は collection owner を消費するが、旧戻り型は `Result<Collection, Diag>` で、grow 失敗時に caller へ owner を返せなかった。
- `btreemap_grow` / `btreeset_grow` は旧 storage を内部で free してから `Diag` だけを返しており、owner の行方が API 型では証明できなかった。
- `Result` helper 経由の owner-bearing nested payload 伝播では ResourceIR の variant owner summary が曖昧化するため、owner-bearing Result は typed `Result::Ok` / `Result::Err` constructor で直接構築する必要がある。

## 問題

BTreeMap.insert and BTreeSet.insert are owner-consuming fallible updates, but their grow failure path returns Result<Collection, Diag> and btreemap_grow/btreeset_grow free or discard the input storage internally. Stage D requires fallible collection updates to return the consumed owner in the error payload so Resource IR can prove ownership from source-level types instead of trusting implementation discipline.

## 影響

A caller cannot recover, inspect, retry, or explicitly free the consumed BTreeMap/BTreeSet owner on grow allocation failure. This keeps collection memory safety dependent on hidden cleanup behavior and contradicts the static-check complexity reduction plan's owner-preserving update rule.

## 修正方針

Introduce BTreeMapInsertError<K,V> and BTreeSetInsertError<T> carrying the original collection owner and Diag. Change grow and public insert to return Result<Collection, InsertError>. Update doctests and source policy to require owner-preserving grow error handling and reject the old Diag-only contract.

## 検証

Run focused BTreeMap/BTreeSet doctests, pipe/btree cost doctests, source policy, issue checks, focused collection cleanup tests, and the Rust pipe regression that exercises BTreeMap insert through a nullary source call.

## 解決内容

- `BTreeMapInsertError<K,V>` / `BTreeSetInsertError<T>` を追加し、grow failure の `Err` payload に元の collection owner と `Diag` を保持するようにした。
- `btreemap_grow` / `btreeset_grow` は allocation failure 時に storage を内部破棄せず、元 owner を error payload として返す。
- public `insert` は `Result<Collection, InsertError>` を返し、成功/失敗どちらも typed `Result::Ok` / `Result::Err` で直接構築する。
- regression policy は owner-preserving error 型、owner を含む error struct、grow failure で内部 free しないこと、typed Result variant constructor を要求する。
- `nepl-core` の pipe 回帰は `BTreeMapInsertError` を明示し、作成した map owner を `free` する現在の ResourceIR 契約に同期した。
