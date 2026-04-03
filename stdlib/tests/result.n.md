# stdlib/result.n.md

## result_main

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn positive_double <(i32)->Result<i32,i32>> (x):
    if gt x 0:
        then ok<i32,i32> mul x 2
        else err<i32,i32> -1

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new
    // Test ok and is_ok
    let r1 <Result<i32,i32>> ok<i32,i32> 5;
    set checks checks_push checks assert is_ok<i32,i32> r1;
    set checks checks_push checks assert_ne true is_err<i32,i32> ok<i32,i32> 5;

    // Test ok with unwrap_or
    let r2 <Result<i32,i32>> ok<i32,i32> 10;
    set checks checks_push checks assert_eq_i32 10 unwrap_or<i32,i32> r2 0;

    // Test multiple ok values
    let r3 <Result<i32,i32>> ok<i32,i32> 42;
    set checks checks_push checks assert is_ok<i32,i32> r3;

    // Test err and is_err
    let e1 <Result<i32,i32>> err<i32,i32> 7;
    set checks checks_push checks assert is_err<i32,i32> e1;
    set checks checks_push checks assert_ne true is_ok<i32,i32> err<i32,i32> 7;

    // Test err with unwrap_or (returns default)
    let e2 <Result<i32,i32>> err<i32,i32> 99;
    set checks checks_push checks assert_eq_i32 9 unwrap_or<i32,i32> e2 9;

    // Test multiple err values
    let e3 <Result<i32,i32>> err<i32,i32> 123;
    set checks checks_push checks assert is_err<i32,i32> e3;
    let e4 <Result<i32,i32>> err<i32,i32> 123;
    set checks checks_push checks assert_eq_i32 50 unwrap_or<i32,i32> e4 50;

    // Test unwrap_ok
    let okv <Result<i32,i32>> ok<i32,i32> 11;
    set checks checks_push checks assert_eq_i32 11 unwrap_ok<i32,i32> okv;

    // Test unwrap_err
    let errv <Result<i32,i32>> err<i32,i32> 7;
    set checks checks_push checks assert_eq_i32 7 unwrap_err<i32,i32> errv;

    // Test and_then with success and error propagation
    let r5 <Result<i32,i32>> ok<i32,i32> 6;
    let r6 <Result<i32,i32>> ok<i32,i32> -1;
    let r7 <Result<i32,i32>> and_then<i32,i32,i32> r5 positive_double;
    let r8 <Result<i32,i32>> and_then<i32,i32,i32> r6 positive_double;
    set checks checks_push checks assert_eq_i32 12 unwrap_ok<i32,i32> r7;
    set checks checks_push checks assert_eq_i32 -1 unwrap_err<i32,i32> r8;
    checks_exit_code checks
```
