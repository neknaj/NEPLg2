# stdlib/hashset.n.md

## hashset_empty_observers

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

fn must_hs %impure fn Result HashSet i32 DefaultHash32 Diag HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs %HashSet i32 DefaultHash32 must_hs new DefaultHash32;
    let len_ok %bool eq len &hs 0;
    let missing_ok %bool not contains &hs 5;
    free hs;
    if and len_ok missing_ok 0 1
```

## hashset_insert_and_contains

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

fn must_hs %impure fn Result HashSet i32 DefaultHash32 Diag HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hs %impure fn Result HashSet i32 DefaultHash32 HashSetUpdateError i32 DefaultHash32 HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet i32 DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet i32 DefaultHash32 must_hs new DefaultHash32;
    let hs1 %HashSet i32 DefaultHash32 must_hs insert hs0 5;
    let hs2 %HashSet i32 DefaultHash32 must_hs insert hs1 1;
    let hs3 %HashSet i32 DefaultHash32 must_hs insert hs2 9;
    let hs4 %HashSet i32 DefaultHash32 must_hs insert hs3 5;
    let len_ok %bool eq len &hs4 3;
    let contains_ok %bool and:
        and contains &hs4 5 contains &hs4 1
        contains &hs4 9
    free hs4;
    if and len_ok contains_ok 0 1
```

## hashset_remove_and_missing_error

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

fn must_hs %impure fn Result HashSet i32 DefaultHash32 Diag HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hs %impure fn Result HashSet i32 DefaultHash32 HashSetUpdateError i32 DefaultHash32 HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet i32 DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet i32 DefaultHash32 must_hs new DefaultHash32;
    let hs1 %HashSet i32 DefaultHash32 must_hs insert hs0 5;
    let hs2 %HashSet i32 DefaultHash32 must_hs insert hs1 1;
    let hs3 %HashSet i32 DefaultHash32 must_hs insert hs2 9;
    let hs4 %HashSet i32 DefaultHash32 must_hs remove hs3 5;
    let remove_ok %bool not contains &hs4 5;
    free hs4;

    let miss0 %HashSet i32 DefaultHash32 must_hs new DefaultHash32;
    let miss1 %HashSet i32 DefaultHash32 must_hs insert miss0 5;
    let missing_err %bool:
        match remove miss1 99:
            Result::Ok hs:
                free hs;
                false
            Result::Err e:
                let hs %HashSet i32 DefaultHash32 hashset_update_error_owner e;
                free hs;
                true
    if and remove_ok missing_err 0 1
```

## hashset_free_smoke

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hs %impure fn Result HashSet i32 DefaultHash32 Diag HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hs %impure fn Result HashSet i32 DefaultHash32 HashSetUpdateError i32 DefaultHash32 HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet i32 DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet i32 DefaultHash32 must_hs new DefaultHash32;
    let hs1 %HashSet i32 DefaultHash32 must_hs insert hs0 5;
    free hs1;
    0
```
