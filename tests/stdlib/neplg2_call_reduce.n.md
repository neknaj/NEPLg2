# NEPLg2 self-host call reduction input boundary

## direct_call_and_fail_closed_errors

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "neplg2/core/check/expr" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let checks1 checks_push checks0 check_eq_i32 0 selfhost_check_expr_stage0
    let shown checks_print_report checks1
    checks_exit_code shown
```
