# CLI レビュー

作成日: 2026-04-25

対象: `nepl-cli/src/**`, `nodesrc/**` の CLI / テスト実行系

## レビュー範囲

| 区分 | ファイル |
|---|---|
| Rust CLI | `nepl-cli/src/main.rs`, `nepl-cli/src/codegen_llvm.rs` |
| Node CLI | `nodesrc/cli.js`, `nodesrc/tests.js`, `nodesrc/run_test.js`, `nodesrc/run_doctest.js` |
| Node parser / docs | `nodesrc/parser.js`, `nodesrc/parser.ts`, `nodesrc/html_gen.js`, `nodesrc/html_gen.ts` |
| playground tests | `nodesrc/playground_*_test_runner.js`, `nodesrc/tui_regression.js` |

## 総評

Rust CLI は compiler 実行、WASI runtime、test runner、WAT/LLVM 出力、診断表示を 1 ファイルに抱えており、core と同様に責務が集中しています。特に `--check` が実際の compile を行わない問題は、ユーザーに誤った成功を返すため最優先で修正が必要です。

Node 側は doctest 実行の中心になっていますが、HTML 生成 CLI、playground test、compiler artifact 探索が同居しています。テストの信頼性を上げるには、Rust CLI の `test` サブコマンドと Node doctest runner の役割を整理する必要があります。

## RV-CLI-001: --check がコンパイルせず成功を返す

- 解決済: true
- 状態: verified
- 優先度: P0
- 種別: bug
- 対象: `nepl-cli/src/main.rs`

### 根拠

- `nepl-cli/src/main.rs:304`: `if cli.check { eprintln!("Check successful"); return Ok(()); }`
- この分岐は `compile_module_with_source_map` 呼び出しより前にある。

### 問題

`--check` が loader の parse 成功後、typecheck / monomorphize / move check / codegen precheck を一切実行せず成功を返します。型エラーや未定義関数があっても `Check successful` になります。

### 影響

CI やユーザーが `--check` を信用できません。コンパイル不能なコードが成功扱いになるため、最も危険な CLI バグです。

### 修正方針

`--check` では target/profile を確定したうえで `prepare_module_for_codegen_with_source_map` 相当まで実行します。wasm bytes は出さず、diagnostics が error を含む場合は exit code 1 にします。

### 対応結果

`nepl-cli/src/main.rs` の `cli.check` 分岐を `compile_module_with_source_map` 実行後へ移動しました。これにより loader だけでなく typecheck / monomorphize / move check / codegen precheck を通過した場合だけ `Check successful` を返します。

### 検証

`nepl-cli/src/main.rs` に `check_runs_compiler_diagnostics` を追加し、未定義シンボルを含む入力で `nepl-cli --check -i file` 相当が失敗することを確認します。

確認済み:

- `cargo test -p nepl-cli check_runs_compiler_diagnostics`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` (`caseCount=13`, `passedCount=13`, `failedCount=0`)

## RV-CLI-002: 通常実行で DEBUG ログが出力される

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-cli/src/main.rs`

### 根拠

- `nepl-cli/src/main.rs:249`: `DEBUG: Creating Loader...`
- `nepl-cli/src/main.rs:251`: `DEBUG: Loader created...`
- `nepl-cli/src/main.rs:345`: `DEBUG: Calling compile_module`
- `nepl-cli/src/main.rs:348`: `DEBUG: compile_module returned Ok`
- `nepl-cli/src/main.rs:462`: test 実行中に `[nepl-cli] run_test_file...` を stdout へ出す。

### 問題

`--verbose` なしでも debug log が stderr/stdout に出ます。プログラム実行出力や JSON wrapper と混ざると、テスト比較や外部ツール連携を壊します。

### 影響

doctest の stdout/stderr 比較が不安定になります。CLI を他ツールから呼ぶと、機械処理できない余分な出力が混入します。

### 修正方針

全 debug output を `cli.verbose` gate の下に移します。test progress は human mode と JSON mode を分け、出力先を統一します。

### 検証

正常 compile/run の stderr が空であることを fixture 化します。

## RV-CLI-003: nepl-cli test が n.md doctest を対象にしない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: test
- 対象: `nepl-cli/src/main.rs`, `nodesrc/tests.js`

### 根拠

