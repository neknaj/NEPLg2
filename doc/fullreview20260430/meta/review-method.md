# レビュー方法

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 目的

今回の総レビューでは、現行の NEPLg2 全体を一次情報から確認する。前回レビューの結論に引きずられないよう、レビュー本文の作成前と作成中は前回レビューの内容を参照しない。前回レビューは、今回レビュー完了後の差分報告で初めて内容を確認する。

## 前回レビューの扱い

- 確認したもの: `doc/fullreview20260430/` 配下のファイル名とディレクトリ構成。
- 確認しないもの: 前回レビュー本文の判断、findings、要約、結論。
- 以後の確認: レビュー完了前は `doc/fullreview20260430/` の旧版内容を比較対象として読まない。差分確認は `git diff --name-status` と `git diff --check` に限定する。
- 注意: README/index の置き換え後に通常の `git diff` を実行すると旧版削除行が出るため、以後は旧内容が本文判断に混ざらないよう、内容差分ではなく name/status と whitespace check を使う。

## 実施済み手順

1. `main` 上で `git pull --ff-only origin main` を実行し、開始時点の remote main を取り込んだ。
2. レビュー用 branch `review/fullreview-20260430-current` を作成した。
3. 前回レビューのファイル構成のみ確認した。
4. 現行ツリーのファイル一覧、主要ディレクトリ、`plan.md`、`note.n.md`、`todo.md`、issue index、recent commit message を確認した。
5. `doc/fullreview20260430/README.md` と `index.md` を目次 checkpoint として更新し、`97b07bad docs(review): refresh full review index` を main に push した。
6. project レビュー開始後、remote main に `545d2ab0 fix(resource): align region_ptr reference coverage` が入ったため、`review/fullreview-project-status` を `origin/main` に rebase した。
7. Rust compiler review 中に `nepl-cli --check` が ResourceIR gate を通らないことを確認し、`ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF` を追加した。
8. Rust compiler rest review 開始時に remote main の `cd44312f fix(resource): preserve region_ptr_at non-owning provenance` を取り込み、ResourceIR static review へ `region_ptr_at` regression を追記対象にした。
9. parser/backend/monomorphize の responsibility split 不足と public monomorphize panic API を確認し、`ISS-20260507T144627703Z-RUST-PARSER-AND-BACKEND-CODEGEN-LACK-11798587` と `ISS-20260507T144641729Z-PUBLIC-MONOMORPHIZE-API-PANICS-ON-UN-4492668C` を追加した。
10. Rust compiler rest review の push 前に remote main の `3742a1a7 fix(cli): run Resource IR gates for check-only` を取り込み、`--check` の ResourceIR gate 不足を fixed 扱いへ更新した。
11. selfhost review 開始時に remote main の `c58dd6e3 fix(monomorphize): return unresolved trait calls` を取り込み、public monomorphize API panic issue を resolved 扱いへ更新した。
12. selfhost review 中に `ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D` と `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B` を追加し、resolver sentinel も同 issue へ追記した。
13. selfhost review 文書 commit 前に remote main の `31291b37 fix(core): add parser backend responsibility policy` を取り込み、parser/backend policy 不足を fixed 扱いへ更新した。
14. stdlib review では `core/mem`、string、collections、hash/json/diag/io、std I/O/fs/env/test、platforms/TUI、nm、kp、source policy を現行ソースから確認した。
15. quality / tools / NEPLg3 review 中に remote main の `0fcc4839 fix(selfhost): compare model enums without numeric tags` を取り込み、selfhost enum equality issue を fixed 扱いへ更新した。
16. 同 review 中に `32e69bf4 docs(issues): track zed build artifacts`、`3951d807 docs(issues): track examples CI coverage gap`、`0ac34132 fix(selfhost): model builtin signatures by arity` を取り込み、Zed artifact issue、examples CI coverage issue、builtin signature arity enum 化をレビュー文書へ反映した。
17. `gh run view 25506281464 --json ...` で最新 main Actions を確認し、`tutorials-test` failure と run in_progress を `project/actions-status.md` へ記録した。
18. remote main の `4da7333 fix(selfhost): split type record payloads` を取り込み、selfhost type record payload 分離を fixed 扱いへ更新した。
19. `gh run view 25506711533 --json ...` で最新 main Actions を確認し、`4da7333` run が in_progress であることを `project/actions-status.md` へ記録した。
20. remote main の `6277239 fix(selfhost): split hir range payloads` を取り込み、selfhost HIR range payload 分離を fixed 扱いへ更新した。
21. `gh run list --branch main --limit 5` で `6277239` run `25507054306` が latest であることを確認し、`project/actions-status.md` へ記録した。
22. remote main の `b9e85f23 fix(selfhost): model mono instance absence with option` を取り込み、selfhost mono instance の未割当表現を `Option<SelfhostMonoInstanceId>` 化済みとしてレビュー文書へ反映した。
23. `gh run list --branch main --limit 8` と `gh run view 25507326678 --json status,conclusion,updatedAt,jobs` で latest Actions が pending であることを確認した。
24. remote main の `8ff05570 fix(selfhost): model hir expr id absence with option` を取り込み、HIR expr ID の未割当表現を `Option<SelfhostHirExprId>` 化済みとしてレビュー文書へ反映した。
25. latest Actions の `tutorials-test` failure を確認し、`ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153` と `ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9` を追加して main へ push した。
26. `gh run list --branch main --limit 8` で latest main run が `f3a4c60b` の `25507959628` pending であることを確認した。
27. remote main の `dc6b82bb fix(selfhost): model def id absence with option` を取り込み、resolver DefId の未割当表現を `Option<SelfhostDefId>` 化済みとしてレビュー文書へ反映した。
28. `gh run list --branch main --limit 5` で latest main run が `dc6b82bb` の `25508091075` in_progress であることを確認した。
29. remote main の `c5f93163 fix(selfhost): split hir expr payloads` を取り込み、HIR expression flat payload issue が resolved になったことを確認した。
30. `gh run list --branch main --limit 5` で latest main run が `c5f93163` の `25508600937` in_progress であることを確認した。

