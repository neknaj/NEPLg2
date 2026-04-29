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

fn must_hss <(Result<HashSet<str,DefaultHash32>, Diag>)*>HashSet<str,DefaultHash32>> (r):
    unwrap_ok<HashSet<str,DefaultHash32>, Diag> r

fn main <()*>i32> ():
    let mut code <i32> 0;
    let hs0 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs0_len <i32> len &hs0;
    free hs0;
    if:
        ne hs0_len 0
        then set code 10
        else ()

    let hs1 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs1_has <bool> contains &hs1 "foo";
    free hs1;
    if:
        and eq code 0 hs1_has
        then set code 20
        else ()

    let hs2 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs2 <HashSet<str,DefaultHash32>> must_hss insert hs2 "foo";
    let hs2 <HashSet<str,DefaultHash32>> must_hss insert hs2 "bar";
    let hs2 <HashSet<str,DefaultHash32>> must_hss insert hs2 "foo";
    let hs2_len <i32> len &hs2;
    free hs2;
    if:
        and eq code 0 ne hs2_len 2
        then set code 30
        else ()

    let hs2a <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs2a <HashSet<str,DefaultHash32>> must_hss insert hs2a "foo";
    let hs2a <HashSet<str,DefaultHash32>> must_hss insert hs2a "bar";
    let hs2a_has <bool> contains &hs2a "foo";
    free hs2a;
    if:
        and eq code 0 not hs2a_has
        then set code 40
        else ()

    let hs2b <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs2b <HashSet<str,DefaultHash32>> must_hss insert hs2b "foo";
    let hs2b <HashSet<str,DefaultHash32>> must_hss insert hs2b "bar";
    let hs2b_has <bool> contains &hs2b "bar";
    free hs2b;
    if:
        and eq code 0 not hs2b_has
        then set code 50
        else ()

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hs3 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs3 <HashSet<str,DefaultHash32>> must_hss insert hs3 s1;
    let hs3_has <bool> contains &hs3 s2;
    free hs3;
    if:
        and eq code 0 not hs3_has
        then set code 60
        else ()

    let hs4 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs4 <HashSet<str,DefaultHash32>> must_hss insert hs4 "foo";
    let hs4 <HashSet<str,DefaultHash32>> must_hss remove hs4 "foo";
    let hs4_has <bool> contains &hs4 "foo";
    free hs4;
    if:
        and eq code 0 hs4_has
        then set code 70
        else ()

    let hs5 <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hs5 <HashSet<str,DefaultHash32>> must_hss insert hs5 "foo";
    let hs5_er <Result<HashSet<str,DefaultHash32>, Diag>> remove hs5 "zzz";
    let hs5_is_err <bool> is_err<HashSet<str,DefaultHash32>, Diag> hs5_er;
    if:
        and eq code 0 not hs5_is_err
        then set code 80
        else ()
    code
```

## hashset_str_free_smoke

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

fn must_hss <(Result<HashSet<str,DefaultHash32>, Diag>)*>HashSet<str,DefaultHash32>> (r):
    unwrap_ok<HashSet<str,DefaultHash32>, Diag> r

fn main <()*>i32> ():
    let hsf <HashSet<str,DefaultHash32>> must_hss new DefaultHash32;
    let hsf <HashSet<str,DefaultHash32>> must_hss insert hsf "x";
    free hsf;
    0
```
