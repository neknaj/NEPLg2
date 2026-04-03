# block と `;` の値・型テスト

`;` の有無でブロック/式の値がどう変わるか、
および `plan.md` の単数行/複数行制約を検証します。

## block_colon_returns_last_expr_value

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> block:
        let a <i32> 1;
        let b <i32> 2;
        add a b
    x
```

## block_colon_last_semicolon_makes_unit_and_causes_type_error

neplg2:test[compile_fail]
diag_id: 3004
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let x <i32> block:
        1;
    x
```

## single_line_block_last_semicolon_makes_unit_and_causes_type_error

neplg2:test[compile_fail]
diag_id: 3004
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let x <i32> block 1;
    x
```

## block_colon_last_semicolon_can_be_used_with_unit_context

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let _u <()> block:
        add 1 2;
    1
```

## semicolon_requires_single_stack_growth_before_drop

neplg2:test[compile_fail]
diag_id: 3016
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let _u <()> block:
        add 1 2 3;
    0
```

## if_result_expected_i32_without_semicolon_then_ok

neplg2:test
ret: 30
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            add 10 20
        else:
            0
    v
```

## if_result_expected_i32_but_then_branch_semicolon_makes_unit

neplg2:test[compile_fail]
diag_id: 3004
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            add 10 20;
        else:
            0
    v
```

## block_last_semicolon_breaks_function_return_type

neplg2:test[compile_fail]
diag_id: 3004
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <()->i32> ():
    block:
        add 1 2;

fn main <()->i32> ():
    calc
```

## single_line_let_with_semicolon_is_allowed

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> add 1 2;
    if eq x 3 1 0
```

## multiline_let_with_trailing_semicolon_is_rejected

neplg2:test[compile_fail]
diag_id: 2002
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let x <i32> if:
        true
        then 1
        else 2;
    x
```
