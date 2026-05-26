# stdlib/list.n.md

## list_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"list_main\" count=15 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"empty len\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"cons len 1\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"cons len 2\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"mk len\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"get 0\" expected=\"30\" actual=\"30\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"get 1\" expected=\"20\" actual=\"20\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"get 2\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"get 3 none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"get 100 none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"get negative none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"head\" expected=\"30\" actual=\"30\" message=\"\"\nassertion index=11 status=ok kind=eq_i32 label=\"tail head\" expected=\"20\" actual=\"20\" message=\"\"\nassertion index=12 status=ok kind=eq_i32 label=\"reverse get 0\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=13 status=ok kind=eq_i32 label=\"reverse get 2\" expected=\"30\" actual=\"30\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"free after mk completes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn mk %impure fn unit List i32 \unit:
    let l0 %List i32 unwrap_ok<List<i32>, Diag> new<i32>;
    let l1 %List i32 uwok cons<i32> 10 l0;
    let l2 %List i32 uwok cons<i32> 20 l1;
    uwok cons<i32> 30 l2

fn main %impure fn unit i32 \unit:
    let l0 %List i32 unwrap_ok<List<i32>, Diag> new<i32>;
    let l0_len %i32 len<i32> &l0;
    free<i32> l0;

    let l0a %List i32 unwrap_ok<List<i32>, Diag> new<i32>;
    let l1 %List i32 uwok cons<i32> 10 l0a;
    let l1_len %i32 len<i32> &l1;
    free<i32> l1;

    let l0b %List i32 unwrap_ok<List<i32>, Diag> new<i32>;
    let l1b %List i32 uwok cons<i32> 10 l0b;
    let l2 %List i32 uwok cons<i32> 20 l1b;
    let l2_len %i32 len<i32> &l2;
    free<i32> l2;

    let l3 %List i32 mk;
    let l3_len %i32 len<i32> &l3;
    free<i32> l3;

    let l3_0 %List i32 mk;
    let mut l3_0_value %i32 -1;
    match get<i32> &l3_0 0:
        Option::Some x:
            set l3_0_value x
        Option::None:
            unit
    free<i32> l3_0;

    let l3_1 %List i32 mk;
    let mut l3_1_value %i32 -1;
    match get<i32> &l3_1 1:
        Option::Some x:
            set l3_1_value x
        Option::None:
            unit
    free<i32> l3_1;

    let l3_2 %List i32 mk;
    let mut l3_2_value %i32 -1;
    match get<i32> &l3_2 2:
        Option::Some x:
            set l3_2_value x
        Option::None:
            unit
    free<i32> l3_2;

    let l3_3 %List i32 mk;
    let l3_3_none %bool is_none<i32> get<i32> &l3_3 3;
    free<i32> l3_3;

    let l3_100 %List i32 mk;
    let l3_100_none %bool is_none<i32> get<i32> &l3_100 100;
    free<i32> l3_100;

    let l3_n1 %List i32 mk;
    let l3_n1_none %bool is_none<i32> get<i32> &l3_n1 -1;
    free<i32> l3_n1;

    let l3h %List i32 mk;
    let mut l3h_value %i32 -1;
    match head<i32> &l3h:
        Option::Some x:
            set l3h_value x
        Option::None:
            unit
    free<i32> l3h;

    let l3t %List i32 mk;
    let mut l3t_head_value %i32 -1;
    match tail<i32> l3t:
        Option::Some l3_tail:
            match head<i32> &l3_tail:
                Option::Some x:
                    set l3t_head_value x
                Option::None:
                    unit
            free<i32> l3_tail
        Option::None:
            unit

    let l3r0 %List i32 mk;
    let l_rev %List i32 reverse<i32> l3r0;
    let mut l_rev0_value %i32 -1;
    match get<i32> &l_rev 0:
        Option::Some x:
            set l_rev0_value x
        Option::None:
            unit
    free<i32> l_rev;

    let l3r1 %List i32 mk;
    let l_rev2 %List i32 reverse<i32> l3r1;
    let mut l_rev2_value %i32 -1;
    match get<i32> &l_rev2 2:
        Option::Some x:
            set l_rev2_value x
        Option::None:
            unit
    free<i32> l_rev2;

    let lf %List i32 mk;
    free<i32> lf;

    let report:
        test_report_new "list_main"
        |> test_report_push assert_eq_i32 "empty len" 0 l0_len
        |> test_report_push assert_eq_i32 "cons len 1" 1 l1_len
        |> test_report_push assert_eq_i32 "cons len 2" 2 l2_len
        |> test_report_push assert_eq_i32 "mk len" 3 l3_len
        |> test_report_push assert_eq_i32 "get 0" 30 l3_0_value
        |> test_report_push assert_eq_i32 "get 1" 20 l3_1_value
        |> test_report_push assert_eq_i32 "get 2" 10 l3_2_value
        |> test_report_push assert "get 3 none" l3_3_none
        |> test_report_push assert "get 100 none" l3_100_none
        |> test_report_push assert "get negative none" l3_n1_none
        |> test_report_push assert_eq_i32 "head" 30 l3h_value
        |> test_report_push assert_eq_i32 "tail head" 20 l3t_head_value
        |> test_report_push assert_eq_i32 "reverse get 0" 10 l_rev0_value
        |> test_report_push assert_eq_i32 "reverse get 2" 30 l_rev2_value
        |> test_report_push assert "free after mk completes" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## list_functional_helpers

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"list_functional_helpers\" count=10 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"map get 0\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"map get 3\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"filter len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"filter get 0\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"filter get 1\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"fold sum\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"reduce sum\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"find gt two\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"any gt two\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"all even false\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn mk %impure fn unit List i32 \unit:
    let xs %List i32:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 4 |> uwok
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    xs

