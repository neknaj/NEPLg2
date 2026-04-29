# Rust / selfhost 共通 `.n.md` テスト運用計画

作成日: 2026-04-29

## 目的

`.n.md` doctest を、Rust 実装と self-host 実装が同じ仕様ケースとして読む共通テスト資産にする。

NEPLg2.0 の仕様回帰は、Rust integration test と `.n.md` doctest に分散している。self-host compiler の各 stage が実装されると、同じ言語仕様を Rust と selfhost の両方で確認する必要がある。ここで fixture を別々に持つと、Rust 側の修正、diagnostic code redesign、selfhost parity の期待値がずれる。

この計画では `.n.md` を「ケース定義と外部期待値の正」にし、Rust runner と selfhost runner は同じ case manifest を consume する。後方互換のための二重形式は置かない。

## 現状調査

### 既存の `.n.md` 実行経路

- `nodesrc/tests.js` は `.n.md` / `.nepl` の `neplg2:test` を収集し、Rust-built web compiler bundle で compile/run する。
- `nodesrc/run_doctest.js` は 1 件の doctest を直接実行する focused reproduction 入口である。
- `nodesrc/run_test.js` は Rust compiler bundle を呼び出し、WASM / WASI / WASIX 実行、`ret:`、`stdout:`、`stderr:`、`compile_fail` を検査する。
- `nodesrc/analyze_source.js` は Rust compiler bundle の `analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics` を呼び、stage-level JSON を取得する。
- CI は `nodesrc/parser.js` / `nodesrc/html_gen.js` を TypeScript から生成して artifact に入れ、`nodesrc/tests.js -i tests` / `-i tutorials` / `-i stdlib` を走らせている。

### Rust integration test との関係

`nepl-core/tests/*.rs` は Rust API へ直接触る integration test で、次の用途に向く。

- Rust 内部構造や helper API の単体 regression。
- Web bundle に出ない内部 compiler API の検証。
- wasmi host import など、Rust harness 固有の実行環境の検証。

一方で、言語仕様、diagnostic code、source span、compile/run 結果のような外部 contract は `.n.md` に寄せるべきである。

### selfhost 側の現状

`stdlib/neplg2/` は self-host compiler の正規ソースツリーである。現時点では lexer / parser / module / CLI の一部があり、完全な compile/run backend は未完成である。

そのため selfhost 共通化は、いきなり `.n.md` を最後まで compile/run する形ではなく、stage ごとの parity backend として始める。

- S1: `.n.md` の source を Rust lexer と selfhost lexer に流し、token JSON / diagnostic code を比較する。
- S1: parser 実装後、Rust parse JSON と selfhost module AST JSON を共通正規化して比較する。
- S2: import / module graph の fixture を `.n.md` + sidecar virtual files として読み、Rust loader と selfhost loader を比較する。
- S3/S4: resolve / typecheck / resource の diagnostic code と selected trace JSON を比較する。
- S5: WASM artifact smoke と実行結果を比較する。
- S7: Rust-built selfhost compiler と selfhost-built compiler の artifact / diagnostic JSON / exit code を比較する。

## 発見した blocker

`ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79` を追加した。

`nodesrc/parser.ts` は `diag_code:` / `diag_codes:` を扱うが、実行時に Node が読む `nodesrc/parser.js` は古い `diag_id:` / `diag_ids:` 実装のままである。`nodesrc/tests.js` と `nodesrc/run_doctest.js` は `dt.diag_codes` を期待しているため、現状では `.n.md` の `diag_code:` が期待値として渡らない。

この issue は共通テスト運用の前提 blocker である。Rust/selfhost 共通化を始める前に、metadata parser の source-of-truth と生成物 drift を CI で固定する。

## stdout report と exit code

`.n.md` の assertion suite は、stdout に検査 report を出し、exit code は可否だけを表す形へ移行する。詳細計画は `nmd_assert_output_plan.md` に分離した。

現行 fixture では `main` の返す `i32` を `ret:` で検査する case が多いが、これは失敗時の詳細を `.n.md` の期待値として固定できない。Rust runner と selfhost runner が同じ case を読むには、assertion detail を stdout に出し、`exit_code:` で 0/1 を検査する。

`ret:` は言語仕様としての戻り値検証に限定する。`std/test` を使う assertion-style doctest は、`std/test` の report helper で deterministic stdout を出し、最後に report の exit code helper で 0/1 を返す。

`core` target は stdout を持たないため、この規則の対象から分ける。core-only の primitive semantics は `ret:`、trap、compile diagnostic、または将来の host report bridge で扱う。

## 設計方針

### 1. `.n.md` を case manifest の正にする

`.n.md` の `neplg2:test` block は、表示用ドキュメントではなく実行可能な case manifest として扱う。

case は次を持つ。

