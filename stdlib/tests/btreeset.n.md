# stdlib/btreeset.n.md

## btreeset_insert_and_len

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreeset_insert_and_len\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after inserts\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/result" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn main %impure fn () i32 \():
    let s0 %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> insert<i32> 3
        |> must_set
    let s0_len %i32 len<i32> &s0;
    free<i32> s0;

    let report:
        test_report_new "btreeset_insert_and_len"
        |> test_report_push assert_eq_i32 "len after inserts" 3 s0_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreeset_insert_growth_boundary

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreeset_insert_growth_boundary\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"contains inserted value after growth\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/result" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn main %impure fn () i32 \():
    let s0 %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 0
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> insert<i32> 2
        |> must_set
        |> insert<i32> 3
        |> must_set
        |> insert<i32> 4
        |> must_set
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 6
        |> must_set
        |> insert<i32> 7
        |> must_set
        |> insert<i32> 8
        |> must_set
    let s0_contains_8 %bool contains<i32> &s0 8;
    free<i32> s0;

    let report:
        test_report_new "btreeset_insert_growth_boundary"
        |> test_report_push assert "contains inserted value after growth" s0_contains_8
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreeset_contains_and_remove

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreeset_contains_and_remove\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"contains inserted value\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"removed value absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"len after remove\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/result" as *
#import "core/math" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn main %impure fn () i32 \():
    let s0 %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
    let s0_contains_1 %bool contains<i32> &s0 1;
    free<i32> s0;

    let s1 %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> remove<i32> 1
    let s1_missing_1 %bool not contains<i32> &s1 1;
    free<i32> s1;

    let s2 %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 1
        |> must_set
        |> remove<i32> 1
    let s2_len %i32 len<i32> &s2;
    free<i32> s2;

    let report:
        test_report_new "btreeset_contains_and_remove"
        |> test_report_push assert "contains inserted value" s0_contains_1
        |> test_report_push assert "removed value absent" s1_missing_1
        |> test_report_push assert_eq_i32 "len after remove" 1 s2_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreeset_duplicate_insert

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreeset_duplicate_insert\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after duplicate insert\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/result" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn main %impure fn () i32 \():
    let s0 %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 3
        |> must_set
        |> insert<i32> 3
        |> must_set
    let s0_len %i32 len<i32> &s0;
    free<i32> s0;

    let report:
        test_report_new "btreeset_duplicate_insert"
        |> test_report_push assert_eq_i32 "len after duplicate insert" 1 s0_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## btreeset_borrowed_reads_keep_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"btreeset_borrowed_reads_keep_owner\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"borrowed len keeps owner\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"contains borrowed key\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as *
#import "core/result" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn main %impure fn () i32 \():
    let s %BTreeSet i32:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 2
        |> must_set
        |> insert<i32> 1
        |> must_set
    let s_len %i32 len<i32> &s;
    let s_contains_1 %bool contains<i32> &s 1;
    free s;

    let report:
        test_report_new "btreeset_borrowed_reads_keep_owner"
        |> test_report_push assert_eq_i32 "borrowed len keeps owner" 2 s_len
        |> test_report_push assert "contains borrowed key" s_contains_1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
