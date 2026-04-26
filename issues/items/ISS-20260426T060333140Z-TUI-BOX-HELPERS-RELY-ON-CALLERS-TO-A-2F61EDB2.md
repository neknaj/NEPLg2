---
id: ISS-20260426T060333140Z-TUI-BOX-HELPERS-RELY-ON-CALLERS-TO-A-2F61EDB2
title: "TUI box helpers rely on callers to avoid narrow column widths"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/platforms/wasix/tui.nepl
---

# ISS-20260426T060333140Z-TUI-BOX-HELPERS-RELY-ON-CALLERS-TO-A-2F61EDB2: TUI box helpers rely on callers to avoid narrow column widths

## 概要

line_box は cols - 2 を inner width として使い、コメントで cols < 2 は呼び出し側で回避するとしている。line_top と line_bottom も sub cols 2 を repeat_text に渡す。get_terminal_size は 0,0 を返し得るため、public TUI helper が narrow width を内部で安全に扱えていない。

## 対象

- `stdlib/platforms/wasix/tui.nepl`

## 根拠

- 未記入

## 問題

line_box は cols - 2 を inner width として使い、コメントで cols < 2 は呼び出し側で回避するとしている。line_top と line_bottom も sub cols 2 を repeat_text に渡す。get_terminal_size は 0,0 を返し得るため、public TUI helper が narrow width を内部で安全に扱えていない。

## 影響

TTY size 取得失敗や小さい viewport で、描画 width invariant が崩れたり、呼び出し側ごとに defensive branch が必要になる。TUI facade の reusable helper として扱いにくい。

## 修正方針

line_box、line_box_styled、line_top、line_bottom の cols <= 1/2 の契約を決め、clamp または Result による失敗表現に統一する。pure helper doctest で cols 0, 1, 2, 3 を固定する。

## 検証

tests/stdlib/features_tui.n.md または専用 doctest に narrow width cases を追加し、WASIX runner で通す。
