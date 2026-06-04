# stdlib/hashmap.n.md

## hashmap_empty_observers

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 Diag HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm %HashMap i32 i32 DefaultHash32 must_hm new DefaultHash32;
    let len_ok %bool eq len &hm 0;
    let missing_ok %bool not contains &hm 1;
    let get_ok %bool is_none get &hm 1;
    free hm;
    if and len_ok and missing_ok get_ok 0 1
```

## hashmap_insert_get_and_update

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 Diag HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 HashMapUpdateError i32 i32 DefaultHash32 HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap i32 i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm0 %HashMap i32 i32 DefaultHash32 must_hm new DefaultHash32;
    let hm1 %HashMap i32 i32 DefaultHash32 must_hm insert hm0 10 100;
    let hm2 %HashMap i32 i32 DefaultHash32 must_hm insert hm1 5 50;
    let hm3 %HashMap i32 i32 DefaultHash32 must_hm insert hm2 20 200;
    let len_ok %bool eq len &hm3 3;
    let contains_ok %bool and contains &hm3 10 contains &hm3 5;
    let missing_ok %bool not contains &hm3 2;
    let get_ok %bool:
        match get &hm3 5:
            Option::Some v:
                eq v 50
            Option::None:
                false
    free hm3;

    let upd0 %HashMap i32 i32 DefaultHash32 must_hm new DefaultHash32;
    let upd1 %HashMap i32 i32 DefaultHash32 must_hm insert upd0 5 50;
    let upd2 %HashMap i32 i32 DefaultHash32 must_hm insert upd1 5 55;
    let update_value_ok %bool:
        match get &upd2 5:
            Option::Some v:
                eq v 55
            Option::None:
                false
    let update_len_ok %bool eq len &upd2 1;
    free upd2;

    let ok %bool:
        and:
            and len_ok contains_ok
            and missing_ok and get_ok and update_value_ok update_len_ok
    if ok 0 1
```

## hashmap_remove_and_missing_error

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 Diag HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 HashMapUpdateError i32 i32 DefaultHash32 HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap i32 i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm0 %HashMap i32 i32 DefaultHash32 must_hm new DefaultHash32;
    let hm1 %HashMap i32 i32 DefaultHash32 must_hm insert hm0 10 100;
    let hm2 %HashMap i32 i32 DefaultHash32 must_hm insert hm1 20 200;
    let hm3 %HashMap i32 i32 DefaultHash32 must_hm remove hm2 10;
    let remove_len_ok %bool eq len &hm3 1;
    let remove_clears_ok %bool not contains &hm3 10;
    free hm3;

    let miss0 %HashMap i32 i32 DefaultHash32 must_hm new DefaultHash32;
    let miss1 %HashMap i32 i32 DefaultHash32 must_hm insert miss0 10 100;
    let missing_err %bool:
        match remove miss1 999:
            Result::Ok hm:
                free hm;
                false
            Result::Err e:
                let hm %HashMap i32 i32 DefaultHash32 hashmap_update_error_owner e;
                free hm;
                true
    if and remove_len_ok and remove_clears_ok missing_err 0 1
```

## hashmap_free_smoke

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 Diag HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 HashMapUpdateError i32 i32 DefaultHash32 HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap i32 i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm0 %HashMap i32 i32 DefaultHash32 must_hm new DefaultHash32;
    let hm1 %HashMap i32 i32 DefaultHash32 must_hm insert hm0 1 1;
    free hm1;
    0
```
