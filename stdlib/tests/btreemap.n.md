# stdlib/btreemap.n.md

## btreemap_insert_and_len

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }
#import "core/result" as *

fn must_map <(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d <Diag> btreemap_insert_error_diag<i32,i32> &e
            btreemap_insert_error_owner<i32,i32> e

fn main <()*>i32> ():
    let mut checks checks_new;

    let m0 <BTreeMap<i32,i32>>:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 5 50
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 1 10
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 3 30
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
    set checks checks_push checks check_eq_i32 3 len<i32,i32> &m0;
    free<i32,i32> m0;

    let shown checks_print_report checks;
    checks_exit_code shown
```

## btreemap_insert_growth_boundary

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn must_map <(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d <Diag> btreemap_insert_error_diag<i32,i32> &e
            btreemap_insert_error_owner<i32,i32> e

fn main <()*>i32> ():
    let mut checks checks_new;

    let m0 <BTreeMap<i32,i32>>:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 0 0
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 1 10
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 2 20
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 3 30
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 4 40
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 5 50
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 6 60
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 7 70
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 8 80
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
    match get<i32,i32> &m0 8:
        Option::Some v:
            set checks checks_push checks check_eq_i32 80 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "btreemap grow boundary lost inserted value";
    free<i32,i32> m0;

    let shown checks_print_report checks;
    checks_exit_code shown
```

## btreemap_get_and_remove

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "core/math" as *

fn must_map <(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d <Diag> btreemap_insert_error_diag<i32,i32> &e
            btreemap_insert_error_owner<i32,i32> e

fn main <()*>i32> ():
    let mut checks checks_new;

    let m0 <BTreeMap<i32,i32>>:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 3 30
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 1 10
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
    match get<i32,i32> &m0 3:
        Option::Some v:
            set checks checks_push checks check_eq_i32 30 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "btreemap get did not return inserted value";
    free<i32,i32> m0;

    let m1 <BTreeMap<i32,i32>>:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 3 30
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 1 10
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> remove<i32,i32> 1
    set checks checks_push checks check_eq_i32 1 len<i32,i32> &m1;
    free<i32,i32> m1;

    let shown checks_print_report checks;
    checks_exit_code shown
```

## btreemap_update_existing

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "core/math" as *

fn must_map <(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d <Diag> btreemap_insert_error_diag<i32,i32> &e
            btreemap_insert_error_owner<i32,i32> e

fn main <()*>i32> ():
    let mut checks checks_new;

    let m0 <BTreeMap<i32,i32>>:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 7 70
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 7 71
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
    match get<i32,i32> &m0 7:
        Option::Some v:
            set checks checks_push checks check_eq_i32 71 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "btreemap update did not overwrite value";
    free<i32,i32> m0;

    let shown checks_print_report checks;
    checks_exit_code shown
```

## btreemap_borrowed_reads_keep_owner

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn must_map <(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d <Diag> btreemap_insert_error_diag<i32,i32> &e
            btreemap_insert_error_owner<i32,i32> e

fn main <()*>i32> ():
    let mut checks checks_new;

    let m <BTreeMap<i32,i32>>:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 2 20
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
        |> insert<i32,i32> 1 10
        |> unwrap_ok<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>
    set checks checks_push checks check_eq_i32 2 len<i32,i32> &m;
    set checks checks_push checks check contains<i32,i32> &m 1;
    match get<i32,i32> &m 2:
        Option::Some v:
            set checks checks_push checks check_eq_i32 20 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "btreemap get lost owner-backed value";
    free m;

    let shown checks_print_report checks;
    checks_exit_code shown
```
