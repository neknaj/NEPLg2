# char_cast

`char` 変数は暗黙には整数へ変換されないため、stdlib の UTF-8 decoder は `core/cast` の明示変換だけを使う。

## char_variable_casts_to_code_point

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_variable_casts_to_code_point\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"char A code point\" expected=\"65\" actual=\"65\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4

#import "core/cast" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let c %char 'A'
    let actual %i32 cast c
    let report:
        test_report_new "char_variable_casts_to_code_point"
        |> test_report_push assert_eq_i32 "char A code point" 65 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## checked_code_point_can_cast_to_char

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"checked_code_point_can_cast_to_char\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"code point 65 casts to A\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4

#import "core/cast" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let c %char cast 65
    let actual %i32 match c:
        'A':
            1
        _:
            0
    let report:
        test_report_new "checked_code_point_can_cast_to_char"
        |> test_report_push assert_eq_i32 "code point 65 casts to A" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
