# 進捗確認及び総レビュー: 目次

対象 commit: `f108cebd`

各レビュー文書には、文書ごとに確認対象 commit を明記する。レビュー中に remote main の更新を取り込んだ場合は、影響を受ける章を更新し、更新後 commit を記録する。最終再レビュー後に入った変更は今回レビューの追従対象外とする。

## 0. レビュー運用

- `README.md`: レビュー基準、対象 commit、開発方針
- `index.md`: 本目次、レビュー順序、章ごとの作成予定ファイル
- `meta/review-method.md`: 調査方法、確認コマンド、判断基準
- `meta/review-validity.md`: 完了後の再レビュー、見落としリスク、妥当性確認

## 1. プロジェクト進捗と方針

- `project/progress.md`
  - `plan.md` と現行実装の差分
  - `note.n.md` に記録された 2026-04-30 の主要変更
  - `issues/index.md` の open issue / resolved issue 状況
  - README / doc の現行説明と実装のずれ
- `project/actions-status.md`
  - `gh` で確認した review 対象 commit の GitHub Actions 結果
  - 成功 job / 失敗 job / 失敗傾向
  - local test を review evidence に使わない運用の記録
- `project/risk-map.md`
  - selfhost 開始可否
  - 技術的負債の残存箇所
  - 型安全・メモリ安全の必達条件に対する blocker

## 2. Rust compiler: `nepl-core`

- `rust-compiler/overview.md`
  - crate 境界、pipeline、主要巨大ファイル、no_std 境界
- `rust-compiler/pipeline-diagnostics.md`
  - `compiler.rs`
  - `diagnostic_codes.rs`
  - diagnostic enum / stable string boundary
  - compiler error mapping と shadow report
- `rust-compiler/source-loader-module.md`
  - `source_map.rs`
  - `loader.rs`
  - `module_graph.rs`
  - import / stdlib resolution / file identity
- `rust-compiler/lexer-parser-ast.md`
  - `lexer.rs`
  - `parser.rs`
  - `ast.rs`
  - char literal、`#indent`、offside rule、block/argument layout
- `rust-compiler/resolve-typecheck.md`
  - `resolve.rs`
  - `types.rs`
  - `typecheck/*`
  - overload、trait capability、effect check、match exhaustiveness
- `rust-compiler/static-check-resource.md`
  - `passes/move_check/*`
  - `passes/drop_insertion.rs`
  - `resource/*`
  - Resource IR lowering、coverage、cell、owner、borrow、effect、drop authority
- `rust-compiler/monomorphize-hir-layout.md`
  - `hir.rs`
  - `monomorphize.rs`
  - `layout.rs`
  - type layout / HIR finalization / instance naming
- `rust-compiler/codegen.md`
  - `codegen_wasm.rs`
  - `codegen_llvm.rs`
  - `wasm_shared.rs`
  - `llvm_ir.rs`
  - runtime helpers、backend diagnostic、WASM/LLVM parity
- `rust-compiler/target-gates.md`
  - `target_gate.rs`
  - `target_precheck.rs`
  - raw body / target-specific constraints

## 3. Rust CLI、language tooling、web/editor

- `tools/cli.md`
  - `nepl-cli`
  - command modes、WASI runner、LLVM runner、exit code / stdout / stderr
- `tools/nodesrc.md`
  - `nodesrc/tests.js`
  - `nodesrc/run_doctest.js`
  - `nodesrc/run_test.js`
  - `nodesrc/run_source_policy_regressions.js`
  - `nodesrc/issues.js`
  - Discord report path
- `tools/language-lsp.md`
  - `nepl-language`
  - `nepl-lsp`
  - editor diagnostics、hover、analysis API
- `tools/web-playground.md`
  - `nepl-web`
  - `nepl-web-playground`
  - `web/`
  - playground editor tests、examples sync、deploy docs

## 4. Standard library: `stdlib`

- `stdlib/overview.md`
  - module map、dependency direction、doc-comment quality、stdlib tests
- `stdlib/core.md`
  - `core/math`
  - `core/cast`
  - `core/char`
  - `core/mem`
  - `core/option`
  - `core/result`
  - `core/test`
  - `core/traits/*`
