# stdlib/hashset.n.md

## hashset_main

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/hash/hash32" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn must_hs <(Result<HashSet<i32,DefaultHash32>, Diag>)*>HashSet<i32,DefaultHash32>> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let hs0 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    set checks checks_push checks check_eq_i32 0 len hs0;

    let hs1 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    set checks checks_push checks check not contains hs1 5;

    let hs2 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 5;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 1;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 9;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 5;
    let hs2_len <i32> len hs2;
    set checks checks_push checks check_eq_i32 3 hs2_len;

    let hs2a <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 5;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 1;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 9;
    set checks checks_push checks check contains hs2a 5;

    let hs2b <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 5;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 1;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 9;
    set checks checks_push checks check contains hs2b 1;

    let hs2c <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 5;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 1;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 9;
    set checks checks_push checks check contains hs2c 9;

    let hs3 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 5;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 1;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 9;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs remove hs3 5;
    set checks checks_push checks check not contains hs3 5;

    let hs4 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs4 <HashSet<i32,DefaultHash32>> must_hs insert hs4 5;
    let er <Result<HashSet<i32,DefaultHash32>, Diag>> remove hs4 99;
    set checks checks_push checks check is_err<HashSet<i32,DefaultHash32>, Diag> er;

    let hsf <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hsf <HashSet<i32,DefaultHash32>> must_hs insert hsf 5;
    free hsf;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
