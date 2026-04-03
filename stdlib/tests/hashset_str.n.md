# stdlib/hashset_str.n.md

## hashset_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/hash/hash32" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn must_hss <(Result<HashSet<str,DefaultHash32>, Diag>)*>HashSet<str,DefaultHash32>> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks <Vec<Result<(),str>>> checks_new;
    let hs0 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    set checks checks_push checks check_eq_i32 0 len hs0;

    let hs1 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    set checks checks_push checks check not contains hs1 "foo";

    let hs2 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs2 <HashSet<str,DefaultHash32>> must_hss insert hs2 "foo";
    let hs2 <HashSet<str,DefaultHash32>> must_hss insert hs2 "bar";
    let hs2 <HashSet<str,DefaultHash32>> must_hss insert hs2 "foo";
    let hs2_len <i32> len hs2;
    set checks checks_push checks check_eq_i32 2 hs2_len;

    let hs2a <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs2a <HashSet<str,DefaultHash32>> must_hss insert hs2a "foo";
    let hs2a <HashSet<str,DefaultHash32>> must_hss insert hs2a "bar";
    set checks checks_push checks check contains hs2a "foo";

    let hs2b <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs2b <HashSet<str,DefaultHash32>> must_hss insert hs2b "foo";
    let hs2b <HashSet<str,DefaultHash32>> must_hss insert hs2b "bar";
    set checks checks_push checks check contains hs2b "bar";

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hs3 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs3 <HashSet<str,DefaultHash32>> must_hss insert hs3 s1;
    set checks checks_push checks check contains hs3 s2;

    let hs4 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs4 <HashSet<str,DefaultHash32>> must_hss insert hs4 "foo";
    let hs4 <HashSet<str,DefaultHash32>> must_hss remove hs4 "foo";
    set checks checks_push checks check not contains hs4 "foo";

    let hs5 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs5 <HashSet<str,DefaultHash32>> must_hss insert hs5 "foo";
    set checks checks_push checks check is_err<HashSet<str,DefaultHash32>, Diag> remove hs5 "zzz";

    let hsf <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hsf <HashSet<str,DefaultHash32>> must_hss insert hsf "x";
    free hsf;
    let shown <Vec<Result<(),str>>> checks_print_report checks;
    checks_exit_code shown
```