- `stdlib/alloc-string-io.md`
  - `alloc/string`
  - `alloc/string/scanner`
  - `alloc/io`
  - ByteBuf / string / scanner / owner boundary
- `stdlib/alloc-collections.md`
  - `vec`
  - `vec/sort`
  - `list`
  - `stack`
  - `queue`
  - `deque`
  - `ringbuffer`
  - `binary_heap`
  - `btreemap`
  - `btreeset`
  - `hashmap`
  - `hashset`
  - `bitset`
  - `sparse_set`
  - `disjoint_set`
  - `fenwick`
  - `segment_tree`
  - `adjacency_matrix`
  - `bloom_filter`
  - `counting_bloom_filter`
- `stdlib/alloc-diag-json-hash.md`
  - `alloc/diag`
  - `alloc/encoding/json`
  - `alloc/hash/fnv1a32`
  - `alloc/hash/hash32`
  - `alloc/hash/sha256`
- `stdlib/std-io-fs-env.md`
  - `std/fs`
  - `std/io`
  - `std/iotarget`
  - `std/stdio`
  - `std/streamio`
  - `std/env/cliarg`
  - `std/text`
  - `std/test`
  - `std/prelude_base`
- `stdlib/nm-kp-platforms.md`
  - `nm/parser`
  - `nm/html_gen`
  - `kp/*`
  - `features/tui`
  - `platforms/wasix/tui`

## 5. Selfhost compiler: `stdlib/neplg2`

- `selfhost/overview.md`
  - selfhost S0-S7 進捗、現時点で実装開始できる範囲
- `selfhost/core-infra.md`
  - `core/infra/span`
  - `core/infra/text`
  - `core/infra/diag`
  - `core/infra/outcome`
- `selfhost/syntax-parser.md`
  - `core/syntax/token`
  - `core/syntax/lexer`
  - `core/syntax/ast/module_ast`
  - `core/syntax/parser/module_parser`
- `selfhost/module-resolve-typecheck.md`
  - `core/module/*`
  - `core/resolve/name_resolver`
  - `core/ty/ty`
  - `core/check/checker`
- `selfhost/hir-resource-mono-codegen.md`
  - `core/hir/hir`
  - `core/resource/move_state`
  - `core/mono/mono`
  - `core/codegen/wasm/binary`
  - `core/codegen/llvm/text`
  - `core/pipeline`
  - `core/options`
- `selfhost/cli.md`
  - `cli/args/*`
  - `cli/file_io`
  - `cli/reporter`
  - `cli/driver`
  - `cli/main`

## 6. NEPLg3 placeholder / migration docs

- `neplg3/status.md`
  - `stdlib/neplg3`
  - `doc/neplg3/spec`
  - `doc/neplg3/impl`
  - README の NEPLg2 / NEPLg3 説明の整合性

## 7. Tests、tutorial、examples

- `quality/tests.md`
  - `tests/compiler`
  - `tests/compiler/tree`
  - `tests/stdlib`
  - `.n.md` stdout report / exit code policy
  - source policy regression
- `quality/tutorials.md`
  - `tutorials/getting_started`
  - tutorial code examples と現行 stdlib API
  - current-style policy
- `quality/examples.md`
  - `examples`
  - README sample
  - web examples sync

## 8. 横断レビュー観点

- `crosscutting/static-safety.md`
  - enum / match / exhaustiveness
  - type safety / memory safety
  - Resource IR final authority 化
  - raw memory boundary
- `crosscutting/stdlib-selfhost-readiness.md`
  - string / mem / collections / hash / fs / stdio が selfhost に十分か
  - compiler と stdlib の責務境界
- `crosscutting/diagnostics-tests-docs.md`
  - diagnostic code parity
  - `.n.md` 共通 test 運用
  - docs と実装の同期

## 9. 最終まとめ

- `summary/findings.md`
  - 重要 findings
  - 追加すべき issue
  - 修正優先順位
- `summary/selfhost-readiness.md`
  - 今の段階で selfhost 実装を開始できる範囲
  - まだ開始すべきでない範囲
- `meta/review-validity.md`
  - このレビュー自体の再レビュー
  - 参照不足、検証不足、結論の妥当性
