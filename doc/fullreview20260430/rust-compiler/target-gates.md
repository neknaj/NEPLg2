# Target gates review

対象 commit: `f108cebd`

## 概要

`target_gate.rs` と `target_precheck.rs` は WASM / WASI / LLVM / raw body の target-specific constraint を扱う。backend 到達前に target mismatch を diagnostic として出す役割を持つ。

## 良い点

- target multiple directive / unknown target は loader diagnostic code を持つ。
- backend target requires CLI などは backend diagnostic code に写像される。
- raw body target mismatch は effect/backend diagnostic と接続されている。

## 残る問題

- raw body / raw memory / target-specific intrinsic は safety policy と絡むため、単なる backend option として扱うと unsafe boundary が広がる。
- final design では raw body capability、unsafe memory internal boundary、target gate を明確に分けるべきである。

## selfhost への示唆

selfhost CLI options は target と profile を typed enum として core pipeline に渡す。target string を parser/driver/codegen の各所で比較しない。raw body や unsafe intrinsic は target gate と static safety gate の両方を通る設計にする。
