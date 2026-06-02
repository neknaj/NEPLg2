# match literal patterns

`match` の arm 見出しで整数 literal、bool literal、char literal、`_` wildcard を扱えることを確認します。

## i32_literal_arm_selects_matching_case

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"i32_literal_arm_selects_matching_case\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"i32 literal matching arm\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn classify %fn i32 i32 \x:
    match x:
        34:
            1
        92:
            2
        _:
            3

fn main %impure fn void i32 \void:
    let actual %i32 classify 92
    let report:
        test_report_new "i32_literal_arm_selects_matching_case"
        |> test_report_push assert_eq_i32 "i32 literal matching arm" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## i32_literal_arm_uses_wildcard_default

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"i32_literal_arm_uses_wildcard_default\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"i32 literal wildcard default\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn classify %fn i32 i32 \x:
    match x:
        34:
            1
        92:
            2
        _:
            3

fn main %impure fn void i32 \void:
    let actual %i32 classify 7
    let report:
        test_report_new "i32_literal_arm_uses_wildcard_default"
        |> test_report_push assert_eq_i32 "i32 literal wildcard default" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## bool_literal_arms_are_exhaustive

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bool_literal_arms_are_exhaustive\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bool false arm\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn classify %fn bool i32 \flag:
    match flag:
        true:
            1
        false:
            2

fn main %impure fn void i32 \void:
    let actual %i32 classify false
    let report:
        test_report_new "bool_literal_arms_are_exhaustive"
        |> test_report_push assert_eq_i32 "bool false arm" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## i32_duplicate_literal_is_rejected

neplg2:test[compile_fail]
diag_code: type.match.duplicate_arm
```neplg2
#target wasm
#entry main
#indent 4

fn main %fn void i32 \void:
    let x %i32 1
    match x:
        1:
            10
        1:
            20
        _:
            0
```

## i32_literal_match_requires_wildcard

neplg2:test[compile_fail]
diag_code: type.match.non_exhaustive
```neplg2
#target wasm
#entry main
#indent 4

fn main %fn void i32 \void:
    let x %i32 1
    match x:
        1:
            10
        2:
            20
```

## wildcard_must_be_last

neplg2:test[compile_fail]
diag_code: type.match.wildcard_not_last
```neplg2
#target wasm
#entry main
#indent 4

fn main %fn void i32 \void:
    let x %i32 1
    match x:
        _:
            0
        1:
            1
```

## char_literal_arm_selects_matching_case

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_literal_arm_selects_matching_case\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"char newline arm\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn classify %fn char i32 \c:
    match c:
        'a':
            1
        '\n':
            2
        _:
            3

fn main %impure fn void i32 \void:
    let actual %i32 classify '\n'
    let report:
        test_report_new "char_literal_arm_selects_matching_case"
        |> test_report_push assert_eq_i32 "char newline arm" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## char_literal_accepts_unicode_scalar

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_literal_accepts_unicode_scalar\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"unicode scalar char arm\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn main %impure fn void i32 \void:
    let c %char '\u{3042}'
    let actual %i32 match c:
        '\u{3042}':
            1
        _:
            0
    let report:
        test_report_new "char_literal_accepts_unicode_scalar"
        |> test_report_push assert_eq_i32 "unicode scalar char arm" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## char_match_rejects_integer_arm

neplg2:test[compile_fail]
diag_code: type.match.pattern_unsupported
```neplg2
#target wasm
#entry main
#indent 4

fn main %fn void i32 \void:
    let c %char 'A'
    match c:
        65:
            1
        _:
            0
```

## char_literal_arm_matches_i32_code_point

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_literal_arm_matches_i32_code_point\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"char literal matches i32 code point\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn main %impure fn void i32 \void:
    let x %i32 65
    let actual %i32 match x:
        'A':
            1
        _:
            0
    let report:
        test_report_new "char_literal_arm_matches_i32_code_point"
        |> test_report_push assert_eq_i32 "char literal matches i32 code point" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## char_literal_arm_matches_u8_code_point

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_literal_arm_matches_u8_code_point\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"char literal matches u8 code point\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn classify %fn u8 i32 \x:
    match x:
        '\n':
            1
        _:
            0

fn main %impure fn void i32 \void:
    let actual %i32 classify '\n'
    let report:
        test_report_new "char_literal_arm_matches_u8_code_point"
        |> test_report_push assert_eq_i32 "char literal matches u8 code point" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## char_literal_function_argument_uses_integer_context

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_literal_function_argument_uses_integer_context\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"char literal function argument i32\" expected=\"65\" actual=\"65\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

fn takes_i32 %fn i32 i32 \x:
    x

fn main %impure fn void i32 \void:
    let actual %i32 takes_i32 'A'
    let report:
        test_report_new "char_literal_function_argument_uses_integer_context"
        |> test_report_push assert_eq_i32 "char literal function argument i32" 65 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## char_literal_backspace_and_formfeed_escapes_compile

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"char_literal_backspace_and_formfeed_escapes_compile\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"backspace plus formfeed\" expected=\"20\" actual=\"20\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let b %i32 '\b'
    let f %i32 '\f'
    let actual %i32 add b f
    let report:
        test_report_new "char_literal_backspace_and_formfeed_escapes_compile"
        |> test_report_push assert_eq_i32 "backspace plus formfeed" 20 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
