# `.n.md` stdout assertion report / stdlib assert 再設計計画

作成日: 2026-04-29

## 目的

`.n.md` doctest の assertion 結果を stdout に出し、exit code は可否だけを表す運用へ移行する。

現状は `main` が返す `i32` を runner が `ret:` として検査している。これは成功/失敗の可否を見るには使えるが、失敗時に「どの assertion が、どの expected/actual で落ちたか」を `.n.md` の期待値として固定できない。Rust compiler と selfhost compiler が同じ `.n.md` を読むためには、stdout に deterministic な report を出し、exit code は 0/1 のみを表す契約に分ける必要がある。

同時に、`stdlib/std/test.nepl` の assert API は assertion 評価、stdout 表示、集約、exit code 変換が混在しているため、stdout report 運用を支える基盤として再設計する。

## 現状調査

### `.n.md` runner

- `nodesrc/tests.js` は `.n.md` / `.nepl` から `neplg2:test` を集め、`expected_stdout`、`expected_stderr`、`expected_ret`、`expected_diag_codes` などを作る。
- `nodesrc/run_doctest.js` は focused reproduction 用に同様の expectation を組み立てる。
- `nodesrc/run_test.js` は Rust-built compiler bundle と WASM/WASI/WASIX 実行結果を JSON として返し、`return_value`、`stdout`、`stderr`、compile diagnostic を含める。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` は、`std/test` import があり stdout expectation がない case で `FAIL:` 行を検出する保険を持つ。ただしこれは成功時 report を仕様として固定するものではない。

### metadata parser drift

`nodesrc/parser.ts` は `diag_code:` / `diag_codes:` を扱うが、実行時に Node が読む `nodesrc/parser.js` は古い `diag_id:` / `diag_ids:` を扱っている。この drift は `ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79` として分離済みである。

stdout / exit code 運用の実装前に、metadata parser の source of truth と生成物 drift を CI で固定する。

### test fixture の分布

2026-04-29 時点で `tests` / `tutorials` / `stdlib` を調査した結果は次の通り。

- doctest 合計: 1481 件
- `ret:` を持つ doctest: 719 件
- stdout expectation を持つ doctest: 98 件
- stderr expectation を持つ doctest: 3 件
- `ret:` だけで stdout/stderr を持たない doctest: 710 件
- `std/test`、`checks_exit_code`、`assert_*` などの assertion 系 doctest: 227 件
- assertion 系で `ret:` だけの doctest: 116 件
- `checks_exit_code` 使用箇所: 171 件
- `checks_print_report` 使用箇所: 101 件

この数字から、assertion suite の一部は既に report を出す形に移行しているが、まだ exit code だけで詳細を固定しない case が多数残っている。

### `stdlib/std/test.nepl`

`std/test` は `Checks`、`check_*`、`assert_*`、`checks_print_report`、`checks_exit_code` を持つ。

- `check_*` は quiet に `Result<(),str>` を返す。
- `assert_*` は `check_*` を呼び、失敗時に `test_fail` 経由で `FAIL:` を stdout に出しつつ `Result<(),str>::Err` を返す。
- `checks_push` は `Result<(),str>` を集約する。
- `checks_print_report` は summary と human report を stdout に出す。
- `checks_exit_code` は `Checks` を 0/1 の `i32` へ変換するが、stdout は出さない。

このため、`checks_push assert_eq_i32 ...` のように使うと、集約前に stdout が出る。一方で `checks_exit_code checks` だけで終わると stdout report は出ない。API が「検査を値として作る」「表示する」「exit code にする」の境界を強制していない。

### `stdlib/core/test.nepl`

`core/test` は stdout を持たない core target の最小 helper で、失敗時は `unreachable` によって trap する。これは std の stdout report とは性質が違うため、同名の `assert` として扱うと混乱する。

core target は I/O を持たないので、stdout assertion report 必須の対象から分ける。core-only の primitive semantics は `ret:`、trap、compile diagnostic、または将来の host report bridge で扱う。

## 根本問題

### 1. `ret:` が二重の意味を持っている

`ret:` は本来、言語レベルで `main` が返す値を検証する expectation として自然である。一方で現在は、test helper が成功なら 0、失敗なら 1 を返す「exit code 相当」の確認にも使われている。

この二重性があると、`.n.md` manifest を Rust runner / selfhost runner / CLI runner の共通 contract にしたとき、`ret:` が process exit code なのか言語戻り値なのか分からなくなる。

### 2. assertion detail が stdout contract になっていない

exit code だけでは、失敗した assertion の label、kind、expected、actual、message が分からない。runner が `FAIL:` を見つける保険はあるが、これは失敗 detection であり、report format の互換性検査ではない。

### 3. assert API が副作用を持つ

`assert_*` は `Result` を返す関数でありながら、失敗時に stdout へ出力する。集約 API と組み合わせると出力の責務が二重化し、report の完全性を API で保証できない。

### 4. failure data が自由文字列に早期整形される

現在の failure は `str` の message として扱われる。これでは assertion kind、status、expected、actual を enum/struct として静的検査できず、selfhost 実装でも同じ形を安全に再現しにくい。

## 設計方針

### 1. `.n.md` に `exit_code:` を導入する

`.n.md` manifest は次の意味に分ける。

- `ret:`: 言語レベルの `main` return value を検査する。
- `exit_code:`: process / WASI / selfhost CLI の終了可否を検査する。
- `stdout:`: assertion report や program output を検査する。
- `stderr:`: diagnostic や CLI error output を検査する。

短期的には runner 内部で取得した `return_value` 相当値を、実行結果 schema の明示 field `exit_code` として出す case がある。ただし expectation logic は `exit_code:` を `return_value` へ fallback して検査してはならない。manifest 上の意味は分離し、後方互換のために `ret:` を exit code として使い続ける設計にはしない。

### 2. assertion suite は stdout report を必須にする

`std/test` を使う assertion-style doctest は、原則として次の形にする。

```neplg2
#entry main
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let report:
        test_report_new "case-name"
        |> test_report_push assert_eq_i32 "addition" 3 add 1 2
        |> test_report_push assert_str_eq "label" "ok" "ok"
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

