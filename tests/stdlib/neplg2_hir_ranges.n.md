# NEPLg2 self-host HIR range validation

## rejects_invalid_hir_ranges_with_typed_errors

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/hir/hir" as *
#import "std/test" as *

fn is_negative_count %fn Result SelfhostHirChildRange SelfhostHirRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostHirRangeBuildError::NegativeFirst:
                    false
                SelfhostHirRangeBuildError::NegativeCount:
                    true
                SelfhostHirRangeBuildError::NonCanonicalEmpty:
                    false
                SelfhostHirRangeBuildError::EndOverflow:
                    false
                SelfhostHirRangeBuildError::OutOfBounds:
                    false

fn is_noncanonical_empty %fn Result SelfhostHirChildRange SelfhostHirRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostHirRangeBuildError::NegativeFirst:
                    false
                SelfhostHirRangeBuildError::NegativeCount:
                    false
                SelfhostHirRangeBuildError::NonCanonicalEmpty:
                    true
                SelfhostHirRangeBuildError::EndOverflow:
                    false
                SelfhostHirRangeBuildError::OutOfBounds:
                    false

fn is_param_end_overflow %fn Result SelfhostHirParamRange SelfhostHirRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostHirRangeBuildError::NegativeFirst:
                    false
                SelfhostHirRangeBuildError::NegativeCount:
                    false
                SelfhostHirRangeBuildError::NonCanonicalEmpty:
                    false
                SelfhostHirRangeBuildError::EndOverflow:
                    true
                SelfhostHirRangeBuildError::OutOfBounds:
                    false

fn is_child_out_of_bounds %fn Result SelfhostHirChildRange SelfhostHirRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostHirRangeBuildError::NegativeFirst:
                    false
                SelfhostHirRangeBuildError::NegativeCount:
                    false
                SelfhostHirRangeBuildError::NonCanonicalEmpty:
                    false
                SelfhostHirRangeBuildError::EndOverflow:
                    false
                SelfhostHirRangeBuildError::OutOfBounds:
                    true

fn is_canonical_empty %fn Result SelfhostHirParamRange SelfhostHirRangeBuildError bool \result:
    match result:
        Result::Ok range:
            and eq selfhost_hir_param_range_first range 0 eq selfhost_hir_param_range_count range 0
        Result::Err _e:
            false

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let checks1 checks_push checks0 check is_negative_count selfhost_hir_child_range_new_result 0 -1
    let checks2 checks_push checks1 check is_noncanonical_empty selfhost_hir_child_range_new_result 2 0
    let checks3 checks_push checks2 check is_param_end_overflow selfhost_hir_param_range_new_result 2147483647 1
    let checks4 checks_push checks3 check is_child_out_of_bounds selfhost_hir_child_range_new_bounded_result 1 2 2
    let checks5 checks_push checks4 check is_canonical_empty selfhost_hir_param_range_new_result 0 0
    let shown checks_print_report checks5
    checks_exit_code shown
```
