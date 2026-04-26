---
id: ISS-20260426T081514183Z-NM-RENDERER-DIVERGES-FROM-GLOSS-HTML-10480257
title: "nm renderer diverges from Gloss HTML and escape contract"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md"
---

# ISS-20260426T081514183Z-NM-RENDERER-DIVERGES-FROM-GLOSS-HTML-10480257: nm renderer diverges from Gloss HTML and escape contract

## 概要

The NEPL nm stdlib is intended to support the gloss/nm dialect, but the renderer outputs ruby without nm-ruby/rb markup, annotation gloss as generic span elements, section class nest instead of nm-sec, and HTML escaping omits apostrophe. Parser JSON escaping also keeps a hand-written finite decision tree that is easy to drift from the text escape contract.

## 対象

- `stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md`

## 根拠

- `https://github.com/neknaj/gloss` の `src-core/src/html.rs` と snapshot tests は Ruby を `nm-ruby` + `rb/rt`、Anno を `nm-anno` + `nm-anno-note`、section を `nm-sec level-N` として出力している。
- 既存 `stdlib/nm/html_gen.nepl` は Ruby に class/rb を付けず、Anno 相当の `Gloss` を generic span で出力し、section class も `nest` だった。
- 既存 `tests/stdlib/nm.n.md` は prefix/suffix だけを確認していたため、HTML contract のずれや段落末尾改行の混入を検出できなかった。

## 問題

The NEPL nm stdlib is intended to support the gloss/nm dialect, but the renderer outputs ruby without nm-ruby/rb markup, annotation gloss as generic span elements, section class nest instead of nm-sec, and HTML escaping omits apostrophe. Parser JSON escaping also keeps a hand-written finite decision tree that is easy to drift from the text escape contract.

## 影響

Generated HTML cannot be styled or compared with the reference Gloss output, and weak regression tests allow renderer/parser drift to recur.

## 修正方針

Align html_gen output with Gloss ruby/annotation/section markup, classify escape decisions through enums and match expressions, and add exact regression tests for ruby, annotation, section class, and escaping.

## 対応

- `stdlib/nm/html_gen.nepl` の Ruby 出力を `<ruby class="nm-ruby"><rb>...</rb><rt>...</rt></ruby>` に揃えた。
- `Gloss` inline の HTML を Gloss 仕様の Anno として `<ruby class="nm-anno">` / `nm-anno-note` へ変更し、main/sub segment の再帰 inline parse は維持した。
- section class を `nm-sec level-N` へ変更し、heading tag 選択と HTML escape を enum + match に整理した。
- HTML escape に apostrophe `&#39;` を追加した。
- `parse_paragraph` が段落末尾に不要な改行を残す問題を修正し、行間だけ `\n` を挿入するようにした。
- `document_to_json` 用 escape を enum + match に整理し、`\r` / `\t` / `\b` / `\f` も扱うようにした。
- `tests/stdlib/nm.n.md` を exact fixture に更新し、JSON escape、section HTML、Ruby/Anno markup、HTML escape の回帰を固定した。

## 検証

- `git fetch origin main`: `origin/main` is `4cf09c3`
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-gloss-focused-after-trunk.json -j 1`: `total=4`, `passed=4`, `failed=0`
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-gloss-suite-after-main-sync.json -j 1`: `total=9`, `passed=9`, `failed=0`
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-gloss-suite-final-crlf.json -j 1`: `total=9`, `passed=9`, `failed=0`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/nm-gloss-stdlib-full.json -j 4`: `total=404`, `passed=404`, `failed=0`
- `node nodesrc/issues.js check`: pass
- `git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check`: pass
