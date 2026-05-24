# block と `;` の値・型テスト

`;` の有無でブロック/式の値がどう変わるか、
および `plan.md` の単数行/複数行制約を検証します。

## block_colon_returns_last_expr_value

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_colon_returns_last_expr_value\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"block returns last expr\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let x %i32 block:
        let a %i32 1;
        let b %i32 2;
        add a b
    let report:
        test_report_new "block_colon_returns_last_expr_value"
        |> test_report_push assert_eq_i32 "block returns last expr" 3 x
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_colon_last_semicolon_makes_unit_and_causes_type_error

neplg2:test[compile_fail]
diag_code: type.annotation.mismatch
```neplg2
#entry main
#indent 4
#target core

fn main %fn () i32 \():
    let x %i32 block:
        1;
    x
```

## single_line_block_last_semicolon_makes_unit_and_causes_type_error

neplg2:test[compile_fail]
diag_code: type.annotation.mismatch
```neplg2
#entry main
#indent 4
#target core

fn main %fn () i32 \():
    let x %i32 block 1;
    x
```

## block_colon_last_semicolon_can_be_used_with_unit_context

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_colon_last_semicolon_can_be_used_with_unit_context\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"unit context accepts trailing semicolon\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let _u %() block:
        add 1 2;
    let report:
        test_report_new "block_colon_last_semicolon_can_be_used_with_unit_context"
        |> test_report_push assert_eq_i32 "unit context accepts trailing semicolon" 1 1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## semicolon_requires_single_stack_growth_before_drop

neplg2:test[compile_fail]
diag_code: type.stack.extra_values
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let _u %() block:
        add 1 2 3;
    0
```

## if_result_expected_i32_without_semicolon_then_ok

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"if_result_expected_i32_without_semicolon_then_ok\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"if returns branch value\" expected=\"30\" actual=\"30\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let v %i32 if:
        true
        then:
            add 10 20
        else:
            0
    let report:
        test_report_new "if_result_expected_i32_without_semicolon_then_ok"
        |> test_report_push assert_eq_i32 "if returns branch value" 30 v
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## if_result_expected_i32_but_then_branch_semicolon_makes_unit

neplg2:test[compile_fail]
diag_code: type.annotation.mismatch
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let v %i32 if:
        true
        then:
            add 10 20;
        else:
            0
    v
```

## block_last_semicolon_breaks_function_return_type

neplg2:test[compile_fail]
diag_code: type.annotation.mismatch
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc %fn () i32 \():
    block:
        add 1 2;

fn main %fn () i32 \():
    calc
```

## single_line_let_with_semicolon_is_allowed

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"single_line_let_with_semicolon_is_allowed\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single line let value\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let x %i32 add 1 2;
    let actual %i32 if eq x 3 1 0
    let report:
        test_report_new "single_line_let_with_semicolon_is_allowed"
        |> test_report_push assert_eq_i32 "single line let value" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## multiline_let_with_trailing_semicolon_is_rejected

neplg2:test[compile_fail]
diag_code: parser.token.unexpected
```neplg2
#entry main
#indent 4
#target core

fn main %fn () i32 \():
    let x %i32 if:
        true
        then 1
        else 2;
    x
```