## 使用した確認コマンド

- `git status --short --branch`
- `git pull --ff-only origin main`
- `git fetch origin main`
- `git log --oneline --decorate -30`
- `git show --stat --oneline --name-only 545d2ab0`
- `git show --stat --oneline 3742a1a7`
- `rg --files`
- `Get-Content plan.md`
- `Get-Content note.n.md`
- `Get-Content todo.md`
- `Get-Content issues/index.json`
- `node -e "...issues/index.json..."`
- `gh run list --limit 12 --json ...`
- `gh run view <run-id> --json ...`
- `Get-Content nepl-core/src/compiler.rs`
- `Get-Content nepl-cli/src/main.rs`
- `Get-Content nepl-core/src/resource/mod.rs`
- `Get-Content nepl-core/src/resource/model.rs`
- `Get-Content nepl-core/src/typecheck/match_check.rs`
- `Get-Content nepl-core/src/diagnostic_codes.rs`
- `Get-Content nodesrc/test_resource_gate_order.js`
- `Get-Content nodesrc/test_static_check_boundary_responsibility.js`
- `Get-Content nodesrc/test_diagnostic_code_first_boundary.js`
- `node nodesrc/issues.js check`
- `Get-Content stdlib/neplg2/README.md`
- `Get-Content stdlib/neplg2/core/{pipeline,options}.nepl`
- `Get-Content stdlib/neplg2/core/infra/{diag,outcome,span,text}.nepl`
- `Get-Content stdlib/neplg2/core/syntax/{token,lexer}.nepl`
- `Get-Content stdlib/neplg2/core/syntax/parser/module_parser.nepl`
- `Get-Content stdlib/neplg2/core/module/{loader,import_spec,stdlib_map,graph}.nepl`
- `Get-Content stdlib/neplg2/core/resolve/name_resolver.nepl`
- `Get-Content stdlib/neplg2/core/{ty,hir,mono,builtins}/...`
- `Get-Content stdlib/neplg2/cli/**`
- `Get-Content nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`
- `Get-Content stdlib/core/{mem,char,result,option}.nepl`
- `Get-Content stdlib/core/math/**`
- `Get-Content stdlib/core/traits/**`
- `Get-Content stdlib/alloc/string/**`
- `Get-Content stdlib/alloc/collections/**`
- `Get-Content stdlib/alloc/{hash,encoding/json,diag,io}/**`
- `Get-Content stdlib/std/**`
- `Get-Content stdlib/platforms/wasix/tui/**`
- `Get-Content stdlib/nm/**`
- `Get-Content stdlib/kp/**`
- `Get-ChildItem nodesrc -Filter test_stdlib*.js`
- `Get-ChildItem nepl-core/src -File`
- `Get-Content nepl-core/src/{lexer,parser,loader,module_graph,resolve,target_gate,target_precheck,layout,monomorphize,codegen_wasm,codegen_llvm,wasm_shared,runtime_helpers}.rs`
- `rg -n "panic!|unwrap\\(|expect\\(" nepl-core/src/...`

## GitHub Actions 確認方針

レビュー上の test 状況は local test ではなく GitHub Actions の結果を根拠にする。現在の latest run は `c5f93163` の CI run `25508600937` で、quality/tools checkpoint 作成時点では in_progress である。CI の最終状態は、レビュー進行中に再確認して `project/actions-status.md` と最終 summary へ反映する。

`3742a1a7` で `--check` ResourceIR gate の regression が追加され、`c58dd6e3` で public monomorphize API panic が Result 化され、`31291b37` で parser/backend responsibility policy が追加された。`0fcc4839` で selfhost enum equality が direct match 化され、`0ac34132` で builtin signature が arity enum 化され、`4da7333` で type record payload が分離され、`6277239` で HIR range payload が分離され、`b9e85f23` で mono instance absence が Option 化され、`8ff05570` で HIR expr id absence が Option 化され、`dc6b82bb` で resolver DefId absence が Option 化され、`c5f93163` で HIR expression payload が variant enum 化された。Actions の最新 run `25508600937` はまだ in_progress であり、completed latest run を後続 checkpoint で確認する。

## レビュー判断基準

- 技術的負債を残さない。
- 後方互換より正しい設計を優先する。
- 暫定実装は許容しても、暫定の雑設計は禁止する。
- 設計ミスが発覚した場合は、継ぎ足しではなく再設計再実装を選ぶ。
- 型安全とメモリ安全は必達とし、静的検査が効くデータ構造と pass 境界にする。
- 数値や文字列 sentinel ではなく enum / Option / typed wrapper を使う。
- 分岐は wildcard で握り潰さず、`match` の網羅性検査を活用する。

## 未完了

- 横断レビュー本文。
- CI run `c5f93163` の完了結果確認。
- レビュー全体の妥当性再確認。
- 前回レビューとの差分報告。
