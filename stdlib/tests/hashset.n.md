# stdlib/hashset.n.md

## hashset_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashset_main\" count=8 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"empty len\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty missing value\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"unique len\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"contains 5\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"contains 1\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"contains 9\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"remove clears 5\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"missing remove returns error\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/hash/hash32" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn must_hs <(Result<HashSet<i32,DefaultHash32>, Diag>)*>HashSet<i32,DefaultHash32>> (r):
    unwrap_ok<HashSet<i32,DefaultHash32>, Diag> r

fn must_hs <(Result<HashSet<i32,DefaultHash32>, HashSetUpdateError<i32,DefaultHash32>>)*>HashSet<i32,DefaultHash32>> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs <HashSet<i32,DefaultHash32>> hashset_update_error_owner<i32,DefaultHash32> e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let hs0 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs0_len <i32> len &hs0;
    free hs0;

    let hs1 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs1_has <bool> contains &hs1 5;
    free hs1;

    let hs2 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 5;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 1;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 9;
    let hs2 <HashSet<i32,DefaultHash32>> must_hs insert hs2 5;
    let hs2_len <i32> len &hs2;
    free hs2;

    let hs2a <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 5;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 1;
    let hs2a <HashSet<i32,DefaultHash32>> must_hs insert hs2a 9;
    let hs2a_has <bool> contains &hs2a 5;
    free hs2a;

    let hs2b <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 5;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 1;
    let hs2b <HashSet<i32,DefaultHash32>> must_hs insert hs2b 9;
    let hs2b_has <bool> contains &hs2b 1;
    free hs2b;

    let hs2c <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 5;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 1;
    let hs2c <HashSet<i32,DefaultHash32>> must_hs insert hs2c 9;
    let hs2c_has <bool> contains &hs2c 9;
    free hs2c;

    let hs3 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 5;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 1;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs insert hs3 9;
    let hs3 <HashSet<i32,DefaultHash32>> must_hs remove hs3 5;
    let hs3_has <bool> contains &hs3 5;
    free hs3;

    let hs4 <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs4 <HashSet<i32,DefaultHash32>> must_hs insert hs4 5;
    let missing_err <bool>:
        match remove hs4 99:
            Result::Ok hs:
                free hs;
                false
            Result::Err e:
                let hs <HashSet<i32,DefaultHash32>> hashset_update_error_owner<i32,DefaultHash32> e;
                free hs;
                true

    let report:
        test_report_new "hashset_main"
        |> test_report_push assert_eq_i32 "empty len" 0 hs0_len
        |> test_report_push assert "empty missing value" not hs1_has
        |> test_report_push assert_eq_i32 "unique len" 3 hs2_len
        |> test_report_push assert "contains 5" hs2a_has
        |> test_report_push assert "contains 1" hs2b_has
        |> test_report_push assert "contains 9" hs2c_has
        |> test_report_push assert "remove clears 5" not hs3_has
        |> test_report_push assert "missing remove returns error" missing_err
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## hashset_free_smoke

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashset_free_smoke\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free after insert completes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn must_hs <(Result<HashSet<i32,DefaultHash32>, Diag>)*>HashSet<i32,DefaultHash32>> (r):
    unwrap_ok<HashSet<i32,DefaultHash32>, Diag> r

fn must_hs <(Result<HashSet<i32,DefaultHash32>, HashSetUpdateError<i32,DefaultHash32>>)*>HashSet<i32,DefaultHash32>> (r):
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs <HashSet<i32,DefaultHash32>> hashset_update_error_owner<i32,DefaultHash32> e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let hsf <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hsf <HashSet<i32,DefaultHash32>> must_hs insert hsf 5;
    free hsf;
    let report:
        test_report_new "hashset_free_smoke"
        |> test_report_push assert "free after insert completes" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
