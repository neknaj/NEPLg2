# stdlib/hashmap_str.n.md

## hashmap_str_empty_observers

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

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 Diag HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let len_ok %bool eq len &hm 0;
    let missing_ok %bool not contains &hm "foo";
    let get_ok %bool is_none get &hm "foo";
    free hm;
    if and len_ok and missing_ok get_ok 0 1
```

## hashmap_str_insert_get_and_update

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 Diag HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 HashMapUpdateError str i32 DefaultHash32 HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap str i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm1 %HashMap str i32 DefaultHash32 must_hms insert hm0 "foo" 10;
    let hm2 %HashMap str i32 DefaultHash32 must_hms insert hm1 "bar" 20;
    let len_ok %bool eq len &hm2 2;
    let contains_ok %bool and contains &hm2 "foo" contains &hm2 "bar";
    let missing_ok %bool not contains &hm2 "baz";
    free hm2;

    let s1 %str concat "a" "b";
    let s2 %str concat "a" "b";
    let concat0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let concat1 %HashMap str i32 DefaultHash32 must_hms insert concat0 s1 30;
    let concat_ok %bool:
        match get &concat1 s2:
            Option::Some v:
                eq v 30
            Option::None:
                false
    free concat1;

    let upd0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let upd1 %HashMap str i32 DefaultHash32 must_hms insert upd0 "foo" 10;
    let upd2 %HashMap str i32 DefaultHash32 must_hms insert upd1 "foo" 11;
    let update_ok %bool:
        match get &upd2 "foo":
            Option::Some v:
                eq v 11
            Option::None:
                false
    free upd2;

    let ok %bool and:
        and len_ok contains_ok
        and missing_ok and concat_ok update_ok
    if ok 0 1
```

## hashmap_str_remove_and_missing_error

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

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 Diag HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 HashMapUpdateError str i32 DefaultHash32 HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap str i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm1 %HashMap str i32 DefaultHash32 must_hms insert hm0 "foo" 10;
    let hm2 %HashMap str i32 DefaultHash32 must_hms insert hm1 "bar" 20;
    let hm3 %HashMap str i32 DefaultHash32 must_hms remove hm2 "bar";
    let remove_ok %bool not contains &hm3 "bar";
    free hm3;

    let miss0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let miss1 %HashMap str i32 DefaultHash32 must_hms insert miss0 "foo" 10;
    let missing_err %bool:
        match remove miss1 "zzz":
            Result::Ok hm:
                free hm;
                false
            Result::Err e:
                let hm %HashMap str i32 DefaultHash32 hashmap_update_error_owner e;
                free hm;
                true
    if and remove_ok missing_err 0 1
```

## hashmap_str_free_smoke

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 Diag HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 HashMapUpdateError str i32 DefaultHash32 HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap str i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hm0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm1 %HashMap str i32 DefaultHash32 must_hms insert hm0 "x" 1;
    free hm1;
    0
```