- `nepl-cli/src/main.rs:499`: `collect_nepl_files` は `.nepl` だけを集める。
- `nodesrc/tests.js:1`: Node runner は `/tests/compiler/*.n.md`, `/tests/stdlib/*.n.md`, `/tutorials/**/*.n.md`, `/stdlib/**/*.nepl` を対象にする設計。
- リポジトリの stdlib test は `stdlib/tests/*.n.md` と `tests/stdlib/*.n.md` が中心。

### 問題

Rust CLI の `test` サブコマンドと実際の標準テスト形式が一致していません。`nepl-cli test` は主要 doctest を拾えないため、テストコマンドとして信頼できません。

### 影響

開発者が `nepl-cli test` を実行しても、実際の回帰テストを通したことになりません。AGENTS.md の「nodesrc/cli.js のテストを実行し output json を確認」とも分断されています。

### 修正方針

Rust CLI の test サブコマンドを Node doctest runner と統合するか、廃止して公式テストコマンドを `nodesrc/tests.js` に一本化します。少なくとも `.n.md` doctest を parse して実行できるようにします。

### 検証

`stdlib/tests/*.n.md` の件数が Rust CLI test と Node runner で一致することを確認します。

## RV-CLI-004: WASI fd_write が stdout 専用で stderr を扱えない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `nepl-cli/src/main.rs`

### 根拠

- `nepl-cli/src/main.rs:1189`: コメントが `Minimal wasi fd_write implementation for stdout (fd 1)`。
- `nepl-cli/src/main.rs:1199`: `if fd != 1 { return 8; }`

### 問題

WASI の `fd_write` は stdout(fd=1) だけを許可し、stderr(fd=2) を `badf` 扱いにします。stdlib や将来の診断出力が stderr を使うと失敗します。

### 影響

WASI 互換性が不足し、標準エラーを使うプログラムが正常に動きません。テストで `stderr` を期待するケースも CLI runtime では扱いにくくなります。

### 修正方針

fd 1 と fd 2 を分けて host stdout/stderr へ書きます。stdout buffering と stderr immediate flush の方針を明記し、`nwritten` は両方で正しく返します。

### 検証

WASI program から fd 2 へ出力する fixture を追加し、stderr 比較を固定します。

## RV-CLI-005: path_open が WASI の preopen モデルを実装していない

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: security
- 対象: `nepl-cli/src/main.rs`, `stdlib/std/fs.nepl`

### 根拠

- `nepl-cli/src/main.rs:919`: `path_open` host function。
- `nepl-cli/src/main.rs:945`: path bytes を host path として直接 `fs::read(path)`。
- `stdlib/std/fs.nepl:160`: `dirfd` は 3 preopen を仮定するとコメントしている。

### 問題

WASI の preopen directory、rights、flags、relative path 解決を実装せず、guest 文字列を host path として直接読んでいます。`dirfd`, `rights_base`, `oflags` も実質無視されています。

### 影響

WASI 互換性だけでなく、sandbox として危険です。テストが host working directory に依存し、将来の file API の挙動が実環境で変わります。

### 修正方針

preopen table を `AllocState` に持ち、`path_open` は dirfd と rights を検査して preopen root 内の canonical path のみ許可します。読み込み専用から始め、errno を WASI の値に合わせます。

### 検証

preopen 内ファイル読み込み、preopen 外 `..` 拒否、存在しない path の errno をテストします。

## RV-CLI-006: stdlib root がビルド時パスに固定されている

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: architecture
- 対象: `nepl-cli/src/main.rs`

### 根拠

- `nepl-cli/src/main.rs:1348`: `stdlib_root()` が `env!("CARGO_MANIFEST_DIR")/../stdlib` を canonicalize する。

### 問題

ビルドしたバイナリを別ディレクトリへ移動すると stdlib が解決できません。CLI オプションや環境変数による stdlib root override もありません。

### 影響

配布・CI・エディタ連携で壊れやすいです。workspace 内でしか動かない CLI になります。

### 修正方針

優先順を `--stdlib-root`、`NEPL_STDLIB_ROOT`、実行ファイル相対、ビルド時 fallback にします。診断には探索した候補を出します。

### 検証

temp dir に stdlib をコピーして `--stdlib-root` で compile する CLI テストを追加します。

## RV-CLI-007: LLVM toolchain 条件が既定で linux + clang 21.1.0 に固定される

