# math overload / cast doctest

## math_i32_overload_add_sub_mul

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"math_i32_overload_add_sub_mul\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"i32 arithmetic result\" expected=\"37\" actual=\"37\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let a %i32 add 40 2;
    let b %i32 sub a 5;
    let c %i32 mul b 2;
    let actual %i32 add c -37
    let report:
        test_report_new "math_i32_overload_add_sub_mul"
        |> test_report_push assert_eq_i32 "i32 arithmetic result" 37 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## math_facade_qualified_alias_reexports_i32_arith

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"math_facade_qualified_alias_reexports_i32_arith\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"qualified facade arithmetic\" expected=\"19\" actual=\"19\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as math
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let actual %i32 math::add math::add 3 4 math::mul 3 4
    let report:
        test_report_new "math_facade_qualified_alias_reexports_i32_arith"
        |> test_report_push assert_eq_i32 "qualified facade arithmetic" 19 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## math_i64_overload_add_sub_mul

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"math_i64_overload_add_sub_mul\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"i64 arithmetic cast result\" expected=\"74\" actual=\"74\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let a %i64 cast 40;
    let b %i64 cast 2;
    let five %i64 cast 5;
    let two %i64 cast 2;
    let c %i64 add a b;
    let d %i64 sub c five;
    let e %i64 mul d two;
    let actual %i32 cast e;
    let report:
        test_report_new "math_i64_overload_add_sub_mul"
        |> test_report_push assert_eq_i32 "i64 arithmetic cast result" 74 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## math_i128_overload_add_sub_mul

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"math_i128_overload_add_sub_mul\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"i128 arithmetic cast result\" expected=\"78\" actual=\"78\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let a64 %i64 cast 40;
    let b64 %i64 cast 2;
    let a %i128 cast a64;
    let b %i128 cast b64;
    let three64 %i64 cast 3;
    let two64 %i64 cast 2;
    let c %i128 add a b;
    let d %i128 sub c %i128 cast three64;
    let e %i128 mul d %i128 cast two64;
    let out64 %i64 cast e;
    let actual %i32 %i32 cast out64
    let report:
        test_report_new "math_i128_overload_add_sub_mul"
        |> test_report_push assert_eq_i32 "i128 arithmetic cast result" 78 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## cast_overload_numeric_roundtrip

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"cast_overload_numeric_roundtrip\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"numeric cast roundtrip\" expected=\"123\" actual=\"123\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let a32 %i32 123;
    let a64 %i64 cast a32;
    let a128 %i128 cast a64;
    let b64 %i64 cast a128;
    let actual %i32 cast b64
    let report:
        test_report_new "cast_overload_numeric_roundtrip"
        |> test_report_push assert_eq_i32 "numeric cast roundtrip" 123 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## cast_ambiguous_without_expected_type

`let v cast 1` の `v` に型注釈がなく `cast` の戻り値型を決める文脈も無いため、戻り値型の形だけで候補を選ばず `type.overload.ambiguous` として拒否する。

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/cast" as *

fn main %fn unit i32 \unit:
    let v cast 1
    0
```
