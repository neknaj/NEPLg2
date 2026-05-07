# NEPLg2 進捗確認及び総レビュー index

この index は、今回の総レビューで確認する対象と成果物の目次です。現時点では目次作成 checkpoint であり、個別領域の結論は各レビュー文書に記録します。

## 0. 進行管理

- `README.md`
  - レビュー方針、前回レビューの扱い、GitHub Actions 確認方針、checkpoint ルール。
- `meta/review-method.md`
  - 実際に確認したコマンド、参照範囲、レビュー中の同期履歴。
- `meta/review-validity.md`
  - 全レビュー完了後の再レビュー。見落とし、根拠不足、過去レビューへの影響、判断の一貫性を確認する。
- `summary/previous-review-diff.md`
  - 今回レビュー完了後にのみ作成する、前回レビューとの差分と進捗報告。

## 1. プロジェクト全体

- `project/progress.md`
  - 現行 HEAD、branch、主要開発領域、selfhost 開始可能性、進捗段階。
  - `plan.md`、`note.n.md`、`todo.md`、recent commit message との整合。
- `project/actions-status.md`
  - `gh` による GitHub Actions の最新結果確認。
  - CI failure がある場合の範囲、失敗 job、レビュー対象への影響。
- `project/risk-map.md`
  - 型安全、メモリ安全、静的検査、stdlib API、selfhost readiness の横断リスク。

## 2. Rust コンパイラ

- `rust-compiler/overview.md`
  - `nepl-core` と `nepl-cli` の責務境界、no_std 方針、CLI/WASI/LLVM/WASM の位置付け。
- `rust-compiler/lexer-parser-ast.md`
  - `lexer.rs`、`parser.rs`、`ast.rs`、`span.rs`、`source_map.rs`。
  - char literal、indent/offside、diagnostic recovery、doc comments、syntax tree tests。
- `rust-compiler/module-resolve-loader.md`
  - `loader.rs`、`module_graph.rs`、`resolve.rs`、import、prelude、VFS、cycle detection。
- `rust-compiler/typecheck.md`
  - `typecheck/`、`types.rs`、`hir.rs`。
  - generics、trait、overload、match、field access、function value、effect typing、diagnostic id。
- `rust-compiler/static-resource-check.md`
  - `resource/` 全体、ResourceIR lower、owner、borrow、initialized cell、raw memory、drop、effect summary。
  - メモリ安全と型安全の必達条件、enum/match による検査可能性、境界責務。
- `rust-compiler/drop-effect.md`
  - drop insertion/elaboration、drop point、drop requirement、effect identity/summary/counts。
- `rust-compiler/codegen-layout-targets.md`
  - `codegen_wasm.rs`、`codegen_llvm.rs`、`llvm_ir.rs`、`layout.rs`、`monomorphize.rs`、`target_gate.rs`、`target_precheck.rs`。
- `rust-compiler/diagnostics.md`
  - `diagnostic.rs`、`diagnostic_codes.rs`、compiler diagnostics redesign plan との整合。
- `rust-compiler/tests.md`
  - `nepl-core/tests`、`tests/compiler`、tree tests、source policy tests との対応。

## 3. selfhost コンパイラ (`stdlib/neplg2`)

- `selfhost/overview.md`
  - `stdlib/neplg2/README.md`、`index.n.md`、self_host_plan、self_host_execution_plan との整合。
- `selfhost/cli.md`
  - `cli/main.nepl`、`driver.nepl`、`file_io.nepl`、`reporter.nepl`、`args/`。
  - WASI CLI と core compiler の境界、exit code と diagnostics。
- `selfhost/infra.md`
  - `core/infra/{diag,outcome,span,text}.nepl`、`options.nepl`、`pipeline.nepl`。
  - diag id 設計、Outcome、Span/Text API、エラー集約。
- `selfhost/syntax.md`
  - `core/syntax/lexer.nepl`、`token.nepl`、`ast/module_ast.nepl`、`parser/module_parser.nepl`。
  - Rust parser parity、char/string、indent directive、match/enum 化。
- `selfhost/module-resolve.md`
  - `module/{graph,import_spec,loader,stdlib_map}.nepl`、`resolve/name_resolver.nepl`。
- `selfhost/typecheck-resource.md`
  - `ty/ty.nepl`、`check/checker.nepl`、`resource/move_state.nepl`。
  - 静的検査大規模修正との整合、型安全、メモリ安全、borrow/owner 追従。
- `selfhost/hir-mono-codegen.md`
  - `hir/hir.nepl`、`mono/mono.nepl`、`codegen/wasm/binary.nepl`、`codegen/llvm/text.nepl`。
- `selfhost/readiness.md`
  - selfhost 実装開始可能性、不足 stdlib、Rust 実装との parity、テスト移行方針。

