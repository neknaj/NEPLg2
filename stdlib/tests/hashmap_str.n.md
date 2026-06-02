# stdlib/hashmap_str.n.md

## hashmap_str_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashmap_str_main\" count=11 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"empty len\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty missing foo\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"empty get none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"unique len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"contains foo\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"contains bar\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"missing baz\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"concat key get\" expected=\"30\" actual=\"30\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"update get foo\" expected=\"11\" actual=\"11\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"remove clears bar\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"missing remove returns error\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

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
    let hm0_len %i32 len &hm0;
    free hm0;

    let hm1 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm1_has %bool contains &hm1 "foo";
    free hm1;

    let hm2 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm2_got %Option i32 get &hm2 "foo";
    let hm2_none %bool is_none hm2_got;
    free hm2;

    let hm3 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm3 %HashMap str i32 DefaultHash32 must_hms insert hm3 "foo" 10;
    let hm3 %HashMap str i32 DefaultHash32 must_hms insert hm3 "bar" 20;
    let hm3_len %i32 len &hm3;
    free hm3;

    let hm3a %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm3a %HashMap str i32 DefaultHash32 must_hms insert hm3a "foo" 10;
    let hm3a %HashMap str i32 DefaultHash32 must_hms insert hm3a "bar" 20;
    let hm3a_has %bool contains &hm3a "foo";
    free hm3a;

    let hm3b %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm3b %HashMap str i32 DefaultHash32 must_hms insert hm3b "foo" 10;
    let hm3b %HashMap str i32 DefaultHash32 must_hms insert hm3b "bar" 20;
    let hm3b_has %bool contains &hm3b "bar";
    free hm3b;

    let hm3c %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm3c %HashMap str i32 DefaultHash32 must_hms insert hm3c "foo" 10;
    let hm3c %HashMap str i32 DefaultHash32 must_hms insert hm3c "bar" 20;
    let hm3c_has %bool contains &hm3c "baz";
    free hm3c;

    let s1 %str concat "a" "b";
    let s2 %str concat "a" "b";
    let hm4 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm4 %HashMap str i32 DefaultHash32 must_hms insert hm4 s1 30;
    let mut hm4_value %i32 -1;
    match get &hm4 s2:
        Option::Some v:
            set hm4_value v
        Option::None:
            unit
    free hm4;

    let hm5 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm5 %HashMap str i32 DefaultHash32 must_hms insert hm5 "foo" 10;
    let hm5 %HashMap str i32 DefaultHash32 must_hms insert hm5 "foo" 11;
    let mut hm5_value %i32 -1;
    match get &hm5 "foo":
        Option::Some v:
            set hm5_value v
        Option::None:
            unit
    free hm5;

    let hm6 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm6 %HashMap str i32 DefaultHash32 must_hms insert hm6 "foo" 10;
    let hm6 %HashMap str i32 DefaultHash32 must_hms insert hm6 "bar" 20;
    let hm6 %HashMap str i32 DefaultHash32 must_hms remove hm6 "bar";
    let hm6_has %bool contains &hm6 "bar";
    free hm6;

    let hm7 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hm7 %HashMap str i32 DefaultHash32 must_hms insert hm7 "foo" 10;
    let hm7_is_err %bool:
        match remove hm7 "zzz":
            Result::Ok hm:
                free hm;
                false
            Result::Err e:
                let hm %HashMap str i32 DefaultHash32 hashmap_update_error_owner e;
                free hm;
                true

    let report:
        test_report_new "hashmap_str_main"
        |> test_report_push assert_eq_i32 "empty len" 0 hm0_len
        |> test_report_push assert "empty missing foo" not hm1_has
        |> test_report_push assert "empty get none" hm2_none
        |> test_report_push assert_eq_i32 "unique len" 2 hm3_len
        |> test_report_push assert "contains foo" hm3a_has
        |> test_report_push assert "contains bar" hm3b_has
        |> test_report_push assert "missing baz" not hm3c_has
        |> test_report_push assert_eq_i32 "concat key get" 30 hm4_value
        |> test_report_push assert_eq_i32 "update get foo" 11 hm5_value
        |> test_report_push assert "remove clears bar" not hm6_has
        |> test_report_push assert "missing remove returns error" hm7_is_err
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## hashmap_str_free_smoke

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashmap_str_free_smoke\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free after string insert completes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

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
    let hmf %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let hmf %HashMap str i32 DefaultHash32 must_hms insert hmf "x" 1;
    free hmf;
    let report:
        test_report_new "hashmap_str_free_smoke"
        |> test_report_push assert "free after string insert completes" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
