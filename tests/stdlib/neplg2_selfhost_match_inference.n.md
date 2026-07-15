# NEPLg2 self-host Match inference

## generic_first_arm_retry

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "neplg2/core/check/expr/stage1_match_actual_direct_fixture" as *

fn main %impure fn void i32 \void:
    if selfhost_check_expr_stage1_fixture_match_actual_arm_retry_case 1 0 1
```
