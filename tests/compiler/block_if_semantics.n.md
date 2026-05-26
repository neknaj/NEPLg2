# block_if_semantics.rs 由来の doctest

このファイルは Rust テスト `block_if_semantics.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## epilogue_drop_preserves_return_value

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"epilogue_drop_preserves_return_value\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"epilogue preserves return value\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let x %i32 1;
    let report:
        test_report_new "epilogue_drop_preserves_return_value"
        |> test_report_push assert_eq_i32 "epilogue preserves return value" 1 x
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## match_arm_local_drop_preserves_return

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"match_arm_local_drop_preserves_return\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"match arm preserves return value\" expected=\"5\" actual=\"5\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/option" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let actual %i32 match some 5:
        Some v:
            let y v;
            v
        None:
            0
    let report:
        test_report_new "match_arm_local_drop_preserves_return"
        |> test_report_push assert_eq_i32 "match arm preserves return value" 5 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## trailing_semicolon_makes_block_unit_and_errors_for_return

neplg2:test[compile_fail]
diag_code: type.return.mismatch
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main %fn unit i32 \unit:
    add 1 2;
```

## no_semicolons_on_line_allowed

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"no_semicolons_on_line_allowed\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"last expression without semicolons\" expected=\"11\" actual=\"11\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let actual %i32 block:
        add 1 2
        add 3 4
        add 5 6
    let report:
        test_report_new "no_semicolons_on_line_allowed"
        |> test_report_push assert_eq_i32 "last expression without semicolons" 11 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## multiple_semicolons_on_line_allowed

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"multiple_semicolons_on_line_allowed\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"last expression after repeated semicolons\" expected=\"11\" actual=\"11\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let actual %i32 block:
        add 1 2;;
        add 3 4;;;
        add 5 6
    let report:
        test_report_new "multiple_semicolons_on_line_allowed"
        |> test_report_push assert_eq_i32 "last expression after repeated semicolons" 11 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
