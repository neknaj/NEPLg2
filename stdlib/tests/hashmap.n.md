# stdlib/hashmap.n.md

## hashmap_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn must_hm <(Result<HashMap<i32,i32,DefaultHash32>, Diag>)*>HashMap<i32,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> i32> ():
    let mut checks checks_new;
    let hm0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    set checks checks_push checks check_eq_i32 0 len &hm0;
    free hm0;

    let hm1 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    set checks checks_push checks check not contains &hm1 1;
    free hm1;

    let hm2 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    set checks checks_push checks check is_none<i32> get &hm2 1;
    free hm2;

    let a0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a1 <HashMap<i32,i32,DefaultHash32>> must_hm insert a0 10 100;
    let a2 <HashMap<i32,i32,DefaultHash32>> must_hm insert a1 5 50;
    let a3 <HashMap<i32,i32,DefaultHash32>> must_hm insert a2 20 200;
    let a3_len <i32> len &a3;
    set checks checks_push checks check_eq_i32 3 a3_len;
    free a3;

    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm insert a4 10 100;
    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm insert a4 5 50;
    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm insert a4 20 200;
    set checks checks_push checks check contains &a4 10;
    free a4;

    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm insert a5 10 100;
    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm insert a5 5 50;
    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm insert a5 20 200;
    set checks checks_push checks check contains &a5 5;
    free a5;

    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm insert a6 10 100;
    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm insert a6 5 50;
    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm insert a6 20 200;
    set checks checks_push checks check not contains &a6 2;
    free a6;

    let b0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let b1 <HashMap<i32,i32,DefaultHash32>> must_hm insert b0 5 50;
    match get &b1 5:
        Option::Some v:
            set checks checks_push checks check_eq_i32 50 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 5 returned None";
    free b1;

    let c0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let c1 <HashMap<i32,i32,DefaultHash32>> must_hm insert c0 5 50;
    let c2 <HashMap<i32,i32,DefaultHash32>> must_hm insert c1 5 55;
    match get &c2 5:
        Option::Some v:
            set checks checks_push checks check_eq_i32 55 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "get 5 after update returned None";
    free c2;

    let c3 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let c3 <HashMap<i32,i32,DefaultHash32>> must_hm insert c3 5 50;
    let c3 <HashMap<i32,i32,DefaultHash32>> must_hm insert c3 5 55;
    set checks checks_push checks check_eq_i32 1 len &c3;
    free c3;

    let d0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let d1 <HashMap<i32,i32,DefaultHash32>> must_hm insert d0 10 100;
    let d2 <HashMap<i32,i32,DefaultHash32>> must_hm insert d1 20 200;
    let d3 <HashMap<i32,i32,DefaultHash32>> must_hm remove d2 10;
    let d3_len <i32> len &d3;
    set checks checks_push checks check_eq_i32 1 d3_len;
    free d3;

    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm insert d4 10 100;
    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm insert d4 20 200;
    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm remove d4 10;
    set checks checks_push checks check not contains &d4 10;
    free d4;

    let e0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let e1 <HashMap<i32,i32,DefaultHash32>> must_hm insert e0 10 100;
    let er <Result<HashMap<i32,i32,DefaultHash32>, Diag>> remove e1 999;
    set checks checks_push checks check is_err<HashMap<i32,i32,DefaultHash32>, Diag> er;

    let f0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let f1 <HashMap<i32,i32,DefaultHash32>> must_hm insert f0 1 1;
    free f1;
    let shown checks_print_report checks;
    checks_exit_code shown
```
