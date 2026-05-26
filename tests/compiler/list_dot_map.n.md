# 名前空間呼び出し（`::`）と alias 展開テスト

## namespace_pathsep_map_with_result

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"namespace_pathsep_map_with_result\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"result namespace map\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/result" as result
#import "core/math" as *
#import "std/test" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %impure fn unit i32 \unit:
    let r %Result i32 i32 result::ok 1;
    let mapped result::map r inc;
    let actual %i32 result::unwrap_ok mapped
    let report:
        test_report_new "namespace_pathsep_map_with_result"
        |> test_report_push assert_eq_i32 "result namespace map" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## list_namespace_map_with_list

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"list_namespace_map_with_list\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"list namespace map first value\" expected=\"31\" actual=\"31\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/list" as list
#import "alloc/collections/list" as *
#import "core/option" as option
#import "core/result" as result
#import "core/math" as *
#import "core/field" as *
#import "std/test" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %impure fn unit i32 \unit:
    let xs0 %List i32 result::unwrap_ok list::new;
    let xs %List i32 result::unwrap_ok list::push xs0 30;
    let ys %List i32 result::unwrap_ok list::map xs inc;
    let actual %i32 option::unwrap list::get &ys 0
    list::free ys;
    let report:
        test_report_new "list_namespace_map_with_list"
        |> test_report_push assert_eq_i32 "list namespace map first value" 31 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## result_map_with_star_alias_works

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"result_map_with_star_alias_works\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"result star alias map\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %impure fn unit i32 \unit:
    let r %Result i32 i32 ok 1;
    let mapped %Result i32 i32 map r inc;
    let actual %i32 unwrap_ok mapped
    let report:
        test_report_new "result_map_with_star_alias_works"
        |> test_report_push assert_eq_i32 "result star alias map" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_map_with_star_alias_works

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_map_with_star_alias_works\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"vec star alias map second value\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "core/option" as *
#import "core/math" as *
#import "core/field" as *
#import "std/test" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %impure fn unit i32 \unit:
    let xs0 %Vec i32 unwrap_ok new;
    let xs1 %Vec i32 unwrap_ok push xs0 1;
    let xs2 %Vec i32 unwrap_ok push xs1 2;
    let ys %Vec i32 unwrap_ok map xs2 inc;
    let out %i32 unwrap get &ys 1;
    free ys;
    let report:
        test_report_new "vec_map_with_star_alias_works"
        |> test_report_push assert_eq_i32 "vec star alias map second value" 3 out
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
