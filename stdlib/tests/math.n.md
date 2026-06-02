# stdlib/math.n.md

## math_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"math_main\" count=27 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"add 1 2\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"sub 1 2\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"mul 2 3\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"div_s 6 2\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"rem_s 7 3\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"and 7 3\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"or 5 3\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"xor 5 3\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"shl 2 2\" expected=\"8\" actual=\"8\" message=\"\"\nassertion index=9 status=ok kind=eq_i32 label=\"shr_s 4 2\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"clz min_i32\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=11 status=ok kind=eq_i32 label=\"ctz 1\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"lt 1 2\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=bool label=\"le 2 2\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"gt 2 1\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=15 status=ok kind=bool label=\"ge 2 2\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=16 status=ok kind=bool label=\"eq 5 5\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=17 status=ok kind=bool label=\"ne 5 6\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=18 status=ok kind=eq_i32 label=\"add 2 3\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=19 status=ok kind=eq_i32 label=\"sub duplicate\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=20 status=ok kind=eq_i32 label=\"mul 3 4\" expected=\"12\" actual=\"12\" message=\"\"\nassertion index=21 status=ok kind=eq_i32 label=\"div_s 6 3\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=22 status=ok kind=eq_i32 label=\"mod_s 7 3\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=23 status=ok kind=bool label=\"lt duplicate\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=24 status=ok kind=bool label=\"le duplicate\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=25 status=ok kind=bool label=\"eq duplicate\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=26 status=ok kind=bool label=\"ne duplicate\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let report:
        test_report_new "math_main"
        |> test_report_push assert_eq_i32 "add 1 2" 3 add 1 2
        |> test_report_push assert_eq_i32 "sub 1 2" -1 sub 1 2
        |> test_report_push assert_eq_i32 "mul 2 3" 6 mul 2 3
        |> test_report_push assert_eq_i32 "div_s 6 2" 3 div_s 6 2
        |> test_report_push assert_eq_i32 "rem_s 7 3" 1 rem_s 7 3
        |> test_report_push assert_eq_i32 "and 7 3" 3 and 7 3
        |> test_report_push assert_eq_i32 "or 5 3" 7 or 5 3
        |> test_report_push assert_eq_i32 "xor 5 3" 6 xor 5 3
        |> test_report_push assert_eq_i32 "shl 2 2" 8 shl 2 2
        |> test_report_push assert_eq_i32 "shr_s 4 2" 1 shr_s 4 2
        |> test_report_push assert_eq_i32 "clz min_i32" 0 clz -2147483648
        |> test_report_push assert_eq_i32 "ctz 1" 0 ctz 1
        |> test_report_push assert "lt 1 2" lt 1 2
        |> test_report_push assert "le 2 2" le 2 2
        |> test_report_push assert "gt 2 1" gt 2 1
        |> test_report_push assert "ge 2 2" ge 2 2
        |> test_report_push assert "eq 5 5" eq 5 5
        |> test_report_push assert "ne 5 6" ne 5 6
        |> test_report_push assert_eq_i32 "add 2 3" 5 add 2 3
        |> test_report_push assert_eq_i32 "sub duplicate" -1 sub 1 2
        |> test_report_push assert_eq_i32 "mul 3 4" 12 mul 3 4
        |> test_report_push assert_eq_i32 "div_s 6 3" 2 div_s 6 3
        |> test_report_push assert_eq_i32 "mod_s 7 3" 1 mod_s 7 3
        |> test_report_push assert "lt duplicate" lt 1 2
        |> test_report_push assert "le duplicate" le 2 2
        |> test_report_push assert "eq duplicate" eq 5 5
        |> test_report_push assert "ne duplicate" ne 5 6
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
