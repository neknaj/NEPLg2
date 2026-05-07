# レビュー方法

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

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
- `Get-ChildItem nepl-core/src -File`
- `Get-Content nepl-core/src/{lexer,parser,loader,module_graph,resolve,target_gate,target_precheck,layout,monomorphize,codegen_wasm,codegen_llvm,wasm_shared,runtime_helpers}.rs`
- `rg -n "panic!|unwrap\\(|expect\\(" nepl-core/src/...`

## GitHub Actions 確認方針

レビュー上の test 状況は local test ではなく GitHub Actions の結果を根拠にする。現在の latest run は `d2ba8b8b` の CI run で、Rust compiler rest checkpoint 作成時点では pending である。CI の最終状態は、レビュー進行中に再確認して `project/actions-status.md` と最終 summary へ反映する。

`3742a1a7` で `--check` ResourceIR gate の regression が追加されたため、関連レビュー文書は同 commit を基準に更新した。Actions の最新 run はまだ pending であり、green 判定は後続 checkpoint で引き続き確認する。

## レビュー判断基準

- 技術的負債を残さない。
- 後方互換より正しい設計を優先する。
- 暫定実装は許容しても、暫定の雑設計は禁止する。
- 設計ミスが発覚した場合は、継ぎ足しではなく再設計再実装を選ぶ。
- 型安全とメモリ安全は必達とし、静的検査が効くデータ構造と pass 境界にする。
- 数値や文字列 sentinel ではなく enum / Option / typed wrapper を使う。
- 分岐は wildcard で握り潰さず、`match` の網羅性検査を活用する。

## 未完了

- selfhost、stdlib、quality、tools の個別レビュー本文。
- CI run `3742a1a7` 以降の完了結果確認。
- レビュー全体の妥当性再確認。
- 前回レビューとの差分報告。
