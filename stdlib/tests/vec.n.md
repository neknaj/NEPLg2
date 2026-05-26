# stdlib/vec.n.md

## vec_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_main\" count=10 failed=0\nassertion index=0 status=ok kind=bool label=\"empty is_empty\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"with_capacity starts empty\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"single push len\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"triple push len\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"get 0\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"replace get 0\" expected=\"11\" actual=\"11\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"out of range none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"negative index none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"u8 get\" expected=\"65\" actual=\"65\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"free after vec operations\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/cast" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn unit i32 \unit:
    let v0_empty %Vec i32 unwrap_ok new;
    let v0_is_empty %bool is_empty &v0_empty;
    let v0_cap %Vec i32 unwrap_ok with_capacity 2;
    let v0_cap_starts_empty %bool is_empty &v0_cap;

    let v2 %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
    let v2_len %i32 len &v2;

    let v6 %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
        |> push 20
        |> unwrap_ok
        |> push 30
        |> unwrap_ok
    let v6_len %i32 len &v6;

    let g2 %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
        |> push 20
        |> unwrap_ok
    let mut g2_value %i32 -1;
    match get &g2 0:
        Option::Some x:
            set g2_value x
        Option::None:
            unit

    let s2 %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
        |> push 20
        |> unwrap_ok
    replace &s2 1 21;
    let s2_ref %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
        |> push 20
        |> unwrap_ok
    replace &s2_ref 0 11;
    let mut s2_ref_value %i32 -1;
    match get &s2_ref 0:
        Option::Some x:
            set s2_ref_value x
        Option::None:
            unit

    let o1 %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
    let o1_missing %Option i32 get &o1 2;
    let o1_none %bool is_none o1_missing;

    let p1 %Vec i32:
        unwrap_ok new
        |> push 10
        |> unwrap_ok
    let p1_missing %Option i32 get &p1 -1;
    let p1_none %bool is_none p1_missing;

    let u8_65 %u8 cast 65;
    let b1 %Vec u8:
        unwrap_ok new
        |> push u8_65
        |> unwrap_ok
    let mut b1_value %i32 -1;
    match get &b1 0:
        Option::Some x:
            set b1_value cast x
        Option::None:
            unit

    free v0_empty;
    free v0_cap;
    free v2;
    free v6;
    free g2;
    free s2;
    free s2_ref;
    free o1;
    free p1;
    free b1;

    let report:
        test_report_new "vec_main"
        |> test_report_push assert "empty is_empty" v0_is_empty
        |> test_report_push assert "with_capacity starts empty" v0_cap_starts_empty
        |> test_report_push assert_eq_i32 "single push len" 1 v2_len
        |> test_report_push assert_eq_i32 "triple push len" 3 v6_len
        |> test_report_push assert_eq_i32 "get 0" 10 g2_value
        |> test_report_push assert_eq_i32 "replace get 0" 11 s2_ref_value
        |> test_report_push assert "out of range none" o1_none
        |> test_report_push assert "negative index none" p1_none
        |> test_report_push assert_eq_i32 "u8 get" 65 b1_value
        |> test_report_push assert "free after vec operations" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_functional_helpers

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_functional_helpers\" count=8 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"map get 2\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"filter len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"filter get 1\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"fold sum\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"reduce sum\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"find gt two\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"any gt two\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"all even\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn inc %fn i32 i32 \x:
    add x 1

fn is_even %fn i32 bool \x:
    eq rem_s x 2 0

fn add_acc %fn i32 fn i32 i32 \acc\x:
    add acc x

fn gt_two %fn i32 bool \x:
    gt x 2

fn lt_four %fn i32 bool \x:
    lt x 4