- 解決済: false
- 状態: open
- 優先度: P2
- 種別: bug
- 対象: `nepl-cli/src/codegen_llvm.rs`

### 根拠

- `nepl-cli/src/codegen_llvm.rs:13`: default clang は `"clang"`。
- `nepl-cli/src/codegen_llvm.rs:16`: 既定で `21.1.0` exact match。
- `nepl-cli/src/codegen_llvm.rs:25`: 既定で linux host を要求。

### 問題

LLVM target の既定値が非常に狭く、Windows / macOS / 別 clang minor version で失敗しやすいです。環境変数で回避できますが、通常利用の初期体験としては厳しすぎます。

### 影響

LLVM backend の検証が特定環境に偏ります。ユーザーの現在環境が Windows の場合、既定では利用不能です。

### 修正方針

既定は clang 実行可否と minimum feature check にし、exact version は CI 用 strict mode に分離します。host OS requirement も target triple option として明示します。

### 検証

strict mode と relaxed mode の CLI unit test を分けます。

## RV-CLI-008: nodesrc/cli が未知引数をエラーにしない

- 解決済: false
- 状態: open
- 優先度: P3
- 種別: test
- 対象: `nodesrc/cli.js`

### 根拠

- `nodesrc/cli.js:36`: 独自 `parseArgs`。
- `nodesrc/cli.js:43` 以降: 認識した引数だけを処理し、未知引数を default で無視する。

### 問題

`--playgroud-editor-tests` のような typo をしても usage error にならず、別モードとして実行される可能性があります。

### 影響

CI や手元確認でテストを実行したつもりが、実際には別処理になっていても検出できません。

### 修正方針

未知引数は即 error にします。`--help` だけは例外です。将来的には `commander` などに寄せてもよいですが、まずは現行 parser に default error を追加します。

### 検証

未知引数で exit code 2 になる Node CLI test を追加します。

## RV-CLI-009: wasm-bindgen-cli cache が rust-cache の後処理で壊れ CI bootstrap が落ちる

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: test
- 対象: `.github/actions/bootstrap-build/action.yml`, `.github/workflows/ci.yml`

### 根拠

- `.github/actions/bootstrap-build/action.yml:55`: `actions/cache@v4` で `~/.cargo/bin/wasm-bindgen` などを `wasm-bindgen-cli-Linux-X64-0.2.108` として cache している。
- `.github/actions/bootstrap-build/action.yml:67`: cache hit 時は `cargo install --locked wasm-bindgen-cli --version 0.2.108` を実行しない。
- `.github/actions/bootstrap-build/action.yml:83`: 直後に `wasm-bindgen --version` で存在確認している。
- `.github/actions/bootstrap-build/action.yml:88`: 同じ composite action の後段で `Swatinem/rust-cache@v2` を実行している。
- GitHub Actions run `24931603415`: `Shared bootstrap build` が `wasm-bindgen: command not found` により exit code 127 で失敗した。
- GitHub Actions run `24929865567`: build job で `wasm-bindgen-cli` を install した後、post step で `... Cleaning cargo/bin ...` が出てから `Cache saved with key: wasm-bindgen-cli-Linux-X64-0.2.108` が実行された。
- `gh cache list --repo neknaj/NEPLg2 --key wasm-bindgen-cli-Linux-X64-0.2.108` では該当 cache が `626 B` で、`wasm-bindgen` バイナリを含むサイズではなかった。

### 問題

`wasm-bindgen-cli` 専用 cache と `Swatinem/rust-cache` が同じ `~/.cargo/bin` を扱っています。GitHub Actions の post step は main step と逆順に実行されるため、後から定義された `Swatinem/rust-cache` の後処理が先に `~/.cargo/bin` を掃除し、その後で先に定義された `actions/cache` が wasm-bindgen 用 cache を保存します。

その結果、`wasm-bindgen-cli-Linux-X64-0.2.108` という正常そうな key に、バイナリを含まない空に近い cache が保存されます。次回以降は cache hit により install step がスキップされ、`wasm-bindgen --version` が `command not found` で失敗します。

### 影響

