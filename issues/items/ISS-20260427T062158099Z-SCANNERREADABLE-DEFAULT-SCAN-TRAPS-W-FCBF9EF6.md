---
id: ISS-20260427T062158099Z-SCANNERREADABLE-DEFAULT-SCAN-TRAPS-W-FCBF9EF6
title: "ScannerReadable default scan traps with unreachable for unsupported types"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: stdlib/std/streamio.nepl
source: ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB
---

# ISS-20260427T062158099Z-SCANNERREADABLE-DEFAULT-SCAN-TRAPS-W-FCBF9EF6: ScannerReadable default scan traps with unreachable for unsupported types

## 概要

std/streamio の ScannerReadable trait default method が unsupported type の scan を実行時 unreachable trap にしており、RV-STDLIB-010 の残債として unsafe helper source policy から除外せざるを得ない。

## 対象

- `stdlib/std/streamio.nepl`

## 根拠

- `rg` で stdlib 実装コードを再スキャンすると、core の unsafe helper 定義と core test 以外では `stdlib/std/streamio.nepl:1463` の `#intrinsic "unreachable"` だけが残る。
- `ScannerReadable::scan` は戻り値が `Self` で、generic default では失敗を `Result` として返せないため、未対応型が trait bound を満たして見えると runtime trap になる。

## 問題

public scanner API の unsupported type 経路が compile-time error や Result::Err ではなく unreachable trap になる。通常コードの unsafe helper 禁止を全体 source policy にすると、この default stub だけを allowlist する必要があり、再発監視の穴になる。

## 影響

self-host parser や CLI scanner が token/number 以外の読み取りを追加したとき、未実装型の scan が実行時 trap として現れ、入力エラーや API misuse を診断できない。RV-STDLIB-010 の完全な verified 化も妨げる。

## 修正方針

ScannerReadable を Result 返却の API へ移行するか、trait method default body を必須にしない compiler/language 側の対応を入れる。互換性を保つ場合は既存 read<T>(StreamScanner)->T を sentinel facade として残し、内部 trait は read_result/scan_result へ分離する。

## 検証

streamio source policy で implementation unreachable の allowlist をなくす。unsupported scanner type の compile_fail または Result::Err 回帰テストを追加し、既存 str/i32/i64/u32/u64/f32/f64 scanner doctest が通ることを確認する。