fn main %impure fn unit i32 \unit:
    let mapped_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
    let mapped %Vec i32 unwrap_ok map mapped_src inc;
    let mut mapped_value %i32 -1;
    match get &mapped 2:
        Option::Some x:
            set mapped_value x
        Option::None:
            unit

    let filtered_len_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let filtered_len %Vec i32 unwrap_ok filter filtered_len_src is_even;
    let filtered_len_value %i32 len &filtered_len;
    let filtered_get_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let filtered_get %Vec i32 unwrap_ok filter filtered_get_src is_even;
    let mut filtered_value %i32 -1;
    match get &filtered_get 1:
        Option::Some x:
            set filtered_value x
        Option::None:
            unit

    let folded_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let folded_sum %i32 fold &folded_src 0 add_acc;

    let reduced_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let mut reduced_value %i32 -1;
    match reduce &reduced_src add_acc:
        Option::Some x:
            set reduced_value x
        Option::None:
            unit

    let find_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
    let mut find_value %i32 -1;
    match find &find_src gt_two:
        Option::Some x:
            set find_value x
        Option::None:
            unit

    let any_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
    let any_value %bool any &any_src gt_two;

    let all_src %Vec i32:
        unwrap_ok new
        |> push 2 |> uwok
        |> push 4 |> uwok
        |> push 6 |> uwok
    let all_value %bool all &all_src is_even;

    free mapped;
    free filtered_len;
    free filtered_get;
    free folded_src;
    free reduced_src;
    free find_src;
    free any_src;
    free all_src;
    let report:
        test_report_new "vec_functional_helpers"
        |> test_report_push assert_eq_i32 "map get 2" 4 mapped_value
        |> test_report_push assert_eq_i32 "filter len" 2 filtered_len_value
        |> test_report_push assert_eq_i32 "filter get 1" 4 filtered_value
        |> test_report_push assert_eq_i32 "fold sum" 10 folded_sum
        |> test_report_push assert_eq_i32 "reduce sum" 10 reduced_value
        |> test_report_push assert_eq_i32 "find gt two" 3 find_value
        |> test_report_push assert "any gt two" any_value
        |> test_report_push assert "all even" all_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_partition_helpers

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_partition_helpers\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"partition evens len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"partition evens get 1\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"partition odds len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"partition odds get 0\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn is_even %fn i32 bool \x:
    eq rem_s x 2 0

fn main %impure fn unit i32 \unit:
    let partition_even_len_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_even_len %VecPartition i32 unwrap_ok partition partition_even_len_src is_even;
    let evens_len_value %i32 vec_partition_matched_len &parts_even_len;
    let partition_even_get_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_even_get %VecPartition i32 unwrap_ok partition partition_even_get_src is_even;
    let mut evens_get_value %i32 -1;
    match vec_partition_matched_get &parts_even_get 1:
        Option::Some x:
            set evens_get_value x
        Option::None:
            unit
    let partition_odds_len_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_odds_len %VecPartition i32 unwrap_ok partition partition_odds_len_src is_even;
    let odds_len_value %i32 vec_partition_rest_len &parts_odds_len;
    let partition_odds_get_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    let parts_odds_get %VecPartition i32 unwrap_ok partition partition_odds_get_src is_even;
    let mut odds_get_value %i32 -1;
    match vec_partition_rest_get &parts_odds_get 0:
        Option::Some x:
            set odds_get_value x
        Option::None:
            unit

    vec_partition_free parts_even_len;
    vec_partition_free parts_even_get;
    vec_partition_free parts_odds_len;
    vec_partition_free parts_odds_get;
    let report:
        test_report_new "vec_partition_helpers"
        |> test_report_push assert_eq_i32 "partition evens len" 2 evens_len_value
        |> test_report_push assert_eq_i32 "partition evens get 1" 4 evens_get_value
        |> test_report_push assert_eq_i32 "partition odds len" 2 odds_len_value
        |> test_report_push assert_eq_i32 "partition odds get 0" 1 odds_get_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_prefix_helpers

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_prefix_helpers\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"take while len\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn lt_four %fn i32 bool \x:
    lt x 4

fn main %impure fn unit i32 \unit:
    let take_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 5 |> uwok
        |> push 6 |> uwok
    let taken %Vec i32 unwrap_ok take_while take_src lt_four;
    let taken_len %i32 len &taken;
    free taken;

    let report:
        test_report_new "vec_prefix_helpers"
        |> test_report_push assert_eq_i32 "take while len" 3 taken_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_drop_while_helper

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_drop_while_helper\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"drop while first\" expected=\"5\" actual=\"5\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn lt_four %fn i32 bool \x:
    lt x 4

fn main %impure fn unit i32 \unit:
    let drop_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 5 |> uwok
        |> push 6 |> uwok
    let dropped %Vec i32 unwrap_ok drop_while drop_src lt_four;
    let mut dropped_first %i32 -1;
    match get &dropped 0:
        Option::Some x:
            set dropped_first x
        Option::None:
            unit
    free dropped;

    let report:
        test_report_new "vec_drop_while_helper"
        |> test_report_push assert_eq_i32 "drop while first" 5 dropped_first
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_count_helper

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_count_helper\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"count even\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn is_even %fn i32 bool \x:
    eq rem_s x 2 0

fn main %impure fn unit i32 \unit:
    let count_src %Vec i32:
        unwrap_ok new
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
        |> push 5 |> uwok
    let even_count %i32 count &count_src is_even;
    free count_src;
    let report:
        test_report_new "vec_count_helper"
        |> test_report_push assert_eq_i32 "count even" 2 even_count
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
