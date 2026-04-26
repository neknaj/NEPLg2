---
id: ISS-20260426T165643523Z-NEPLG2-SELFHOST-LACKS-SOURCETEXT-LIN-815E29E4
title: "neplg2 selfhost lacks SourceText line map infrastructure"
area: selfhost
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/neplg2/core/infra/text.nepl, tests/stdlib/neplg2_text.n.md"
---

# ISS-20260426T165643523Z-NEPLG2-SELFHOST-LACKS-SOURCETEXT-LIN-815E29E4: neplg2 selfhost lacks SourceText line map infrastructure

## 概要

NEPLg2 self-host S1 has spans from the lexer but no shared SourceText/line map layer. Parser, diagnostics, and import/module stages would have to rescan source strings or invent their own byte-to-line conversions.

## 対象

- `stdlib/neplg2/core/infra/text.nepl, tests/stdlib/neplg2_text.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S1 は `infra/text.nepl` で byte offset、line/column、source file id を扱うことを求めている。
- 直前の lexer foundation は `SelfhostSourceSpan` を byte offset として返すが、offset から line / column へ変換する共有層がなかった。

## 問題

NEPLg2 self-host S1 has spans from the lexer but no shared SourceText/line map layer. Parser, diagnostics, and import/module stages would have to rescan source strings or invent their own byte-to-line conversions.

## 影響

Diagnostic labels, parser recovery, and module error reporting will drift from each other, and future self-host code will grow ad hoc text scanning helpers instead of a single core data structure.

## 修正方針

Add a pure core infra/text module that builds line starts from source text, exposes byte offset to line/column conversion, line span lookup, and source metadata helpers without depending on filesystem or stdio.

## 解決内容

- `stdlib/neplg2/core/infra/text.nepl` を追加し、`SelfhostSourceText`、`SelfhostSourceLocation`、line start table 構築、line count、line start lookup、line span lookup、offset to line/column 変換を実装した。
- LF / CRLF / CR を newline として扱い、diagnostic 用 line span は newline byte を含めない `[start, end)` を返す契約にした。
- allocation failure は `unwrap_ok` にせず `Result::Err StdErrorKind::OutOfMemory` として返すようにし、self-host core 側の input-dependent code が panic helper へ逃げない形にした。
- `tests/stdlib/neplg2_text.n.md` を追加し、LF / EOF boundary、CRLF span trimming、out-of-range offset / line を executable doctest で固定した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_text.n.md --no-tree -o tmp/neplg2-source-text-after-rebase.json -j 1`: 26/26 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-neplg2-source-text-after-rebase.json -j 4`: 412/412 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-neplg2-source-text-after-rebase.json -j 4`: 280/280 passed
- `trunk build`: passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-neplg2-source-text-after-rebase.json`: 13/13 passed
- `node nodesrc/issues.js check`: ok
- `git diff --check`: ok
