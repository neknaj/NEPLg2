---
id: ISS-20260426T060333140Z-TUI-BOX-HELPERS-RELY-ON-CALLERS-TO-A-2F61EDB2
title: "TUI box helpers rely on callers to avoid narrow column widths"
area: stdlib
status: verified
resolved: true
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

- `tests/stdlib/features_tui.n.md` に `cols` 0, 1, 2, 3 の回帰テストを追加したところ、旧実装では `line_box "abc" 3` が `│abc│` を返し、指定幅を超えていた。
- 調査中に `line_clip_to_cols` が超過時に `i = n` としてから `str_slice s 0 i` を返しており、実質的に切り詰めていないことも確認した。

## 問題

line_box は cols - 2 を inner width として使い、コメントで cols < 2 は呼び出し側で回避するとしている。line_top と line_bottom も sub cols 2 を repeat_text に渡す。get_terminal_size は 0,0 を返し得るため、public TUI helper が narrow width を内部で安全に扱えていない。

## 影響

TTY size 取得失敗や小さい viewport で、描画 width invariant が崩れたり、呼び出し側ごとに defensive branch が必要になる。TUI facade の reusable helper として扱いにくい。

## 修正方針

line_box、line_box_styled、line_top、line_bottom の cols <= 1/2 の契約を決め、clamp または Result による失敗表現に統一する。pure helper doctest で cols 0, 1, 2, 3 を固定する。

## 検証

tests/stdlib/features_tui.n.md または専用 doctest に narrow width cases を追加し、WASIX runner で通す。

## 対応

- `box_inner_cols` で box helper の内側幅計算を一元化し、`cols <= 2` の内側幅を 0 に丸めた。
- `line_box_body` を追加し、本文を内側幅へ `line_clip_to_cols` + `line_pad_to_cols` で揃えるようにした。
- `line_box` / `line_box_styled` / `line_top` / `line_bottom` が `cols <= 0` では空文字列、`cols == 1` では左側罫線 1 文字、`cols >= 2` では指定幅に収まる行を返す契約にした。
- `line_clip_to_cols` が超過時に末尾全体を返してしまう問題を修正し、表示幅を超える直前の byte index で切り出すようにした。
- `tests/stdlib/features_tui.n.md` に narrow width と長い本文の回帰テストを追加した。

## 検証結果

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-narrow-width.json -j 1`: `total=4`, `passed=4`
- `node nodesrc/tests.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-narrow-width-focused-files.json -j 1`: `total=5`, `passed=5`
- `node nodesrc/tests.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md --with-tree --no-stdlib -o tmp/tui-narrow-width-tree.json -j 1`: `total=25`, `passed=25`
- `node nodesrc/issues.js index` / `node nodesrc/issues.js check`: pass
- `cargo fmt --all --check`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tui-narrow-width.json`: `13/13 passed`
- `git diff --check`: pass（issue file/index の CRLF warning のみ）
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/tui-narrow-width-stdlib-full.json -j 4`: timeout after 304s, `partial=true`, `completed_results=0`。focused tests は通過しており、full stdlib は別途再計測対象。
