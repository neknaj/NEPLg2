# stdlib/hashset_str.n.md

## hashset_str_empty_observers

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hss %impure fn Result HashSet str DefaultHash32 Diag HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let len_ok %bool eq len &hs 0;
    let missing_ok %bool not contains &hs "foo";
    free hs;
    if and len_ok missing_ok 0 1
```

## hashset_str_insert_and_contains

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hss %impure fn Result HashSet str DefaultHash32 Diag HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hss %impure fn Result HashSet str DefaultHash32 HashSetUpdateError str DefaultHash32 HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs1 %HashSet str DefaultHash32 must_hss insert hs0 "foo";
    let hs2 %HashSet str DefaultHash32 must_hss insert hs1 "bar";
    let hs3 %HashSet str DefaultHash32 must_hss insert hs2 "foo";
    let len_ok %bool eq len &hs3 2;
    let contains_ok %bool and contains &hs3 "foo" contains &hs3 "bar";
    free hs3;

    let s1 %str concat "a" "b";
    let s2 %str concat "a" "b";
    let hs4 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs5 %HashSet str DefaultHash32 must_hss insert hs4 s1;
    let concat_ok %bool contains &hs5 s2;
    free hs5;
    if and len_ok and contains_ok concat_ok 0 1
```

## hashset_str_remove_and_missing_error

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hss %impure fn Result HashSet str DefaultHash32 Diag HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hss %impure fn Result HashSet str DefaultHash32 HashSetUpdateError str DefaultHash32 HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs1 %HashSet str DefaultHash32 must_hss insert hs0 "foo";
    let hs2 %HashSet str DefaultHash32 must_hss remove hs1 "foo";
    let remove_ok %bool not contains &hs2 "foo";
    free hs2;

    let miss0 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let miss1 %HashSet str DefaultHash32 must_hss insert miss0 "foo";
    let missing_err %bool:
        match remove miss1 "zzz":
            Result::Ok hs:
                free hs;
                false
            Result::Err e:
                let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
                free hs;
                true
    if and remove_ok missing_err 0 1
```

## hashset_str_free_smoke

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hss %impure fn Result HashSet str DefaultHash32 Diag HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hss %impure fn Result HashSet str DefaultHash32 HashSetUpdateError str DefaultHash32 HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs1 %HashSet str DefaultHash32 must_hss insert hs0 "x";
    free hs1;
    0
```
