# stdlib/counting_bloom_filter.n.md

## counting_bloom_filter_insert_remove_contains

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"counting_bloom_filter_insert_remove_contains\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"contains inserted item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"counter len\" expected=\"64\" actual=\"64\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"counter len after remove\" expected=\"64\" actual=\"64\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let bf0 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok<CountingBloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 4
        |> insert 9
        |> insert 15
    let ok0 %bool contains &bf0 9;
    let size0 %i32 len &bf0;
    free bf0
    let bf1 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok<CountingBloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 4
        |> insert 9
        |> insert 15
        |> remove 9
    let size1 %i32 len &bf1;
    free bf1
    let report:
        test_report_new "counting_bloom_filter_insert_remove_contains"
        |> test_report_push assert "contains inserted item" ok0
        |> test_report_push assert_eq_i32 "counter len" 64 size0
        |> test_report_push assert_eq_i32 "counter len after remove" 64 size1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## counting_bloom_filter_clear

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"counting_bloom_filter_clear\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"clear removes inserted item\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let bf0 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok<CountingBloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 7
        |> clear
    let ok0 %bool not contains &bf0 7;
    free bf0
    let report:
        test_report_new "counting_bloom_filter_clear"
        |> test_report_push assert "clear removes inserted item" ok0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
