---
id: ISS-20260426T060223863Z-BYTEBUF-CONVERSIONS-HIDE-ALLOCATION--3BF03711
title: "ByteBuf conversions hide allocation failure as empty values"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/io.nepl
---

# ISS-20260426T060223863Z-BYTEBUF-CONVERSIONS-HIDE-ALLOCATION--3BF03711: ByteBuf conversions hide allocation failure as empty values

## 概要

io_bytebuf_from_str は alloc_ptr 失敗時に io_bytebuf_empty を返し、io_bytebuf_to_str は string allocation 失敗時に ByteBuf を解放して空文字列を返す。どちらも API が Result を返さないため、空入力と allocation failure を呼び出し側が区別できない。

## 対象

- `stdlib/alloc/io.nepl`

## 根拠

- 未記入

## 問題

io_bytebuf_from_str は alloc_ptr 失敗時に io_bytebuf_empty を返し、io_bytebuf_to_str は string allocation 失敗時に ByteBuf を解放して空文字列を返す。どちらも API が Result を返さないため、空入力と allocation failure を呼び出し側が区別できない。

## 影響

binary I/O や self-host artifact generation で allocation failure が正常な空 payload として扱われ、出力欠落や破損を検出できない。Result を値として扱う stdlib 方針とも不整合になる。

## 修正方針

Result-returning variants を追加し、既存 helper は安全な wrapper に移行する。ByteBuf の所有権、失敗時解放、空入力を区別する doctest を追加する。

## 検証

alloc/io と streamio/io の ByteBuf conversion tests を追加し、空文字列と allocation failure 設計を分離して検証する。
