# numerics.rs 由来の doctest

このファイルは Rust テスト `numerics.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_i32_literals_decimal

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_i32_literals_decimal\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"decimal literal sum\" expected=\"78\" actual=\"78\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a 123;
    let b -45;
    let actual %i32 add a b
    let report:
        test_report_new "test_i32_literals_decimal"
        |> test_report_push assert_eq_i32 "decimal literal sum" 78 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_i32_literals_hex

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_i32_literals_hex\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"hex literal sum\" expected=\"271\" actual=\"271\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a 0x10;      // 16
    let b 0xFF;      // 255
    let c 0x0;       // 0
    let bc %i32 add b c;
    let actual %i32 add a bc
    let report:
        test_report_new "test_i32_literals_hex"
        |> test_report_push assert_eq_i32 "hex literal sum" 271 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_f32_literals

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_f32_literals\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"f32 arithmetic cast result\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a 1.5;
    let b -0.5;
    let c 10.0;
    let ab %f32 add a b;
    let res %f32 mul ab c;
    let actual %i32 cast res
    let report:
        test_report_new "test_f32_literals"
        |> test_report_push assert_eq_i32 "f32 arithmetic cast result" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_u8_literals_and_wrapping_add

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_u8_literals_and_wrapping_add\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"u8 wrapping add\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a %u8 cast 255;
    let b %u8 cast 1;
    let c %u8 add a b;
    let actual %i32 cast c
    let report:
        test_report_new "test_u8_literals_and_wrapping_add"
        |> test_report_push assert_eq_i32 "u8 wrapping add" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_u8_wrapping_sub

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_u8_wrapping_sub\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"u8 wrapping sub\" expected=\"255\" actual=\"255\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a %u8 cast 0;
    let b %u8 cast 1;
    let c %u8 sub a b;
    let actual %i32 cast c
    let report:
        test_report_new "test_u8_wrapping_sub"
        |> test_report_push assert_eq_i32 "u8 wrapping sub" 255 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_u8_wrapping_mul

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_u8_wrapping_mul\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"u8 wrapping mul\" expected=\"16\" actual=\"16\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a %u8 cast 16;
    let b %u8 cast 17;
    let c %u8 mul a b;
    let actual %i32 cast c
    let report:
        test_report_new "test_u8_wrapping_mul"
        |> test_report_push assert_eq_i32 "u8 wrapping mul" 16 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_u8_division_and_remainder

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_u8_division_and_remainder\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"u8 division\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"u8 remainder\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"division plus remainder\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a %u8 cast 200;
    let b %u8 cast 20;
    let div_res %u8 div_u a b; // 10
    let rem_res %u8 rem_u a b; // 0
    let d %i32 cast div_res;
    let r %i32 cast rem_res;
    let total %i32 add d r
    let report:
        test_report_new "test_u8_division_and_remainder"
        |> test_report_push assert_eq_i32 "u8 division" 10 d
        |> test_report_push assert_eq_i32 "u8 remainder" 0 r
        |> test_report_push assert_eq_i32 "division plus remainder" 10 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_u8_comparisons

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_u8_comparisons\" count=6 failed=0\nassertion index=0 status=ok kind=bool label=\"lt_u\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"le_u\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"gt_u\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"ge_u\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"eq\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"ne\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/cast" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a %u8 cast 10;
    let b %u8 cast 20;
    let c %u8 cast 10;
    let ok_lt %bool lt_u a b
    let ok_le %bool le_u a c
    let ok_gt %bool gt_u b a
    let ok_ge %bool ge_u b c
    let ok_eq %bool eq a c
    let ok_ne %bool ne a b
    let report:
        test_report_new "test_u8_comparisons"
        |> test_report_push assert "lt_u" ok_lt
        |> test_report_push assert "le_u" ok_le
        |> test_report_push assert "gt_u" ok_gt
        |> test_report_push assert "ge_u" ok_ge
        |> test_report_push assert "eq" ok_eq
        |> test_report_push assert "ne" ok_ne
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_bitwise_operations

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_bitwise_operations\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bitwise and\" expected=\"8\" actual=\"8\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"bitwise or\" expected=\"14\" actual=\"14\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"bitwise xor\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"bitwise sum\" expected=\"28\" actual=\"28\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a 0xC; // 12
    let b 0xA; // 10
    // and: 1000 (8)
    // or:  1110 (14)
    // xor: 0110 (6)
    // 8 + 14 + 6 = 28
    let r_and and a b;
    let r_or  or a b;
    let r_xor xor a b;
    let rx %i32 add r_or r_xor;
    let total %i32 add r_and rx
    let report:
        test_report_new "test_bitwise_operations"
        |> test_report_push assert_eq_i32 "bitwise and" 8 r_and
        |> test_report_push assert_eq_i32 "bitwise or" 14 r_or
        |> test_report_push assert_eq_i32 "bitwise xor" 6 r_xor
        |> test_report_push assert_eq_i32 "bitwise sum" 28 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_shift_operations

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_shift_operations\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"shift left\" expected=\"16\" actual=\"16\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"signed shift right\" expected=\"-4\" actual=\"-4\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"unsigned shift right\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"shift sum\" expected=\"16\" actual=\"16\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let a 8;
    let b -16;
    // shl 8 1 -> 16
    // shr_s -16 2 -> -4
    // shr_u 8 1 -> 4
    // 16 + (-4) + 4 = 16
    let r_shl shl a 1;
    let r_shr_s shr_s b 2;
    let r_shr_u shr_u a 1;
    let rr %i32 add r_shr_s r_shr_u;
    let total %i32 add r_shl rr
    let report:
        test_report_new "test_shift_operations"
        |> test_report_push assert_eq_i32 "shift left" 16 r_shl
        |> test_report_push assert_eq_i32 "signed shift right" -4 r_shr_s
        |> test_report_push assert_eq_i32 "unsigned shift right" 4 r_shr_u
        |> test_report_push assert_eq_i32 "shift sum" 16 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_f32_comparisons

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_f32_comparisons\" count=6 failed=0\nassertion index=0 status=ok kind=bool label=\"f32 lt\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"f32 le\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"f32 gt\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"f32 ge\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"f32 eq\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"f32 ne\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let ok_lt %bool lt 1.0 2.0
    let ok_le %bool le 2.0 2.0
    let ok_gt %bool gt 3.0 2.0
    let ok_ge %bool ge 3.0 3.0
    let ok_eq %bool eq 4.0 4.0
    let ok_ne %bool ne 4.0 5.0
    let report:
        test_report_new "test_f32_comparisons"
        |> test_report_push assert "f32 lt" ok_lt
        |> test_report_push assert "f32 le" ok_le
        |> test_report_push assert "f32 gt" ok_gt
        |> test_report_push assert "f32 ge" ok_ge
        |> test_report_push assert "f32 eq" ok_eq
        |> test_report_push assert "f32 ne" ok_ne
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
