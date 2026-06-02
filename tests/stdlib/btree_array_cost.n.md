# btree_array_cost.n.md

`BTreeMap` / `BTreeSet` 互換実装を sorted-array collection として扱うための focused fixture です。
構築件数を分け、構築コストと構築後の検索コストを test JSON 上で別ケースとして見ます。

## sorted_array_map_insert_32

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn build_desc_map %impure fn i32 BTreeMap i32 i32 \n:
    let mut m %BTreeMap i32 i32 unwrap_ok sorted_array_map_new;
    let mut i %i32 n;
    while gt i 0:
        do:
            let k %i32 sub i 1;
            set m must_map sorted_array_map_insert m k mul k 10;
            set i sub i 1;
    m

fn main %impure fn void i32 \void:
    let m %BTreeMap i32 i32 build_desc_map 32;
    let ok %bool eq sorted_array_map_len &m 32;
    sorted_array_map_free m;
    if ok 0 1
```

## sorted_array_map_insert_128

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn build_desc_map %impure fn i32 BTreeMap i32 i32 \n:
    let mut m %BTreeMap i32 i32 unwrap_ok sorted_array_map_new;
    let mut i %i32 n;
    while gt i 0:
        do:
            let k %i32 sub i 1;
            set m must_map sorted_array_map_insert m k mul k 10;
            set i sub i 1;
    m

fn main %impure fn void i32 \void:
    let m %BTreeMap i32 i32 build_desc_map 128;
    let ok %bool eq sorted_array_map_len &m 128;
    sorted_array_map_free m;
    if ok 0 1
```

## sorted_array_map_lookup_after_128

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "core/math" as *

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn build_desc_map %impure fn i32 BTreeMap i32 i32 \n:
    let mut m %BTreeMap i32 i32 unwrap_ok sorted_array_map_new;
    let mut i %i32 n;
    while gt i 0:
        do:
            let k %i32 sub i 1;
            set m must_map sorted_array_map_insert m k mul k 10;
            set i sub i 1;
    m

fn main %impure fn void i32 \void:
    let m %BTreeMap i32 i32 build_desc_map 128;
    let value %Option i32 sorted_array_map_get &m 64;
    sorted_array_map_free m;
    match value:
        Option::Some v:
            if eq v 640 0 1
        Option::None:
            1
```

## sorted_array_set_insert_32

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn build_desc_set %impure fn i32 BTreeSet i32 \n:
    let mut s %BTreeSet i32 unwrap_ok sorted_array_set_new;
    let mut i %i32 n;
    while gt i 0:
        do:
            let k %i32 sub i 1;
            set s must_set sorted_array_set_insert s k;
            set i sub i 1;
    s

fn main %impure fn void i32 \void:
    let s %BTreeSet i32 build_desc_set 32;
    let ok %bool eq sorted_array_set_len &s 32;
    sorted_array_set_free s;
    if ok 0 1
```

## sorted_array_set_insert_128

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn build_desc_set %impure fn i32 BTreeSet i32 \n:
    let mut s %BTreeSet i32 unwrap_ok sorted_array_set_new;
    let mut i %i32 n;
    while gt i 0:
        do:
            let k %i32 sub i 1;
            set s must_set sorted_array_set_insert s k;
            set i sub i 1;
    s

fn main %impure fn void i32 \void:
    let s %BTreeSet i32 build_desc_set 128;
    let ok %bool eq sorted_array_set_len &s 128;
    sorted_array_set_free s;
    if ok 0 1
```

## sorted_array_set_contains_after_128

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn build_desc_set %impure fn i32 BTreeSet i32 \n:
    let mut s %BTreeSet i32 unwrap_ok sorted_array_set_new;
    let mut i %i32 n;
    while gt i 0:
        do:
            let k %i32 sub i 1;
            set s must_set sorted_array_set_insert s k;
            set i sub i 1;
    s

fn main %impure fn void i32 \void:
    let s %BTreeSet i32 build_desc_set 128;
    let ok %bool sorted_array_set_contains &s 64;
    sorted_array_set_free s;
    if ok 0 1
```
