# stdlib/option.n.md

## option_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "core/option" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn positive_double <(i32)->Option<i32>> (x):
    if gt x 0:
        then some<i32> mul x 2
        else none<i32>

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new
    // Test is_some
    set checks checks_push checks assert is_some<.i32> some<.i32> 42;
    set checks checks_push checks assert_ne true is_none<.i32> some<.i32> 42;

    // Test is_none
    set checks checks_push checks assert is_none<.i32> none<.i32>;
    set checks checks_push checks assert_ne true is_some<.i32> none<.i32>;

    // Test unwrap on Some
    set checks checks_push checks assert_eq_i32 99 unwrap<.i32> some<.i32> 99;

    // Test unwrap_or with Some
    set checks checks_push checks assert_eq_i32 10 unwrap_or<.i32> some<.i32> 10 5;

    // Test unwrap_or with None
    set checks checks_push checks assert_eq_i32 5 unwrap_or<.i32> none<.i32> 5;

    // Test and_then with Some / None result
    set checks checks_push checks assert_eq_i32 12 unwrap<.i32> and_then<i32,i32> some<i32> 6 positive_double;
    set checks checks_push checks assert is_none<.i32> and_then<i32,i32> some<i32> -1 positive_double;

    // Option<T: Copy> can be copied through a shared reference.
    let original <Option<i32>> some<i32> 77
    let copied <Option<i32>> *&original
    set checks checks_push checks assert_eq_i32 77 unwrap<i32> copied;
    checks_exit_code checks
```
