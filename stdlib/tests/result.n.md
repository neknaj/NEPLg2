# stdlib/result.n.md

## result_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"result_main\" count=13 failed=0\nassertion index=0 status=ok kind=bool label=\"ok is_ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"ok is not err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"ok unwrap_or\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"second ok is_ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"err is_err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"err is not ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"err unwrap_or default\" expected=\"9\" actual=\"9\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"second err is_err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"second err unwrap_or default\" expected=\"50\" actual=\"50\" message=\"\"\nassertion index=9 status=ok kind=eq_i32 label=\"unwrap_ok\" expected=\"11\" actual=\"11\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"unwrap_err\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=11 status=ok kind=eq_i32 label=\"and_then success\" expected=\"12\" actual=\"12\" message=\"\"\nassertion index=12 status=ok kind=eq_i32 label=\"and_then error\" expected=\"-1\" actual=\"-1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn positive_double %fn i32 Result i32 i32 \x:
    if gt x 0:
        then ok mul x 2
        else err -1

fn main %impure fn () i32 \():
    let r1 %Result i32 i32 ok 5;
    let r1_ok %bool is_ok r1;
    let r1_err %bool is_err<i32,i32> ok 5;

    let r2 %Result i32 i32 ok 10;
    let r2_value %i32 unwrap_or r2 0;

    let r3 %Result i32 i32 ok 42;
    let r3_ok %bool is_ok r3;

    let e1 %Result i32 i32 err 7;
    let e1_err %bool is_err e1;
    let e1_ok %bool is_ok<i32,i32> err 7;

    let e2 %Result i32 i32 err 99;
    let e2_value %i32 unwrap_or e2 9;

    let e3 %Result i32 i32 err 123;
    let e3_err %bool is_err e3;
    let e4 %Result i32 i32 err 123;
    let e4_value %i32 unwrap_or e4 50;

    let okv %Result i32 i32 ok 11;
    let okv_value %i32 unwrap_ok okv;

    let errv %Result i32 i32 err 7;
    let errv_value %i32 unwrap_err errv;

    let r5 %Result i32 i32 ok 6;
    let r6 %Result i32 i32 ok -1;
    let r7 %Result i32 i32 and_then r5 positive_double;
    let r8 %Result i32 i32 and_then r6 positive_double;
    let r7_value %i32 unwrap_ok r7;
    let r8_value %i32 unwrap_err r8;

    let report:
        test_report_new "result_main"
        |> test_report_push assert "ok is_ok" r1_ok
        |> test_report_push assert "ok is not err" not r1_err
        |> test_report_push assert_eq_i32 "ok unwrap_or" 10 r2_value
        |> test_report_push assert "second ok is_ok" r3_ok
        |> test_report_push assert "err is_err" e1_err
        |> test_report_push assert "err is not ok" not e1_ok
        |> test_report_push assert_eq_i32 "err unwrap_or default" 9 e2_value
        |> test_report_push assert "second err is_err" e3_err
        |> test_report_push assert_eq_i32 "second err unwrap_or default" 50 e4_value
        |> test_report_push assert_eq_i32 "unwrap_ok" 11 okv_value
        |> test_report_push assert_eq_i32 "unwrap_err" 7 errv_value
        |> test_report_push assert_eq_i32 "and_then success" 12 r7_value
        |> test_report_push assert_eq_i32 "and_then error" -1 r8_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
