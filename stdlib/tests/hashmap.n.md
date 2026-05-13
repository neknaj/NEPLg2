# stdlib/hashmap.n.md

## hashmap_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashmap_main\" count=14 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"empty len\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty missing key\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"empty get none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"unique len\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"contains 10\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"contains 5\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"missing contains 2\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"get 5 value\" expected=\"50\" actual=\"50\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"update get 5 value\" expected=\"55\" actual=\"55\" message=\"\"\nassertion index=9 status=ok kind=eq_i32 label=\"update keeps len\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"remove len\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"remove clears 10\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"missing remove returns error\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=bool label=\"free after insert completes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn must_hm <(Result<HashMap<i32,i32,DefaultHash32>, Diag>)*>HashMap<i32,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*> i32> ():
    let hm0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let hm0_len <i32> len &hm0;
    free hm0;

    let hm1 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let hm1_has <bool> contains &hm1 1;
    free hm1;

    let hm2 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let hm2_none <bool> is_none<i32> get &hm2 1;
    free hm2;

    let a0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a1 <HashMap<i32,i32,DefaultHash32>> must_hm insert a0 10 100;
    let a2 <HashMap<i32,i32,DefaultHash32>> must_hm insert a1 5 50;
    let a3 <HashMap<i32,i32,DefaultHash32>> must_hm insert a2 20 200;
    let a3_len <i32> len &a3;
    free a3;

    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm insert a4 10 100;
    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm insert a4 5 50;
    let a4 <HashMap<i32,i32,DefaultHash32>> must_hm insert a4 20 200;
    let a4_has <bool> contains &a4 10;
    free a4;

    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm insert a5 10 100;
    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm insert a5 5 50;
    let a5 <HashMap<i32,i32,DefaultHash32>> must_hm insert a5 20 200;
    let a5_has <bool> contains &a5 5;
    free a5;

    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm insert a6 10 100;
    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm insert a6 5 50;
    let a6 <HashMap<i32,i32,DefaultHash32>> must_hm insert a6 20 200;
    let a6_has <bool> contains &a6 2;
    free a6;

    let b0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let b1 <HashMap<i32,i32,DefaultHash32>> must_hm insert b0 5 50;
    let mut b1_value <i32> -1;
    match get &b1 5:
        Option::Some v:
            set b1_value v
        Option::None:
            ()
    free b1;

    let c0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let c1 <HashMap<i32,i32,DefaultHash32>> must_hm insert c0 5 50;
    let c2 <HashMap<i32,i32,DefaultHash32>> must_hm insert c1 5 55;
    let mut c2_value <i32> -1;
    match get &c2 5:
        Option::Some v:
            set c2_value v
        Option::None:
            ()
    free c2;

    let c3 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let c3 <HashMap<i32,i32,DefaultHash32>> must_hm insert c3 5 50;
    let c3 <HashMap<i32,i32,DefaultHash32>> must_hm insert c3 5 55;
    let c3_len <i32> len &c3;
    free c3;

    let d0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let d1 <HashMap<i32,i32,DefaultHash32>> must_hm insert d0 10 100;
    let d2 <HashMap<i32,i32,DefaultHash32>> must_hm insert d1 20 200;
    let d3 <HashMap<i32,i32,DefaultHash32>> must_hm remove d2 10;
    let d3_len <i32> len &d3;
    free d3;

    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm insert d4 10 100;
    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm insert d4 20 200;
    let d4 <HashMap<i32,i32,DefaultHash32>> must_hm remove d4 10;
    let d4_has <bool> contains &d4 10;
    free d4;

    let e0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let e1 <HashMap<i32,i32,DefaultHash32>> must_hm insert e0 10 100;
    let er <Result<HashMap<i32,i32,DefaultHash32>, Diag>> remove e1 999;
    let missing_err <bool> is_err<HashMap<i32,i32,DefaultHash32>, Diag> er;

    let f0 <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let f1 <HashMap<i32,i32,DefaultHash32>> must_hm insert f0 1 1;
    free f1;

    let report:
        test_report_new "hashmap_main"
        |> test_report_push assert_eq_i32 "empty len" 0 hm0_len
        |> test_report_push assert "empty missing key" not hm1_has
        |> test_report_push assert "empty get none" hm2_none
        |> test_report_push assert_eq_i32 "unique len" 3 a3_len
        |> test_report_push assert "contains 10" a4_has
        |> test_report_push assert "contains 5" a5_has
        |> test_report_push assert "missing contains 2" not a6_has
        |> test_report_push assert_eq_i32 "get 5 value" 50 b1_value
        |> test_report_push assert_eq_i32 "update get 5 value" 55 c2_value
        |> test_report_push assert_eq_i32 "update keeps len" 1 c3_len
        |> test_report_push assert_eq_i32 "remove len" 1 d3_len
        |> test_report_push assert "remove clears 10" not d4_has
        |> test_report_push assert "missing remove returns error" missing_err
        |> test_report_push assert "free after insert completes" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
