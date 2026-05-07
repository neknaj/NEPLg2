---
id: ISS-20260507T173212101Z-QUALIFIED-IMPORT-ALIASES-IGNORE-PUB--6A7D10D7
title: "qualified import aliases ignore pub open reexports"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "nepl-core/src/resolve.rs, nepl-core/src/typecheck/name_lookup.rs, nepl-core/tests/import_clause.rs"
---

# ISS-20260507T173212101Z-QUALIFIED-IMPORT-ALIASES-IGNORE-PUB--6A7D10D7: qualified import aliases ignore pub open reexports

## 概要

Qualified alias imports such as `#import "facade" as facade` kept only direct target files. If the target module publicly reexported implementation files with `pub #import ... as *`, `facade::member` could not resolve the reexported member unless the facade was rewritten as `@merge`.

## 対象

- `nepl-core/src/resolve.rs, nepl-core/src/typecheck/name_lookup.rs, nepl-core/tests/import_clause.rs`

## 根拠

- `doc/examples/07_modules.nepl` documents `use core::math as m` style qualified module access.
- `tutorials/getting_started/17_imports_and_modules.n.md` uses `#import "core/math" as math` and then `math::add` / `math::mul`.
- The current stdlib workaround changed math facades to `as @merge`, but the resolver still failed for an ordinary user facade that uses `pub #import "dep" as *`.
- `QualifiedImportTargets` was only `alias -> target file set`, so it could not represent public `as *` reexports or `as { foo as bar }` alias reexports.

## 問題

Qualified alias lookup collapsed an imported module into a flat file set and then searched for the requested member name in those files only. That representation cannot express the public API of a facade module: direct members, `pub as *` reexports, and selective alias reexports need different member-name mapping rules. Treating every facade as `@merge` hides this resolver model bug in stdlib layout instead of fixing the import semantics.

## 影響

Facade modules must be rewritten as merge-only workarounds for qualified access, and self-host/module code can lose public API visibility depending on file split style. This hides a resolver model bug behind stdlib layout changes.

## 修正方針

Represent qualified alias targets as per-file visibility rules, compose public reexport visibility through pub imports, and map requested qualified members through All/Selected visibility before binding lookup.

## 検証

Add import_clause regressions for pub open and pub selective reexports through qualified aliases; keep private transitive imports hidden; run focused tutorial doctests.

## 解決内容

`QualifiedImportTargets` を target file set から、target file ごとの `UnqualifiedImportVisibility` rule に変更した。alias import は direct target file を `All` として保持し、target module の `pub #import` から得られる public reexport visibility を合成する。

qualified lookup は requested member を visibility rule に照合してから binding name / target file を解決する。これにより、`facade::name` は direct target の同名定義、`pub #import ... as *` の公開再export、`pub #import ... as { foo as bar }` の alias 公開を同じ経路で扱う。非 `pub` の transitive open import は qualified alias から見えないまま維持した。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test import_clause -- --nocapture`: 13 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tutorials/getting_started/15_move_and_borrow.n.md -i tutorials/getting_started/17_imports_and_modules.n.md --no-tree -o tmp/agent1-getting-started-after-main-sync.json -j 1 --dist web/dist`: total=2, passed=2
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
