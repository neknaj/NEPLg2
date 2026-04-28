# stdlib/cast.n.md

## cast_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "core/cast" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let bti_true_i32 <i32> <i32> cast true;
    let bti_false_i32 <i32> <i32> cast false;
    let inferred_true_i32 <i32> cast true;
    let inferred_false_i32 <i32> cast false;
    let i1_as_bool <bool> <bool> cast 1;
    let i42_as_bool <bool> <bool> cast 42;
    let i0_as_bool <bool> <bool> cast 0;
    let cast_1_bool <bool> cast 1;
    let cast_42_bool <bool> cast 42;
    let cast_0_bool <bool> cast 0;
    let b <u8> cast 222;
    let checks:
        checks_new
        |> checks_push assert_eq_i32 1 bti_true_i32
        |> checks_push assert_eq_i32 0 bti_false_i32
        |> checks_push assert_eq_i32 1 inferred_true_i32
        |> checks_push assert_eq_i32 0 inferred_false_i32
        |> checks_push assert i1_as_bool
        |> checks_push assert i42_as_bool
        |> checks_push assert_ne true i0_as_bool
        |> checks_push assert cast_1_bool
        |> checks_push assert cast_42_bool
        |> checks_push assert_ne true cast_0_bool
        |> checks_push assert_eq_i32 222 cast b
    let shown checks_print_report checks;
    checks_exit_code shown
```
