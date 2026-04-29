# std/test の構造化 report API

## std_test_collect_success_summary

[目的/もくてき]:
- `std/test` の `TestAssertion` を `TestReport` へ積み、stdout report と exit code が分離していることを確認します。

[何/なに]を[確/たし]かめるか:
- `assert_*` は stdout を出さず、`TestAssertion` を返すこと。
- `test_report_print_stdout` が canonical report を 1 回だけ stdout に出すこと。
- `test_report_exit_code` が report の failed count だけから 0 を返すこと。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"std_test_collect_success_summary\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"addition\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"concat\" expected=\"ab\" actual=\"ab\" message=\"\"\nassertion index=2 status=ok kind=ok_i32 label=\"result ok\" expected=\"Ok\" actual=\"7\" message=\"\"\nassertion index=3 status=ok kind=err_i32 label=\"result err\" expected=\"Err\" actual=\"5\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let report:
        test_report_new "std_test_collect_success_summary"
        |> test_report_push assert_eq_i32 "addition" 3 add 1 2
        |> test_report_push assert_str_eq "concat" "ab" concat "a" "b"
        |> test_report_push assert_ok_i32 "result ok" Result<i32,i32>::Ok 7
        |> test_report_push assert_err_i32 "result err" Result<i32,i32>::Err 5
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## std_test_collect_continues_after_string_allocation

[目的/もくてき]:
- 途中で文字列生成を挟んでも、report が保持済みの assertion 行と failed count から安定して出力されることを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"std_test_collect_continues_after_string_allocation\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"initial\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"concat after allocation\" expected=\"prefix-suffix\" actual=\"prefix-suffix\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"after concat\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

fn main <()*>i32> ():
    let mut report test_report_new "std_test_collect_continues_after_string_allocation"
    set report test_report_push report assert "initial" true
    let text <str> concat "prefix-" "suffix"
    set report test_report_push report assert_str_eq "concat after allocation" "prefix-suffix" text
    set report test_report_push report assert "after concat" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## std_test_collect_failure_summary_and_details

[目的/もくてき]:
- 失敗が含まれても後続 assertion が継続実行され、stdout に kind、label、expected、actual、message が残ることを確認します。

[何/なに]を[確/たし]かめるか:
- `test_report_print_stdout` が失敗詳細を stdout に出すこと。
- `test_report_exit_code` が failed count から 1 を返すこと。

neplg2:test[stdio, normalize_newlines]
exit_code: 1
stdout: "test_report name=\"std_test_collect_failure_summary_and_details\" count=4 failed=2\nassertion index=0 status=ok kind=eq_i32 label=\"addition\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=fail kind=eq_i32 label=\"mismatch\" expected=\"2\" actual=\"3\" message=\"assert_eq_i32 failed: expected=2 actual=3\"\nassertion index=2 status=ok kind=err_i32 label=\"result err\" expected=\"Err\" actual=\"5\" message=\"\"\nassertion index=3 status=fail kind=bool label=\"forced false\" expected=\"true\" actual=\"false\" message=\"check failed\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let report:
        test_report_new "std_test_collect_failure_summary_and_details"
        |> test_report_push assert_eq_i32 "addition" 3 add 1 2
        |> test_report_push assert_eq_i32 "mismatch" 2 3
        |> test_report_push assert_err_i32 "result err" Result<i32,i32>::Err 5
        |> test_report_push assert "forced false" false
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
