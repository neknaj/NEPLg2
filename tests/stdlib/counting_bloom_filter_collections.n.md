# tests/counting_bloom_filter_collections.n.md

## counting_bloom_filter_pipe_usage

[目的/もくてき]:
- `CountingBloomFilter` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `clear`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"counting_bloom_filter_pipe_usage\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"contains retained item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"counter len\" expected=\"128\" actual=\"128\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"clear removes inserted item\" expected=\"true\" actual=\"true\" message=\"\"\n"
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

fn main %impure fn void i32 \void:
    let bf0 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok new DefaultHash32 128
        |> insert 3
        |> insert 8
        |> insert 21
        |> remove 8
    let ok0 %bool contains &bf0 3;
    let size0 %i32 len &bf0;
    free bf0
    let bf1 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok new DefaultHash32 128
        |> insert 3
        |> insert 8
        |> insert 21
        |> clear
    let ok2 %bool not contains &bf1 8;
    free bf1
    let report:
        test_report_new "counting_bloom_filter_pipe_usage"
        |> test_report_push assert "contains retained item" ok0
        |> test_report_push assert_eq_i32 "counter len" 128 size0
        |> test_report_push assert "clear removes inserted item" ok2
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## counting_bloom_filter_free_releases_owned_storage

[目的/もくてき]:
- `CountingBloomFilter.free` が owner [管理/かんり]している counter [配列/はいれつ]を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `insert`
- `remove`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"counting_bloom_filter_free_releases_owned_storage\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free permits later allocation\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let bf0 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok new DefaultHash32 128
        |> insert 8
        |> remove 8
    free bf0
    let bf1 %CountingBloomFilter i32 DefaultHash32:
        unwrap_ok new DefaultHash32 128
        |> insert 21
    free bf1
    let report:
        test_report_new "counting_bloom_filter_free_releases_owned_storage"
        |> test_report_push assert "free permits later allocation" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## counting_bloom_filter_rejects_non_positive_length

[目的/もくてき]:
- counter [長/ちょう]が 0 [以下/いか]のとき、counter owner を[作/つく]らず `Result::Err` を[返/かえ]すことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- invalid length
- `Result`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"counting_bloom_filter_rejects_non_positive_length\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"non-positive length rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %Result CountingBloomFilter i32 DefaultHash32 Diag new DefaultHash32 0
    let ok %bool match result:
        Result::Ok bf:
            free bf
            false
        Result::Err _d:
            true
    let report:
        test_report_new "counting_bloom_filter_rejects_non_positive_length"
        |> test_report_push assert "non-positive length rejected" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
