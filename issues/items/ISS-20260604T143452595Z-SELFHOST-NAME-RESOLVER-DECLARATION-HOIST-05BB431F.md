---
id: ISS-20260604T143452595Z-SELFHOST-NAME-RESOLVER-DECLARATION-HOIST-05BB431F
title: "selfhost name resolver should hoist declarations from module AST"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/resolve/name_resolver/hoist.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl, stdlib/neplg2/core/syntax/lexer/keyword.nepl, nodesrc/test_selfhost_name_resolver_declaration_hoist.js, tests/stdlib/neplg2_name_resolver.n.md, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260604T143452595Z-SELFHOST-NAME-RESOLVER-DECLARATION-HOIST-05BB431F: selfhost name resolver should hoist declarations from module AST

## 概要

S1 parser は top-level declaration header を typed evidence として `SelfhostModuleAst` に保持するようになったが、S3 name resolver はまだ手書きの scope binding smoke test だけを持ち、module AST から定義 scope を構成する入口を持たない。

## 対象

- `stdlib/neplg2/core/resolve/name_resolver/hoist.nepl`
- `stdlib/neplg2/core/resolve/name_resolver.nepl`
- `stdlib/neplg2/core/syntax/lexer/keyword.nepl`
- `nodesrc/test_selfhost_name_resolver_declaration_hoist.js`
- `tests/stdlib/neplg2_name_resolver.n.md`
- `tests/stdlib/neplg2_lexer.n.md`

## 根拠

- セルフホスト設計では parser が call tree を完全確定せず、module item / declaration header evidence を後続 stage へ渡す。
- 後続の HIR / checker は文字列の再走査ではなく、parser が保持した typed header evidence から定義候補を得る必要がある。
- 現状の `SelfhostNameScope` は binding table 単体の API であり、`SelfhostModuleAst` から `FunctionDecl` / `StructDecl` / `EnumDecl` / `TraitDecl` を scope へ hoist する root API がない。

## 問題

module parser と name resolver の接続がないため、selfhost compiler pipeline は parsed module から resolve scope を構築できない。実装が進むほど、各 stage が宣言 lexeme や header span を個別に解釈する危険がある。

## 影響

- declaration name の扱いが parser、checker、resolver の間で重複しやすい。
- `impl` のような名前束縛ではない declaration と、function / type / trait の名前束縛を enum で分ける境界が曖昧になる。
- HIR lowering が `SelfhostDefId` ではなく文字列参照へ戻りやすい。
- 実装中に、selfhost lexer が `trait` を 6 byte keyword bucket に置いていたため、`KwTrait` ではなく `Ident` として token 化していたことが分かった。この状態では parser / resolver が正しくても trait declaration は module AST へ入らない。

## 修正方針

`selfhost_name_scope_hoist_module_declarations(source, ast)` を追加し、module AST の declaration item を順に見て scope binding を構築する。名前束縛は declaration header の `Name` head だけから作り、`Impl` / `TypeLabel` head は初期段階では scope binding にしない。

`trait` は 5 byte keyword なので `lex_keyword_kind_len5` へ移し、function / struct / enum / trait / impl の declaration keyword が selfhost lexer で正しい `TokenKind` へ分類されることも回帰テストに含める。

## 検証

- source policy で hoist module が split list / facade に登録されていることを確認する。
- focused doctest で `fn main`、`struct Item`、`enum Choice`、`trait Show` が対応する `SelfhostDefKind` として scope に入ることを確認する。
- `impl .T` は名前 binding を増やさないことを確認する。
- lexer doctest で `trait` が `KwTrait` として分類されることを確認する。

## 解決内容

- `selfhost_name_scope_hoist_module_declarations` を追加し、parser の typed declaration header evidence から `SelfhostNameScope` を構築できるようにした。
- `Function` / `Struct` / `Enum` / `Trait` を `SelfhostDefKind` へ写し、`Impl` と `TypeLabel` head は名前 binding にしない規則を固定した。
- `trait` の keyword bucket を 5 byte bucket へ修正し、parser 以前に trait declaration が落ちる根本原因を解消した。
- source policy と doctest で、facade 登録、source list 登録、header span からの name slicing、Trait hoist、Impl 非binding化を固定した。

## 検証結果

- `node nodesrc\test_selfhost_name_resolver_declaration_hoist.js`
- `node nodesrc\test_selfhost_name_resolver_split_contract.js`
- `node nodesrc\test_selfhost_name_resolver_report_contract.js`
- `node nodesrc\tests.js -i tests\stdlib\neplg2_name_resolver.n.md --no-tree -o tmp\selfhost-name-resolver-hoist-focused.json -j 1 --assert-io --dist web\dist`
- `node nodesrc\tests.js -i tests\stdlib\neplg2_lexer.n.md --no-tree -o tmp\selfhost-lexer-focused.json -j 1 --assert-io --dist web\dist`