`.n.md` metadata は次のようにする。

```text
neplg2:test
exit_code: 0
stdout: mlstr:
  test case-name
  ok 0 addition
  ok 1 label
  summary passed=2 failed=0
```

失敗 case では stdout に failure detail を出し、exit code は 1 にする。

```text
neplg2:test
exit_code: 1
stdout: mlstr:
  test case-name
  not ok 0 addition
    kind: eq_i32
    expected: 3
    actual: 4
  summary passed=0 failed=1
```

### 3. stdout format は deterministic にする

report は人間が読め、かつ `.n.md` で exact match しやすい安定形式にする。

- assertion 順は push 順。
- index は 0 始まりで固定。
- status は `ok` / `not ok` のような enum 由来の固定語にする。
- failure detail は kind、label、expected、actual、message を安定順で出す。
- ANSI color は canonical stdout report では使わない。端末向け color は別 helper に分ける。

### 4. `std/test` は 4 層に分ける

`std/test` は次の責務に分ける。

#### Assertion value

検査結果を構造化して返す層。

候補:

- `AssertionStatus::Passed`
- `AssertionStatus::Failed`
- `AssertionKind::Bool`
- `AssertionKind::EqI32`
- `AssertionKind::StrEq`
- `AssertionFailure`
- `TestAssertion`

`assert_*` は stdout を出さず、`TestAssertion` を返す。

#### Report aggregation

複数 assertion を集約し、count、passed、failed、assertion list を保持する層。

候補:

- `TestReport`
- `test_report_new`
- `test_report_push`
- `test_report_has_failure`

#### Rendering

stdout へ deterministic report を出す層。

候補:

- `test_report_to_text`
- `test_report_print_stdout`
- `test_report_print_human_color`

canonical `.n.md` は color なし stdout を使う。

#### Exit code

可否だけを 0/1 へ変換する層。

候補:

- `test_report_exit_code`
- `assertion_exit_code`

この層は stdout を出さない。

### 5. trap 型 assertion と report 型 assertion を分ける

`core/test` は stdout を持たないため、std の report 型 assertion と同じ名前にしない。

候補:

- core: `core_assert` / `core_assert_eq_i32` / `trap_assert_eq_i32`
- std: `assert_eq_i32` は `TestAssertion` を返す report 型