## 4. NEPLg3

- `neplg3/status.md`
  - `stdlib/neplg3/cli`、`stdlib/neplg3/core`、`doc/neplg3/spec`、`doc/neplg3/impl`。
  - NEPLg3 compiler の位置付け、WASI CLI と core WASM の境界、仕様と実装の差分。

## 5. stdlib

- `stdlib/overview.md`
  - `stdlib/index.n.md`、facade、module split、コメント品質、stdlib test coverage。
- `stdlib/core.md`
  - `core/{char,mem,option,result,test,cast,field}.nepl`。
  - traits、math、rand、char/std string 連携、mem safety contract。
- `stdlib/math-traits.md`
  - `core/math/**`、`core/traits/**`。
  - numeric width、hash/ord/eq/serde/debug/stringify traits、enum/match 活用。
- `stdlib/alloc-string.md`
  - `alloc/string/**`、builder、storage、slice、search、split、integer/float、utf8、char offsets。
  - unsafe unwrap 排除、owner boundary、stdlib 不足による不自然なコードの解消状況。
- `stdlib/alloc-collections.md`
  - `alloc/collections/**`。
  - Vec、HashMap、HashSet、BTree、Deque、RingBuffer、Graph 系、Fenwick、SegmentTree、DisjointSet、Heap、Set/Map。
- `stdlib/alloc-hash-json-diag-io.md`
  - `alloc/hash/**`、`alloc/encoding/json/**`、`alloc/diag/**`、`alloc/io/**`。
- `stdlib/std-io-fs-env-test.md`
  - `std/{stdio,streamio,io,iotarget,fs,env,text,test,prelude_base}.nepl`。
  - ANSI/color output、debug profile、reader/writer/scanner、assert/test report。
- `stdlib/platforms-tui-kp-nm.md`
  - `features/tui.nepl`、`platforms/wasix/tui/**`、`kp/**`、`nm/**`。
  - nm parser/htmlgen split、TUI/ANSI design、競プロ向け API。
- `stdlib/tests.md`
  - `stdlib/tests` と `tests/stdlib` の範囲、n.md doctest、Rust/selfhost 共通化計画との整合。

## 6. テスト、examples、tutorial、docs

- `quality/tests.md`
  - `tests/compiler`、`tests/stdlib`、`stdlib/tests`、`nodesrc/test_*.js`、WASM/WASI runner。
- `quality/examples.md`
  - `examples/*.nepl`、`doc/examples/*.nepl`、RPN、stdio、nm、LLVM/WASM 実行例。
- `quality/tutorials.md`
  - `tutorials/`、`doc/neplg2/tutorial_rewrite_plan.md`、現行 NEPLg2 構文との差分。
- `quality/docs.md`
  - `doc/README.md`、`doc/neplg2/**`、`doc/compare/**`、`doc/testing.md`、`doc/cli.md`。

## 7. tools、web、editor

- `tools/nodesrc.md`
  - `nodesrc/cli.js`、test runner、doctest、issues、source policy regression、discord webhook。
- `tools/web-playground.md`
  - `nepl-web`、`nepl-web-playground`、`web`、playground editor tests。
- `tools/language-editor.md`
  - `nepl-language`、`nepl-lsp`、`editors`、analysis/hover/definition API。
- `tools/build-ci.md`
  - `Cargo.toml`、`trunk.toml`、`.github/workflows/ci.yml`、bootstrap action、build scripts。

## 8. 横断レビュー

- `crosscutting/static-safety.md`
  - 静的検査の設計、型安全、メモリ安全、ResourceIR/selfhost 追従。
- `crosscutting/diagnostics-tests-docs.md`
  - diag id、error reporting、n.md tests、assert/report、documentation contract。
- `crosscutting/stdlib-selfhost-readiness.md`
  - selfhost に必要な stdlib、文字列/collection/mem、I/O、hash、json、test API。

## 9. 最終成果物

- `summary/findings.md`
  - 重要度順の問題、根本原因、推奨修正、issue 化対象。
- `summary/selfhost-readiness.md`
  - 今の段階で selfhost 実装を開始できるか、開始条件、blocker、段階計画。
- `summary/progress-diff.md`
  - 今回レビューで確認した進捗と未解決領域。
- `summary/previous-review-diff.md`
  - レビュー完了後に前回レビュー内容を読んで作成する、前回との差分報告。

## checkpoint 進捗

- 目次作成: 完了。
- project status: 完了。
- Rust compiler 静的検査 / typecheck / diagnostics: 完了。
- Rust compiler lexer/parser / loader/resolve / drop/effect / codegen/tests: 完了。
- selfhost compiler: 完了。
- レビュー妥当性確認: 未着手。
- 前回との差分確認: 未着手。今回レビュー完了後まで前回レビュー内容は読まない。
