---
id: ISS-20260428T141156276Z-SELF-HOST-MODULE-LOADER-LACKS-TYPED--7277C7AB
title: "self-host module loader lacks typed import spec extraction"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/module/import_spec.nepl, stdlib/neplg2/core/syntax/ast/module_ast.nepl, tests/stdlib/neplg2_import_spec.n.md"
---

# ISS-20260428T141156276Z-SELF-HOST-MODULE-LOADER-LACKS-TYPED--7277C7AB: self-host module loader lacks typed import spec extraction

## 概要

self-host parser は `DirImport` module item を保持できるが、module loading S2 で使う typed import spec が無い。import graph や stdlib map が raw lexeme を各所で再解析する形になる。

## 対象

- `stdlib/neplg2/core/module/import_spec.nepl, stdlib/neplg2/core/syntax/ast/module_ast.nepl, tests/stdlib/neplg2_import_spec.n.md`

## 根拠

- `module_parser.nepl` は `#import "..." as ...` を `SelfhostModuleItemKind::ImportDirective` として AST item stream に残している。
- `core/module/loader.nepl` は VFS から `SelfhostModuleAst` まで到達できるようになったが、AST 内の import directive を typed data に変換する API がまだ無い。
- `doc/neplg2/self_host_plan.md` の S2 は import graph と stdlib path 解決を要求しており、その前段として import directive の path / alias / wildcard を一箇所で構造化する必要がある。

## 問題

import directive が raw string のままだと、loader、import graph、resolver がそれぞれ独自に `#import` の文字列を読むことになる。malformed directive の診断 code/span もばらつき、後続実装が parser の内部表現へ過剰に依存する。

## 影響

S2 が reliable な import graph を構築できず、wildcard import と alias import の違いも安定して扱えない。stdlib map や resolver へ進む前に、import directive の typed boundary を固定する必要がある。

## 修正方針

`core/module/import_spec.nepl` を追加し、`SelfhostModuleItemKind::ImportDirective` の lexeme を `SelfhostImportSpec` に変換する。path、alias、wildcard を typed field として保持し、malformed directive は `SelfhostDiagnostic` で返す。`SelfhostModuleAst` から import item だけを収集する helper も提供する。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree`
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree`
- `node nodesrc/issues.js check`

## 対応

- `stdlib/neplg2/core/module/import_spec.nepl` を追加し、`SelfhostImportSpec` に `span` / `path` / `alias` / `is_wildcard` を保持する typed boundary を作った。
- `selfhost_import_spec_parse_lexeme` / `selfhost_import_spec_parse_item` / `selfhost_module_import_specs` を追加し、raw `#import` lexeme の再解析を module 層の一箇所へ集約した。
- malformed directive は `selfhost.import.path_quote.expected` / `selfhost.import.path_quote.unclosed` / `selfhost.import.path.empty` / `selfhost.import.as.expected` / `selfhost.import.alias.expected` / `selfhost.import.trailing_text` の `SelfhostDiagnostic` として返すようにした。
- `SelfhostModuleItemKind` の分類は全 variant を明示的に列挙し、import item だけを収集する helper で後続 import graph / resolver が AST の raw item stream に過剰依存しないようにした。
- `tests/stdlib/neplg2_import_spec.n.md` を追加し、wildcard import、alias import、malformed directive の回帰を固定した。

## 検証結果

- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_spec.nepl --no-tree -o tmp/neplg2-import-spec-doctest.json -j 1`: total=1 passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/neplg2-import-spec-focused.json -j 1`: total=3 passed=3
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-import-spec-syntax.json -j 1`: total=45 passed=45
- remote main の `6e69db4 issues: track resource checker split` まで rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-import-spec-syntax-after-rebase.json -j 1`: total=45 passed=45
- remote main の `aff994b refactor(core): split resource cell state table` まで rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_import_spec.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-import-spec-syntax-after-rebase2.json -j 1`: total=45 passed=45
- `node nodesrc/issues.js check`: pass
- `git diff --check HEAD`: whitespace warning only for generated issue index line endings, no diff whitespace error
