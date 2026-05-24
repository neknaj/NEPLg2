# stdlib/cast.n.md

## cast_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"cast_main\" count=11 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bool true to i32\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"bool false to i32\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"inferred true to i32\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"inferred false to i32\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"i32 1 to bool\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"i32 42 to bool\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"i32 0 to bool false\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"inferred 1 to bool\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"inferred 42 to bool\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"inferred 0 to bool false\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"u8 to i32\" expected=\"222\" actual=\"222\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "core/cast" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let bti_true_i32 %i32 %i32 cast true;
    let bti_false_i32 %i32 %i32 cast false;
    let inferred_true_i32 %i32 cast true;
    let inferred_false_i32 %i32 cast false;
    let i1_as_bool %bool %bool cast 1;
    let i42_as_bool %bool %bool cast 42;
    let i0_as_bool %bool %bool cast 0;
    let cast_1_bool %bool cast 1;
    let cast_42_bool %bool cast 42;
    let cast_0_bool %bool cast 0;
    let b %u8 cast 222;
    let b_i32 %i32 cast b;
    let report:
        test_report_new "cast_main"
        |> test_report_push assert_eq_i32 "bool true to i32" 1 bti_true_i32
        |> test_report_push assert_eq_i32 "bool false to i32" 0 bti_false_i32
        |> test_report_push assert_eq_i32 "inferred true to i32" 1 inferred_true_i32
        |> test_report_push assert_eq_i32 "inferred false to i32" 0 inferred_false_i32
        |> test_report_push assert "i32 1 to bool" i1_as_bool
        |> test_report_push assert "i32 42 to bool" i42_as_bool
        |> test_report_push assert "i32 0 to bool false" not i0_as_bool
        |> test_report_push assert "inferred 1 to bool" cast_1_bool
        |> test_report_push assert "inferred 42 to bool" cast_42_bool
        |> test_report_push assert "inferred 0 to bool false" not cast_0_bool
        |> test_report_push assert_eq_i32 "u8 to i32" 222 b_i32
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
