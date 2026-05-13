# stdlib/hashmap_str.n.md

## hashmap_str_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn must_hms <(Result<HashMap<str,i32,DefaultHash32>, Diag>)*>HashMap<str,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> i32> ():
    let mut code <i32> 0;
    let hm0 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm0_len <i32> len &hm0;
    if:
        ne hm0_len 0
        then set code 10
        else ()
    free hm0;

    let hm1 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm1_has <bool> contains &hm1 "foo";
    if:
        and eq code 0 hm1_has
        then set code 20
        else ()
    free hm1;

    let hm2 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm2_got <Option<i32>> get &hm2 "foo";
    let hm2_none <bool> is_none<i32> hm2_got;
    if:
        and eq code 0 not hm2_none
        then set code 30
        else ()
    free hm2;

    let hm3 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3 <HashMap<str,i32,DefaultHash32>> must_hms insert hm3 "foo" 10;
    let hm3 <HashMap<str,i32,DefaultHash32>> must_hms insert hm3 "bar" 20;
    let hm3_len <i32> len &hm3;
    if:
        and eq code 0 ne hm3_len 2
        then set code 40
        else ()
    free hm3;

    let hm3a <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3a <HashMap<str,i32,DefaultHash32>> must_hms insert hm3a "foo" 10;
    let hm3a <HashMap<str,i32,DefaultHash32>> must_hms insert hm3a "bar" 20;
    let hm3a_has <bool> contains &hm3a "foo";
    if:
        and eq code 0 not hm3a_has
        then set code 50
        else ()
    free hm3a;

    let hm3b <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3b <HashMap<str,i32,DefaultHash32>> must_hms insert hm3b "foo" 10;
    let hm3b <HashMap<str,i32,DefaultHash32>> must_hms insert hm3b "bar" 20;
    let hm3b_has <bool> contains &hm3b "bar";
    if:
        and eq code 0 not hm3b_has
        then set code 60
        else ()
    free hm3b;

    let hm3c <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm3c <HashMap<str,i32,DefaultHash32>> must_hms insert hm3c "foo" 10;
    let hm3c <HashMap<str,i32,DefaultHash32>> must_hms insert hm3c "bar" 20;
    let hm3c_has <bool> contains &hm3c "baz";
    if:
        and eq code 0 hm3c_has
        then set code 70
        else ()
    free hm3c;

    let s1 <str> concat "a" "b";
    let s2 <str> concat "a" "b";
    let hm4 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm4 <HashMap<str,i32,DefaultHash32>> must_hms insert hm4 s1 30;
    match get &hm4 s2:
        Option::Some v:
            if:
                and eq code 0 ne v 30
                then set code 80
                else ()
        Option::None:
            if:
                eq code 0
                then set code 90
                else ()
    free hm4;

    let hm5 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm5 <HashMap<str,i32,DefaultHash32>> must_hms insert hm5 "foo" 10;
    let hm5 <HashMap<str,i32,DefaultHash32>> must_hms insert hm5 "foo" 11;
    match get &hm5 "foo":
        Option::Some v:
            if:
                and eq code 0 ne v 11
                then set code 100
                else ()
        Option::None:
            if:
                eq code 0
                then set code 110
                else ()
    free hm5;

    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms insert hm6 "foo" 10;
    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms insert hm6 "bar" 20;
    let hm6 <HashMap<str,i32,DefaultHash32>> must_hms remove hm6 "bar";
    let hm6_has <bool> contains &hm6 "bar";
    if:
        and eq code 0 hm6_has
        then set code 120
        else ()
    free hm6;

    let hm7 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hm7 <HashMap<str,i32,DefaultHash32>> must_hms insert hm7 "foo" 10;
    let hm7_er <Result<HashMap<str,i32,DefaultHash32>, Diag>> remove hm7 "zzz";
    let hm7_is_err <bool> is_err<HashMap<str,i32,DefaultHash32>, Diag> hm7_er;
    if:
        and eq code 0 not hm7_is_err
        then set code 130
        else ()
    code
```

## hashmap_str_free_smoke

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn must_hms <(Result<HashMap<str,i32,DefaultHash32>, Diag>)*>HashMap<str,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> i32> ():
    let hmf <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hmf <HashMap<str,i32,DefaultHash32>> must_hms insert hmf "x" 1;
    free hmf;
    0
```