名前は最終実装時に stdlib 全体の import ergonomics を見て決めるが、trap と report の意味が同じ名前で混ざる設計は避ける。

### 6. enum と match で静的検査を効かせる

assertion kind、status、render mode、exit decision は enum で表現する。rendering は match で分岐し、新しい assertion kind を追加したときに report 生成の追加漏れが静的検査で見えるようにする。

自由文字列は外部表示用の label / message / expected text / actual text に限定する。

## `.n.md` runner 変更計画

### P0: metadata parser を修正する

- `diag_code:` / `diag_codes:` を `parser.js` へ反映する。
- `exit_code:` を `parser.ts` / generated `parser.js` の metadata に追加する。
- `parser.ts` と `parser.js` の drift check を CI に入れる。

### P1: expectation logic を共通化する

- `nodesrc/tests.js` と `nodesrc/run_doctest.js` の expectation 適用を共通 module に分ける。
- `expected_exit_code`、`expected_ret`、`expected_stdout`、`expected_stderr`、`expected_diag_codes` の意味を 1 箇所に閉じる。
- `ret:` と `exit_code:` の同時指定は、意味が違う case だけ許可する。assertion suite では `exit_code:` を使う。

### P2: assertion suite lint を追加する

`std/test` import または `test_report_exit_code` を含む doctest について、次を検出する。

- stdout expectation がない。
- `checks_exit_code` または `test_report_exit_code` だけで終わっている。
- `assert_*` が旧 printing API として使われている。
- `ret:` だけで exit code を代用している。

### P3: std/test canonical fixture を移行する

- `tests/stdlib/std_test_collect.n.md` を新 API の canonical fixture にする。
- 成功 report、失敗 report、複数 assertion、label、expected/actual 表示、exit code 0/1 を固定する。

### P4: 既存 doctest を移行する

- `checks_print_report` を既に使う fixture は stdout expectation を安定形式へ更新する。
- `checks_exit_code` だけの fixture は `test_report_print_stdout` + `exit_code:` へ更新する。
- tutorials は最新 API に合わせて全面的に書き直す。古い `ret:` exit code 代用は残さない。

### P5: selfhost runner と共有する

- selfhost stage runner は `.n.md` manifest の `stdout` / `stderr` / `exit_code` / `diag_code` を Rust runner と同じ schema で読む。
- selfhost compile/run backend が完成したら、Rust/selfhost dual comparison で stdout report と exit code を同時に比較する。

## stdlib assert 再設計計画

### S0: 既存 API の互換維持を前提にしない

開発方針として後方互換は不要である。`check_*` / `assert_*` / `checks_*` の名前をそのまま残すより、責務境界が明確な API へ再設計する。

ただし移行作業中に大量の fixture を一度に壊さないため、実装順序は段階化する。

### S1: data model を追加する

最初に構造化 data model を追加する。

- `AssertionStatus`
- `AssertionKind`
- `AssertionFailure`
- `TestAssertion`
- `TestReport`

この時点では旧 API を一時的に wrapper として残してもよいが、設計上の正は新 data model とする。

### S2: pure assertion API を作る

`assert_*` は stdout を出さず、`TestAssertion` を返す。

候補:

- `assert_true <(str,bool)->TestAssertion>`
- `assert_eq_i32 <(str,i32,i32)->TestAssertion>`
- `assert_str_eq <(str,str,str)->TestAssertion>`
- `assert_ok_i32 <(str,Result<i32,i32>)->TestAssertion>`
- `assert_err_i32 <(str,Result<i32,i32>)->TestAssertion>`

label を必須にし、failure report に必ず出す。

### S3: report API を作る

`TestReport` を作り、push 順で assertion を保持する。

- `test_report_new`
- `test_report_push`
- `test_report_count`
- `test_report_failed_count`
- `test_report_has_failure`

大量 assertion の O(n^2) 文字列連結は report rendering 時に閉じ込める。現行 `Checks` のように push 時点で summary/human string を作り続ける設計は避ける。

### S4: rendering API を作る

canonical stdout renderer を 1 つ定める。

- color なし
- stable line order
- stable escaping
- assertion kind は enum から match で出す
- expected/actual は type-specific formatter を通す

