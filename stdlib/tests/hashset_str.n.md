# stdlib/hashset_str.n.md

## hashset_str_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashset_str_main\" count=8 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"empty len\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty missing foo\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"unique len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"contains foo\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"contains bar\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"concat key contains\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"remove clears foo\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"missing remove returns error\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/hash/hash32" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn must_hss %impure fn Result HashSet str DefaultHash32 Diag HashSet str DefaultHash32 \r:
    unwrap_ok r

fn must_hss %impure fn Result HashSet str DefaultHash32 HashSetUpdateError str DefaultHash32 HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hs0 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs0_len %i32 len &hs0;
    free hs0;

    let hs1 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs1_has %bool contains &hs1 "foo";
    free hs1;

    let hs2 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs2 %HashSet str DefaultHash32 must_hss insert hs2 "foo";
    let hs2 %HashSet str DefaultHash32 must_hss insert hs2 "bar";
    let hs2 %HashSet str DefaultHash32 must_hss insert hs2 "foo";
    let hs2_len %i32 len &hs2;
    free hs2;

    let hs2a %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs2a %HashSet str DefaultHash32 must_hss insert hs2a "foo";
    let hs2a %HashSet str DefaultHash32 must_hss insert hs2a "bar";
    let hs2a_has %bool contains &hs2a "foo";
    free hs2a;

    let hs2b %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs2b %HashSet str DefaultHash32 must_hss insert hs2b "foo";
    let hs2b %HashSet str DefaultHash32 must_hss insert hs2b "bar";
    let hs2b_has %bool contains &hs2b "bar";
    free hs2b;

    let s1 %str concat "a" "b";
    let s2 %str concat "a" "b";
    let hs3 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs3 %HashSet str DefaultHash32 must_hss insert hs3 s1;
    let hs3_has %bool contains &hs3 s2;
    free hs3;

    let hs4 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs4 %HashSet str DefaultHash32 must_hss insert hs4 "foo";
    let hs4 %HashSet str DefaultHash32 must_hss remove hs4 "foo";
    let hs4_has %bool contains &hs4 "foo";
    free hs4;

    let hs5 %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hs5 %HashSet str DefaultHash32 must_hss insert hs5 "foo";
    let hs5_is_err %bool:
        match remove hs5 "zzz":
            Result::Ok hs:
                free hs;
                false
            Result::Err e:
                let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
                free hs;
                true

    let report:
        test_report_new "hashset_str_main"
        |> test_report_push assert_eq_i32 "empty len" 0 hs0_len
        |> test_report_push assert "empty missing foo" not hs1_has
        |> test_report_push assert_eq_i32 "unique len" 2 hs2_len
        |> test_report_push assert "contains foo" hs2a_has
        |> test_report_push assert "contains bar" hs2b_has
        |> test_report_push assert "concat key contains" hs3_has
        |> test_report_push assert "remove clears foo" not hs4_has
        |> test_report_push assert "missing remove returns error" hs5_is_err
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## hashset_str_free_smoke

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hashset_str_free_smoke\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free after string insert completes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn must_hss %impure fn Result HashSet str DefaultHash32 Diag HashSet str DefaultHash32 \r:
    unwrap_ok r

fn must_hss %impure fn Result HashSet str DefaultHash32 HashSetUpdateError str DefaultHash32 HashSet str DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet str DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let hsf %HashSet str DefaultHash32 must_hss new DefaultHash32;
    let hsf %HashSet str DefaultHash32 must_hss insert hsf "x";
    free hsf;
    let report:
        test_report_new "hashset_str_free_smoke"
        |> test_report_push assert "free after string insert completes" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
