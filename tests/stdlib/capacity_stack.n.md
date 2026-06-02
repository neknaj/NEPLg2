# capacity_stack

メモリ容量・スタック深さ・複合利用（文字列/enum/vec/再帰）の段階的な回帰テストです。

## stage1_recursive_depth_64

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stage1_recursive_depth_64\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"recursive depth\" expected=\"64\" actual=\"64\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn depth %fn i32 fn i32 i32 \n\acc:
    if le n 0:
        acc
    else:
        depth sub n 1 add acc 1

fn main %impure fn void i32 \void:
    let actual %i32 depth 64 0
    let report:
        test_report_new "stage1_recursive_depth_64"
        |> test_report_push assert_eq_i32 "recursive depth" 64 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stage2_recursive_depth_512

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stage2_recursive_depth_512\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"recursive depth\" expected=\"512\" actual=\"512\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn depth %fn i32 fn i32 i32 \n\acc:
    if le n 0:
        acc
    else:
        depth sub n 1 add acc 1

fn main %impure fn void i32 \void:
    let actual %i32 depth 512 0
    let report:
        test_report_new "stage2_recursive_depth_512"
        |> test_report_push assert_eq_i32 "recursive depth" 512 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stage3_vec_growth_4096

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stage3_vec_growth_4096\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"vec length after growth\" expected=\"4096\" actual=\"4096\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "alloc/collections/vec" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let mut v %Vec i32 uwok new;
    let mut i %i32 0;
    while lt i 4096:
        do:
            set v uwok push v i;
            set i add i 1;
    let actual %i32 len &v
    free v
    let report:
        test_report_new "stage3_vec_growth_4096"
        |> test_report_push assert_eq_i32 "vec length after growth" 4096 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stage4_mem_block_store_load

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stage4_mem_block_store_load\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"first loaded value\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"middle loaded value\" expected=\"512\" actual=\"512\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"last loaded value\" expected=\"1023\" actual=\"1023\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"loaded value sum\" expected=\"1535\" actual=\"1535\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let n %i32 1024;
    let mut a %i32 -1;
    let mut b %i32 -1;
    let mut c %i32 -1;
    match alloc_region<i32> n:
        Result::Err _e:
            unit
        Result::Ok region:
            let mid_off %i32 mul 512 4
            let last_off %i32 mul 1023 4
            match region_ptr_at<i32, i32> &region 0:
                Result::Err _:
                    unit
                Result::Ok first_ptr:
                    match region_ptr_at<i32, i32> &region mid_off:
                        Result::Err _:
                            unit
                        Result::Ok mid_ptr:
                            match region_ptr_at<i32, i32> &region last_off:
                                Result::Err _:
                                    unit
                                Result::Ok last_ptr:
                                    match store_i32 first_ptr 0:
                                        Result::Err _:
                                            unit
                                        Result::Ok _:
                                            match store_i32 mid_ptr 512:
                                                Result::Err _:
                                                    unit
                                                Result::Ok _:
                                                    match store_i32 last_ptr 1023:
                                                        Result::Err _:
                                                            unit
                                                        Result::Ok _:
                                                            match load_i32 first_ptr:
                                                                Option::Some value:
                                                                    set a value
                                                                Option::None:
                                                                    unit
                                                            match load_i32 mid_ptr:
                                                                Option::Some value:
                                                                    set b value
                                                                Option::None:
                                                                    unit
                                                            match load_i32 last_ptr:
                                                                Option::Some value:
                                                                    set c value
                                                                Option::None:
                                                                    unit
            match dealloc_region<i32> region:
                Result::Ok _:
                    unit
                Result::Err _:
                    unit
    let total %i32 add add a b c
    let report:
        test_report_new "stage4_mem_block_store_load"
        |> test_report_push assert_eq_i32 "first loaded value" 0 a
        |> test_report_push assert_eq_i32 "middle loaded value" 512 b
        |> test_report_push assert_eq_i32 "last loaded value" 1023 c
        |> test_report_push assert_eq_i32 "loaded value sum" 1535 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stage5_string_builder_len_3000

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stage5_string_builder_len_3000\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"string builder length\" expected=\"3000\" actual=\"3000\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "alloc/string" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let mut sb %StringBuilder string_builder_new;
    let mut i %i32 0;
    while lt i 1500:
        do:
            set sb sb_append sb "ab";
            set i add i 1;
    let out %str sb_build sb;
    let actual %i32 len out
    let report:
        test_report_new "stage5_string_builder_len_3000"
        |> test_report_push assert_eq_i32 "string builder length" 3000 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stage6_enum_vec_recursive_mix

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stage6_enum_vec_recursive_mix\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"enum vec length\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"recursive mix total\" expected=\"15\" actual=\"15\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *
#import "alloc/collections/vec" as *
#import "std/test" as *

enum Kind:
    A
    B

impl Clone for Kind:
    fn clone %fn &Kind Kind \x:
        *x

impl Copy for Kind:
    fn copy_mark %fn Kind Kind \x:
        x

fn depth %fn i32 i32 \n:
    if le n 0:
        0
    else:
        add 1 depth sub n 1

fn main %impure fn void i32 \void:
    let mut v %Vec Kind uwok new;
    set v uwok push v Kind::A;
    set v uwok push v Kind::B;
    set v uwok push v Kind::A;
    set v uwok push v Kind::B;
    set v uwok push v Kind::A;
    let n %i32 len &v;
    free v
    let total %i32 add n depth 10
    let report:
        test_report_new "stage6_enum_vec_recursive_mix"
        |> test_report_push assert_eq_i32 "enum vec length" 5 n
        |> test_report_push assert_eq_i32 "recursive mix total" 15 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