fn inc %fn i32 i32 \x:
    add x 1

fn is_even %fn i32 bool \x:
    eq rem_s x 2 0

fn add_acc %fn i32 fn i32 i32 \acc\x:
    add acc x

fn gt_two %fn i32 bool \x:
    gt x 2

fn main %impure fn unit i32 \unit:
    let mapped_src0 %List i32 mk;
    let mapped0 %List i32 uwok map<i32,i32> mapped_src0 inc;
    let mut mapped0_value %i32 -1;
    match get<i32> &mapped0 0:
        Option::Some x:
            set mapped0_value x
        Option::None:
            unit
    free<i32> mapped0;

    let mapped_src3 %List i32 mk;
    let mapped3 %List i32 uwok map<i32,i32> mapped_src3 inc;
    let mut mapped3_value %i32 -1;
    match get<i32> &mapped3 3:
        Option::Some x:
            set mapped3_value x
        Option::None:
            unit
    free<i32> mapped3;

    let filtered_len_src %List i32 mk;
    let filtered_len_list %List i32 uwok filter<i32> filtered_len_src is_even;
    let filtered_len %i32 len<i32> &filtered_len_list;
    free<i32> filtered_len_list;

    let filtered_src0 %List i32 mk;
    let filtered0 %List i32 uwok filter<i32> filtered_src0 is_even;
    let mut filtered0_value %i32 -1;
    match get<i32> &filtered0 0:
        Option::Some x:
            set filtered0_value x
        Option::None:
            unit
    free<i32> filtered0;

    let filtered_src1 %List i32 mk;
    let filtered1 %List i32 uwok filter<i32> filtered_src1 is_even;
    let mut filtered1_value %i32 -1;
    match get<i32> &filtered1 1:
        Option::Some x:
            set filtered1_value x
        Option::None:
            unit
    free<i32> filtered1;

    let folded_src %List i32 mk;
    let folded_sum %i32 fold<i32,i32> &folded_src 0 add_acc;
    free<i32> folded_src;

    let reduced_src %List i32 mk;
    let mut reduced_value %i32 -1;
    match reduce<i32> &reduced_src add_acc:
        Option::Some x:
            set reduced_value x
        Option::None:
            unit
    free<i32> reduced_src;

    let find_src %List i32 mk;
    let mut find_value %i32 -1;
    match find<i32> &find_src gt_two:
        Option::Some x:
            set find_value x
        Option::None:
            unit
    free<i32> find_src;

    let any_src %List i32 mk;
    let any_value %bool any<i32> &any_src gt_two;
    free<i32> any_src;

    let all_src %List i32 mk;
    let all_value %bool all<i32> &all_src is_even;
    free<i32> all_src;

    let report:
        test_report_new "list_functional_helpers"
        |> test_report_push assert_eq_i32 "map get 0" 2 mapped0_value
        |> test_report_push assert_eq_i32 "map get 3" 5 mapped3_value
        |> test_report_push assert_eq_i32 "filter len" 2 filtered_len
        |> test_report_push assert_eq_i32 "filter get 0" 2 filtered0_value
        |> test_report_push assert_eq_i32 "filter get 1" 4 filtered1_value
        |> test_report_push assert_eq_i32 "fold sum" 10 folded_sum
        |> test_report_push assert_eq_i32 "reduce sum" 10 reduced_value
        |> test_report_push assert_eq_i32 "find gt two" 3 find_value
        |> test_report_push assert "any gt two" any_value
        |> test_report_push assert "all even false" not all_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
