# tutorials review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `tutorials/getting_started/*.n.md`
- `doc/neplg2/tutorial_rewrite_plan.md`
- `nodesrc/test_tutorial_getting_started_current_style.js`
- `.github/workflows/ci.yml`

## 良い点

`tutorials/getting_started` は現行 NEPLg2 に合わせた 30 章構成へ整理されている。Part 0 から Part 6 で実行環境、値、関数、`Option` / `Result`、string/byte/char、collection/ownership、module/generic/trait、実践 project を順に扱い、競技プログラミング向け内容は Advanced track に分離されている。

`12_char_and_ascii.n.md` は char literal、ASCII classifier、`str_char_at_result` を runnable example として示している。byte index と `char` 値を分ける説明もあり、string / char 連携の導線として妥当である。

`15_move_and_borrow.n.md` は use-after-move の compile_fail を固定し、読者向け tutorial が静的検査の重要な contract を直接見せる意図は良い。ただし現在の Actions では期待 diagnostic code が stale になっている可能性があり、`ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153` で追跡する。

`nodesrc/test_tutorial_getting_started_current_style.js` は章リスト、削除済み旧章、raw memory、`MemPtr`、panic helper、古い signature、char chapter、Copy-bound generic example を source policy として固定している。古い tutorial への後戻りを防ぐ仕組みがある。

CI に `tutorials-test` job があり、`node nodesrc/tests.js -i tutorials -o tutorials-tests.json -j 4` が実行される。tutorial は executable documentation として main branch gate に入っている。

直前の `b9e85f23` Actions run `25507326678` では `tutorials-test` と `nm-compile` が failure になったが、後続 push により古い run になった。最新 main の `c5f93163` run `25508600937` は確認時点で in_progress である。`25507326678` の `tutorials-test` log は run 完了後に取得可能になり、getting_started doctest 2 件と VFS cross-file tree test 2 件の failure を確認したため、それぞれ issue 化した。

## 問題とリスク

`doc/neplg2/nmd_assert_output_plan.md` は `std/test` の structured report API 実装済みを記録しているが、tutorial 本体はまだ `checks_exit_code` 中心の章がある。`ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` の残件として、tutorial の assertion example も stdout report + `exit_code` へ寄せる必要がある。

Advanced track は有用だが、競技プログラミング例が現在の stdlib collection/drop/memory 方針に継続追従できているかは、stdlib 変更のたびに確認が必要である。特に collection free/drop contract が未解決のため、non-Copy payload を扱う例を増やす前に stdlib 側の安全 contract を固めるべきである。

CI gate に入っている tutorial が赤い状態は、document quality だけでなく compiler/stdlib regression の早期検出を弱める。既存の assertion contract issue に含められる failure でなければ、個別 issue として分離する必要がある。

NEPLg3 仕様文書とは構文差分が大きい。現行 tutorial は NEPLg2.0 用として正しいが、NEPLg3 への移行資料と混ざらないよう、doc index と migration note で対象実装を明示し続ける必要がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `00_index.n.md` | 現行章立てを提示。 | 良い。 |
| Part 0-2 | 実行、値、失敗処理。 | 現行 NEPLg2 に追従。 |
| Part 3 | string / ByteBuf / char。 | char 連携済み。 |
| Part 4 | collection / ownership / drop。 | stdlib drop contract の残件に注意。 |
| Part 5 | module / generics / traits。 | Copy-bound generic 方針を固定。 |
| Part 6 | 実践 project。 | executable docs として有効。 |
| Advanced | 競プロ track。 | stdlib 変更時の追従確認が必要。 |
| GitHub Actions `tutorials-test` | latest `c5f93163` run `25508600937` は in_progress。直前 `b9e85f23` run では failure。 | getting_started failure と VFS cross-file tree failure は issue 化済み。latest completed run で再判定。 |

## 推奨対応

- tutorial の `std/test` example を `test_report_*` API と stdout expectation へ段階移行する。
- collection/drop の未解決 issue が動いたら、Part 4 と Advanced track を再確認する。
- NEPLg3 仕様との混同を避けるため、tutorial 先頭と doc index の対象実装表示を維持する。
- `tutorials-test` failure は latest completed run の log / artifact を確認し、`ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153` と `ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9` の範囲と一致するか再判定する。
