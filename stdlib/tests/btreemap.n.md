# stdlib/btreemap.n.md

## btreemap_insert_and_len

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreemap_insert_and_len\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after inserts\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/result" as *

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn main %impure fn () i32 \():
    let m0 %BTreeMap i32 i32:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 5 50
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
        |> insert<i32,i32> 3 30
        |> must_map
    let m0_len %i32 len<i32,i32> &m0;
    free<i32,i32> m0;

    let report:
        test_report_new "btreemap_insert_and_len"
        |> test_report_push assert_eq_i32 "len after inserts" 3 m0_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreemap_insert_growth_boundary

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreemap_insert_growth_boundary\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"get inserted value after growth\" expected=\"80\" actual=\"80\" message=\"\"\n"
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

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn main %impure fn () i32 \():
    let m0 %BTreeMap i32 i32:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 0 0
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
        |> insert<i32,i32> 2 20
        |> must_map
        |> insert<i32,i32> 3 30
        |> must_map
        |> insert<i32,i32> 4 40
        |> must_map
        |> insert<i32,i32> 5 50
        |> must_map
        |> insert<i32,i32> 6 60
        |> must_map
        |> insert<i32,i32> 7 70
        |> must_map
        |> insert<i32,i32> 8 80
        |> must_map
    let mut get8_value %i32 -1;
    match get<i32,i32> &m0 8:
        Option::Some v:
            set get8_value v
        Option::None:
            ()
    free<i32,i32> m0;

    let report:
        test_report_new "btreemap_insert_growth_boundary"
        |> test_report_push assert_eq_i32 "get inserted value after growth" 80 get8_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreemap_get_and_remove

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreemap_get_and_remove\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"get returns inserted value\" expected=\"30\" actual=\"30\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"len after remove\" expected=\"1\" actual=\"1\" message=\"\"\n"
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

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn main %impure fn () i32 \():
    let m0 %BTreeMap i32 i32:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 3 30
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
    let mut get3_value %i32 -1;
    match get<i32,i32> &m0 3:
        Option::Some v:
            set get3_value v
        Option::None:
            ()
    free<i32,i32> m0;

    let m1 %BTreeMap i32 i32:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 3 30
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
        |> remove<i32,i32> 1
    let m1_len %i32 len<i32,i32> &m1;
    free<i32,i32> m1;

    let report:
        test_report_new "btreemap_get_and_remove"
        |> test_report_push assert_eq_i32 "get returns inserted value" 30 get3_value
        |> test_report_push assert_eq_i32 "len after remove" 1 m1_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreemap_update_existing

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreemap_update_existing\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"update overwrites value\" expected=\"71\" actual=\"71\" message=\"\"\n"
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

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn main %impure fn () i32 \():
    let m0 %BTreeMap i32 i32:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 7 70
        |> must_map
        |> insert<i32,i32> 7 71
        |> must_map
    let mut updated_value %i32 -1;
    match get<i32,i32> &m0 7:
        Option::Some v:
            set updated_value v
        Option::None:
            ()
    free<i32,i32> m0;

    let report:
        test_report_new "btreemap_update_existing"
        |> test_report_push assert_eq_i32 "update overwrites value" 71 updated_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreemap_borrowed_reads_keep_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreemap_borrowed_reads_keep_owner\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"borrowed len keeps owner\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"contains borrowed key\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"borrowed get keeps owner\" expected=\"20\" actual=\"20\" message=\"\"\n"
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

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn main %impure fn () i32 \():
    let m %BTreeMap i32 i32:
        unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
        |> insert<i32,i32> 2 20
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
    let m_len %i32 len<i32,i32> &m;
    let m_contains_1 %bool contains<i32,i32> &m 1;
    let mut get2_value %i32 -1;
    match get<i32,i32> &m 2:
        Option::Some v:
            set get2_value v
        Option::None:
            ()
    free m;

    let report:
        test_report_new "btreemap_borrowed_reads_keep_owner"
        |> test_report_push assert_eq_i32 "borrowed len keeps owner" 2 m_len
        |> test_report_push assert "contains borrowed key" m_contains_1
        |> test_report_push assert_eq_i32 "borrowed get keeps owner" 20 get2_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