string escaping は `alloc/string` の再利用可能な escaping helper を使う。必要なら `std/test` 専用ではなく `alloc/string` 側に issue を追加する。

### S5: exit API を作る

`test_report_exit_code` は report の failed count だけを見て 0/1 を返す。stdout 副作用は持たない。

`.n.md` の canonical pattern は次の通り。

```neplg2
let shown test_report_print_stdout report
test_report_exit_code shown
```

### S6: 旧 API を削除する

移行が終わったら、旧 `Checks` / `check_*` / printing `assert_*` / `checks_print_machine` / `checks_print_human` / `result_exit_code` を削除または新 API に置き換える。

後方互換 wrapper を恒久的に残さない。

## 移行時の注意

- `core/test` は stdout report 設計の対象ではない。trap helper として明確に分離する。
- `.n.md` の `ret:` を一括で `exit_code:` に置換しない。言語戻り値を検証している case は `ret:` に残す。
- `std/test` を使う assertion suite は stdout report を期待値に固定する。
- compiler diagnostic case は stdout report ではなく `diag_code:` / `diag_span:` を使う。
- CLI diagnostic case は stderr / JSON stdout / exit code の split を case ごとに明確にする。

## 関連 issue

- `ISS-20260429T101413560Z-NODESRC-DOCTEST-PARSER-RUNTIME-IGNOR-6E5E5A79`: `diag_code:` metadata が runtime parser で無視される。
- `ISS-20260429T101530928Z-N-MD-SHARED-TEST-OPERATION-FOR-RUST--52938450`: Rust/selfhost 共通 `.n.md` 運用計画。
- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`: assertion 系 `.n.md` が stdout report ではなく return value に依存している。
- `ISS-20260429T102809685Z-STDLIB-ASSERT-API-MIXES-ASSERTION-RE-0F17011A`: `std/test` assert API の責務混在。

## 進捗状況

- `nodesrc/tests.js`: `.n.md` compile/run は稼働中。stdout/stderr/ret/diagnostic/exit_code expectation を持つ。expectation logic の共通化は残る。
- `nodesrc/run_doctest.js`: focused reproduction は稼働中。`diag_code:` / `diag_span:` / `exit_code:` expectation を検査する。expectation logic の共通化は残る。
- `nodesrc/parser.ts`: `diag_code:` と `exit_code:` を実装済み。
- `nodesrc/parser.js`: generated artifact として扱う。`npx tsc -p nodesrc/tsconfig.json` 後の runtime parser behavior を `nodesrc/test_doctest_diag_code_metadata.js` と `nodesrc/test_doctest_exit_code_metadata.js` で固定済み。
- `stdlib/std/test.nepl`: `Checks` と `checks_print_report` はあるが、assertion/report/exit code の責務境界が不十分。
- `stdlib/core/test.nepl`: core trap helper として存在するが、std report API と名前が近く、分離方針を明文化した段階。
- `tests/stdlib/std_test_collect.n.md`: 現行 `Checks` の report fixture はある。新 API の canonical fixture へ更新予定。
- `tutorials/getting_started`: `checks_exit_code` 中心の説明が多く、新 assert/report 設計に合わせて tutorial rewrite 対象。
- `stdlib/neplg2`: selfhost runner は未完成。`.n.md` manifest の stdout/exit_code schema を先に固定する。

## 2026-04-29 実装メモ

- `diag_code:` / `diag_codes:` の runtime parser drift と focused runner enforcement を修正した。
- `exit_code:` metadata を parser / focused runner / aggregate runner に追加した。
- `ret:` は言語戻り値、`exit_code:` は runner/process の終了可否として扱う schema に分離した。
- 既存 assertion suite の移行と `std/test` 再設計は継続作業である。

## 完了条件

- `.n.md` runner が `exit_code:` を読み、`ret:` と別の意味で検査する。
- `std/test` の canonical assertion API が structured value を返し、stdout は report renderer だけが出す。
- assertion suite の `.n.md` は stdout report と exit code を両方固定する。
- runner/lint が `std/test` assertion suite の stdout report 省略を検出する。
- Rust runner と selfhost runner が同じ manifest schema で stdout report / exit code / diagnostic code を比較できる。
