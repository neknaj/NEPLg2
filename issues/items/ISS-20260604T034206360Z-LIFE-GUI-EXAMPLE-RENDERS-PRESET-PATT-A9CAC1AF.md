---
id: ISS-20260604T034206360Z-LIFE-GUI-EXAMPLE-RENDERS-PRESET-PATT-A9CAC1AF
title: "Life GUI example renders preset patterns instead of a pure Conway board model"
area: examples
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: examples/gui_life.nepl
---

# ISS-20260604T034206360Z-LIFE-GUI-EXAMPLE-RENDERS-PRESET-PATT-A9CAC1AF: Life GUI example renders preset patterns instead of a pure Conway board model

## 概要

Subagent audit found gui_life.nepl modeling step/animate/cell_size while glider and blinker are drawn directly. The Life rules are not represented as pure Model + Event -> Model update logic. This conflicts with the Elm Architecture direction and Zenn pure core / explicit state guidance.

## 対象

- `examples/gui_life.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found gui_life.nepl modeling step/animate/cell_size while glider and blinker are drawn directly. The Life rules are not represented as pure Model + Event -> Model update logic. This conflicts with the Elm Architecture direction and Zenn pure core / explicit state guidance.

## 影響

Next Step and Animate cannot be trusted as tests of the actual Game of Life algorithm, and the example cannot scale to arbitrary board sizes or HD cell layouts without rewriting the model.

## 修正方針

Introduce a bounded or alloc-backed board model, implement neighbor count and next generation as pure functions, and make UI actions dispatch through typed events.

## 検証

Add still-life, blinker, glider, boundary-cell, interactive size, Next, Animate, and HD cell-size regular tests.
