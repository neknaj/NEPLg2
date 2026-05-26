# stdlib/bloom_filter.n.md

## bloom_filter_insert_and_contains

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bloom_filter_insert_and_contains\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"contains inserted item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"bloom filter len\" expected=\"64\" actual=\"64\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let bf %BloomFilter i32 DefaultHash32:
        unwrap_ok new DefaultHash32 64
        |> insert 4
        |> insert 9
        |> insert 15
    let ok0 %bool contains &bf 9;
    let size %i32 len &bf;
    free bf
    let report:
        test_report_new "bloom_filter_insert_and_contains"
        |> test_report_push assert "contains inserted item" ok0
        |> test_report_push assert_eq_i32 "bloom filter len" 64 size
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## bloom_filter_clear_and_invalid_len

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bloom_filter_clear_and_invalid_len\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"clear removes inserted item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"invalid len rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let bf0 %BloomFilter i32 DefaultHash32:
        unwrap_ok new DefaultHash32 64
        |> insert 7
    let bf1 %BloomFilter i32 DefaultHash32 clear bf0;
    let seen %bool contains<i32, DefaultHash32> &bf1 7;
    free bf1
    let ok0 %bool if seen false true;
    let bad %Result BloomFilter i32 DefaultHash32 Diag new DefaultHash32 0;
    let ok1 %bool is_err bad;
    let report:
        test_report_new "bloom_filter_clear_and_invalid_len"
        |> test_report_push assert "clear removes inserted item" ok0
        |> test_report_push assert "invalid len rejected" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