`build` job が `Shared bootstrap build` で止まるため、`bootstrap-build` artifact が upload されません。`compile-test` / `rust-test` / `wasi-test` / `stdlib-test` など build に依存する job は実行されず、`pages-final-bundle` と `pages-final-deploy` も artifact 不在で派生失敗します。結果として CI が本来のコンパイラ・stdlib 回帰を検証できません。

### 修正方針

`wasm-bindgen-cli` の cache を `Swatinem/rust-cache` の `~/.cargo/bin` cleaning と競合しない形に変更します。候補は次のいずれかです。

- `wasm-bindgen-cli` 専用 cache を廃止し、毎回 `cargo install --locked wasm-bindgen-cli --version 0.2.108` を実行する。
- `cargo install --root` で workspace 内または専用 directory に install し、その directory を cache して `GITHUB_PATH` へ追加する。
- cache hit 後も `command -v wasm-bindgen` と `wasm-bindgen --version` を検査し、壊れた cache の場合は再 install する。
- 既存 key を変えて、壊れた `wasm-bindgen-cli-Linux-X64-0.2.108` cache を再利用しない。

単に `Verify wasm-bindgen-cli` を消すのではなく、cache が壊れていても bootstrap が自己修復する構造にします。

### 検証

`gh run view <run-id> --log-failed` で `Shared bootstrap build` が `wasm-bindgen --version` を通過することを確認します。壊れた cache が残っている状態でも、再 install または専用 path への install により `trunk build --release --public-url /NEPLg2/` まで進むことを確認します。

## RV-CLI-010: Pages fast/final deploy が同じ github-pages artifact 名を使い final deploy が落ちる

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: test
- 対象: `.github/workflows/ci.yml`

### 根拠

- `.github/workflows/ci.yml:389`: `pages-fast-bundle` が `actions/upload-pages-artifact@v3` を既定設定で実行している。
- `.github/workflows/ci.yml:410`: `pages-fast-deploy` が `actions/deploy-pages@v4` を既定 artifact 名で実行している。
- `.github/workflows/ci.yml:412`: `pages-final-bundle` は `needs` を `always()` で受け、同じ push run 内で final Pages artifact を作る。
- `.github/workflows/ci.yml:496`: `pages-final-bundle` も `actions/upload-pages-artifact@v3` を既定設定で実行している。
- `.github/workflows/ci.yml:517`: `pages-final-deploy` も `actions/deploy-pages@v4` を既定 artifact 名で実行している。
- GitHub Actions run `24929865567`: build と final bundle は通ったが、`pages-final-deploy` が `Multiple artifacts named "github-pages" were unexpectedly found for this workflow run. Artifact count is 2.` で失敗した。

### 問題

`pages-fast-bundle` と `pages-final-bundle` が同じ workflow run 内で、どちらも既定名 `github-pages` の Pages artifact を upload します。`pages-fast-deploy` で pending site を先に出す設計自体は有効ですが、artifact は同じ run 内に残ります。

そのため `pages-final-deploy` が final site を deploy しようとすると、`actions/deploy-pages` が同名 `github-pages` artifact を 2 つ見つけて停止します。今回 run `24931603415` では build が先に失敗したため `No artifacts named "github-pages"` として見えていますが、bootstrap が直ると run `24929865567` と同じ同名 artifact 問題が再発する可能性があります。

### 影響

push CI で Pages の pending deploy と final deploy を同じ workflow run に入れている限り、final Pages 更新が安定しません。テスト結果 JSON を merge した final site を公開できず、CI 上の失敗も実テスト失敗なのか Pages 配布設計の失敗なのか分かりにくくなります。

### 修正方針

fast と final の Pages artifact 名を分離し、それぞれの deploy step が対応する artifact 名を明示して参照するようにします。`actions/upload-pages-artifact` / `actions/deploy-pages` の対応 version で artifact name 指定が可能かを確認し、可能なら `github-pages-fast` と `github-pages-final` のように分けます。

artifact 名分離ができない場合は、pending deploy と final deploy を別 workflow に分割するか、fast deploy を廃止して final deploy だけにするなど、同一 run 内に `github-pages` artifact が複数残らない構造へ変更します。

### 検証

bootstrap が成功する状態で push CI を実行し、`pages-fast-deploy` と `pages-final-deploy` がそれぞれ意図した artifact を deploy することを確認します。`gh run view <run-id> --log-failed` で `Multiple artifacts named "github-pages"` が出ないことを確認します。