- `id`: `<path>::doctest#<index>` を既定の安定 ID とする。
- `source`: hidden line `|` を含めて実行用 source とする。
- `tags`: runner 選択と実行条件に使う。
- `stdin` / `argv`: runtime input。
- `ret` / `stdout` / `stderr`: runtime expectation。
- `exit_code`: process / WASI / selfhost CLI の終了可否 expectation。assertion suite では `ret` ではなくこちらを使う。
- `diag_code` / `diag_codes`: compile diagnostic の stable string expectation。
- `diag_span` / `diag_spans`: source span expectation。
- 将来追加: `stage`, `expect_json`, `compare`, `fixture_root` などの stage parity metadata。

旧 `diag_id` / `diag_ids` は受け付けない。診断の外部 contract は `DiagnosticCode::as_str()` / `selfhost_diag_code_name` の stable string に統一する。

### 2. 実行 backend だけを差し替える

共通 runner は「case manifest parser」と「execution backend」を分ける。

```text
.n.md / .nepl doc comment
        |
        v
DoctestCase manifest
        |
        +-- RustCompileRunBackend
        +-- RustStageJsonBackend
        +-- SelfhostStageJsonBackend
        +-- SelfhostCompileRunBackend
        +-- DualCompareBackend
```

Rust 側の現行 `nodesrc/tests.js` は当面 `RustCompileRunBackend` として扱う。selfhost 側は `SelfhostStageJsonBackend` から追加し、compile/run が完成してから `SelfhostCompileRunBackend` を入れる。

### 3. 期待値は外部 contract だけに置く

`.n.md` に直接固定する期待値は、仕様として外部へ見えるものに限定する。

- 許可: diagnostic code、diagnostic span、return value、stdout/stderr、CLI exit code、stable stage JSON。
- 非推奨: Rust 内部 enum debug 表示、allocator address、hash iteration order、未正規化 AST debug print。

stage JSON は比較用に正規化する。たとえば file path separator、source file id、span file id、hash map order は runner 側で canonical form へ変換する。

### 4. tag は runner 条件を enum 的に扱う

tag は文字列として metadata に現れるが、runner 内部では enum 相当の分類へ変換して扱う。

既存 tag:

- `compile_fail`
- `should_panic`
- `skip`
- `assert_io`
- `normalize_newlines`
- `strip_ansi`
- `trim_stdout`
- `llvm_cli`
- `llvm_only`
- `wasm_only`
- `wasi_only`
- `skip_wasm`
- `skip_llvm`

追加予定 tag:

- `rust_only`: selfhost が stage 未実装の場合ではなく、Rust 内部 API 固有の case に限定する。
- `selfhost_only`: selfhost 実装固有の regression に限定し、言語仕様 case には使わない。
- `stage_lex`
- `stage_parse`
- `stage_resolve`
- `stage_semantics`
- `stage_resource`
- `stage_codegen`
- `parity_json`

tag の意味は runner 側に散らさず、`DoctestTag` のような分類 module で集中管理する。

### 5. skip は期限付きにする

`skip` は selfhost 未実装の代替として乱用しない。selfhost が未実装の stage は runner matrix 側で「対象外」とし、case に恒久的な `skip` を置かない。

skip が必要な場合は次を明記する。

- 理由。
- 対象 runner。
- 対応 issue。
- 外す条件。

## 運用設計

### ディレクトリ運用

既存配置を維持する。

- `tests/compiler/*.n.md`: 言語仕様、診断、compiler pipeline。
- `tests/stdlib/*.n.md`: stdlib public behavior。
- `stdlib/**/*.nepl`: API 近傍の短い doctest。
- `tutorials/**/*.n.md`: 実行可能 tutorial。

selfhost 専用 fixture を無制限に増やさない。Rust と共有できる仕様 case は `tests/compiler` または `tests/stdlib` に置く。selfhost stage の内部表現だけを確認する case は `tests/selfhost/` を新設する前に、既存 `.n.md` に stage metadata を足して共有できないか確認する。

### Rust integration test から `.n.md` への移管基準

Rust integration test を削除するのではなく、用途で分ける。

`.n.md` へ移すもの:

- source text から始まる parse / resolve / type / effect / resource / backend behavior。
- diagnostic code / span を外部 contract として固定する case。
- stdout/stderr/return value で表現できる runtime behavior。

Rust integration test に残すもの:

- Rust API の内部不変条件。
- unit-level helper の boundary。
- wasm host import harness の低レベル検証。
- property test / fuzz-like test。

### selfhost parity runner の段階

#### P0: metadata parser 信頼性

- `parser.ts` / `parser.js` drift を解消する。
- `diag_code:` が `expected_diag_codes` へ渡る regression を追加する。
- `.n.md` case manifest の JSON dump command を用意する。

#### P1: Rust manifest runner の明確化

- `nodesrc/tests.js` の内部 case object を `DoctestCase` schema として文書化する。
- `run_doctest.js` と `tests.js` の expectation 適用差をなくす。
- `schema: neplg2-doctest/v1` を入力 manifest と出力 result の両方で扱える形にする。

