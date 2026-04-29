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
    unwrap_ok<HashSet<i32,DefaultHash32>, Diag> r

fn main <()*>i32> ():
    let mut checks checks_new;
    let hs0 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs0_len <i32> len &hs0;
    let c0 <Result<(),str>> check_eq_i32 0 hs0_len;
    set checks checks_push checks c0;
    free hs0;

    let hs1 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs1_has <bool> contains &hs1 5;
    let c1 <Result<(),str>> check not hs1_has;
    set checks checks_push checks c1;
    free hs1;

    let hs2 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 5;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 1;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 9;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 5;
    let hs2_len <i32> len &hs2;
    let c2 <Result<(),str>> check_eq_i32 3 hs2_len;
    set checks checks_push checks c2;
    free hs2;

    let hs2a <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 5;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 1;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 9;
    let hs2a_has <bool> contains &hs2a 5;
    let c3 <Result<(),str>> check hs2a_has;
    set checks checks_push checks c3;
    free hs2a;

    let hs2b <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 5;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 1;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 9;
    let hs2b_has <bool> contains &hs2b 1;
    let c4 <Result<(),str>> check hs2b_has;
    set checks checks_push checks c4;
    free hs2b;

    let hs2c <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 5;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 1;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 9;
    let hs2c_has <bool> contains &hs2c 9;
    let c5 <Result<(),str>> check hs2c_has;
    set checks checks_push checks c5;
    free hs2c;

    let hs3 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 5;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 1;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 9;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs remove hs3 5;
    let hs3_has <bool> contains &hs3 5;
    let c6 <Result<(),str>> check not hs3_has;
    set checks checks_push checks c6;
    free hs3;

    let hs4 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs4 <HashSet<i32,DefaultHash32>> must_hs insert hs4 5;
    let er <Result<HashSet<i32,DefaultHash32>, Diag>> remove hs4 99;
    set checks checks_push checks check is_err<HashSet<i32,DefaultHash32>, Diag> er;

    let shown checks_print_report checks;
    checks_exit_code shown
```

## hashset_free_smoke

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn must_hs <(Result<HashSet<i32,DefaultHash32>, Diag>)*>HashSet<i32,DefaultHash32>> (r):
    unwrap_ok<HashSet<i32,DefaultHash32>, Diag> r

fn main <()*>i32> ():
    let hsf <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hsf <HashSet<i32,DefaultHash32>> must_hs insert hsf 5;
    free hsf;
    0
```
