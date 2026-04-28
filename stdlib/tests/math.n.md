# stdlib/math.n.md

## math_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push assert_eq_i32 3 add 1 2
        |> checks_push assert_eq_i32 -1 sub 1 2
        |> checks_push assert_eq_i32 6 mul 2 3
        |> checks_push assert_eq_i32 3 div_s 6 2
        |> checks_push assert_eq_i32 1 rem_s 7 3
        |> checks_push assert_eq_i32 3 and 7 3
        |> checks_push assert_eq_i32 7 or 5 3
        |> checks_push assert_eq_i32 6 xor 5 3
        |> checks_push assert_eq_i32 8 shl 2 2
        |> checks_push assert_eq_i32 1 shr_s 4 2
        |> checks_push assert_eq_i32 0 clz -2147483648
        |> checks_push assert_eq_i32 0 ctz 1
        |> checks_push assert lt 1 2
        |> checks_push assert le 2 2
        |> checks_push assert gt 2 1
        |> checks_push assert ge 2 2
        |> checks_push assert eq 5 5
        |> checks_push assert ne 5 6
        |> checks_push assert_eq_i32 5 add 2 3
        |> checks_push assert_eq_i32 -1 sub 1 2
        |> checks_push assert_eq_i32 12 mul 3 4
        |> checks_push assert_eq_i32 2 div_s 6 3
        |> checks_push assert_eq_i32 1 mod_s 7 3
        |> checks_push assert lt 1 2
        |> checks_push assert le 2 2
        |> checks_push assert eq 5 5
        |> checks_push assert ne 5 6
    let shown checks_print_report checks;
    checks_exit_code shown
```