#### P2: Rust stage JSON backend

- `nodesrc/analyze_source.js` 相当を `.n.md` case 単位で実行できる runner にする。
- `stage_lex` / `stage_parse` / `stage_resolve` / `stage_semantics` の期待 JSON を比較できるようにする。
- tree tests の個別 JS fixture は、安定したものから `.n.md` stage metadata へ移す。

#### P3: selfhost stage JSON backend

- selfhost lexer が stable token JSON を返す API を持つ。
- selfhost parser が stable module AST JSON を返す API を持つ。
- Rust stage JSON と selfhost stage JSON を canonicalize して比較する。
- diagnostic は stable string code で比較する。

#### P4: dual compile/run backend

- Rust compiler と selfhost compiler の両方で compile/run し、return value / stdout / stderr / diagnostic code を比較する。
- backend が未実装の target は matrix で対象外にし、case に `skip` を追加しない。

#### P5: bootstrap comparison

- `stdlib/neplg2/cli/main.nepl` を Rust compiler と selfhost compiler で compile する。
- artifact hash、diagnostic JSON、exit code、必要なら normalized WAT / LLVM IR を比較する。

## CI 設計

短期:

- 既存 `nmd-doctest` / `wasi-test` / `stdlib-test` を維持する。
- `parser.ts` と `parser.js` の drift check を追加する。
- `diag_code` metadata regression を追加する。

中期:

- `nmd-manifest` job を追加し、`.n.md` から manifest JSON を生成して artifact 化する。
- Rust compile/run job は manifest JSON を入力にする。
- stage parity job は同じ manifest JSON を入力にする。

長期:

- selfhost stage parity job を追加し、実装済み stage のみ matrix で実行する。
- selfhost compile/run job を追加し、Rust result と selfhost result の dual comparison を行う。

## 回帰検査方針

この運用自体の regression は、言語 fixture ではなく runner fixture で固定する。

- `diag_code:` が parser から case manifest へ残る。
- `diag_codes:` 配列が parser から case manifest へ残る。
- `run_doctest.js` と `tests.js` が同じ expectation logic を使う。
- `compile_fail` case が期待 code と違う code で fail になる。
- `stdout:` / `stderr:` / `ret:` の比較が Rust backend と selfhost backend で同じ規則になる。
- `exit_code:` が `ret:` と別の意味で検査され、assertion suite が stdout report なしの exit code だけに戻らない。
- `skip` / runner-specific tag の判定が matrix 側に閉じる。

## 実装順序

1. `ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79` を修正し、`diag_code` metadata を実際に検査できる状態にする。
2. `exit_code:` metadata と stdout assertion report 運用を追加する。
3. `DoctestCase` manifest schema を `nodesrc` の共通 module として切り出す。
4. `run_doctest.js` と `tests.js` の expectation 適用を共通化する。
5. `.n.md` manifest dump command を追加する。
6. Rust stage JSON runner を `.n.md` case 単位にする。
7. selfhost lexer parity backend を追加する。
8. selfhost parser / module / resolve / type / resource / codegen の順に backend を追加する。
9. CI に manifest / parity matrix を段階追加する。

## 進捗状況

- `nodesrc/tests.js`: Rust-built web compiler bundle による `.n.md` compile/run は稼働中。
- `nodesrc/run_doctest.js`: focused reproduction は稼働中。ただし expectation logic は `tests.js` と完全共通ではない。
- `nodesrc/parser.ts`: `diag_code` metadata 設計はある。
- `nodesrc/parser.js`: `diag_code` metadata に追従しておらず、修正 blocker。
- `nodesrc parser metadata`: `exit_code` metadata は未実装。
- `stdlib/std/test.nepl`: `Checks` と `checks_print_report` はあるが、assertion/report/exit code の責務境界を再設計する必要がある。
- `nepl-core/tests`: Rust integration test が多数あり、言語仕様 case と内部 API case が混在している。
- `stdlib/neplg2/core/syntax/lexer`: selfhost lexer は実装済み部分があり、最初の parity backend 候補。
- `stdlib/neplg2/core/syntax/parser`: module parser は実装済み部分があり、Rust parse JSON との正規化設計が必要。
- `stdlib/neplg2/core/module`: module loader / graph の parity fixture 候補がある。
- `stdlib/neplg2/core/check` 以降: selfhost 実装は未成熟で、共通 runner 設計を先に固定する段階。

## 完了条件

- `.n.md` に書いた 1 つの case を、Rust compile/run、Rust stage JSON、selfhost stage JSON、将来の selfhost compile/run が同じ manifest から実行できる。
- diagnostic expectation は stable string code だけを使い、数値 ID は残らない。
- runner 内部の分岐は enum 相当の分類を通し、自由文字列 tag 判定が各所に散らばらない。
- CI で metadata parser drift と expectation bypass が検出される。
- selfhost 実装が未完成の stage は matrix の対象外として扱われ、case 側に暫定 `skip` を増やさない。
