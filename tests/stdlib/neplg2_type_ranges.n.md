# NEPLg2 self-host type range validation

## rejects_invalid_function_type_argument_ranges

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
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn is_negative_arg_count %fn Result SelfhostFunctionTypeArgRange SelfhostFunctionTypeArgRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostFunctionTypeArgRangeBuildError::NegativeFirst:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NegativeCount:
                    true
                SelfhostFunctionTypeArgRangeBuildError::NonCanonicalEmpty:
                    false
                SelfhostFunctionTypeArgRangeBuildError::EndOverflow:
                    false
                SelfhostFunctionTypeArgRangeBuildError::OutOfBounds:
                    false

fn is_noncanonical_empty %fn Result SelfhostFunctionTypeArgRange SelfhostFunctionTypeArgRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostFunctionTypeArgRangeBuildError::NegativeFirst:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NegativeCount:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NonCanonicalEmpty:
                    true
                SelfhostFunctionTypeArgRangeBuildError::EndOverflow:
                    false
                SelfhostFunctionTypeArgRangeBuildError::OutOfBounds:
                    false

fn is_arg_end_overflow %fn Result SelfhostFunctionTypeArgRange SelfhostFunctionTypeArgRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostFunctionTypeArgRangeBuildError::NegativeFirst:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NegativeCount:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NonCanonicalEmpty:
                    false
                SelfhostFunctionTypeArgRangeBuildError::EndOverflow:
                    true
                SelfhostFunctionTypeArgRangeBuildError::OutOfBounds:
                    false

fn is_arg_out_of_bounds %fn Result SelfhostFunctionTypeArgRange SelfhostFunctionTypeArgRangeBuildError bool \result:
    match result:
        Result::Ok _range:
            false
        Result::Err e:
            match e:
                SelfhostFunctionTypeArgRangeBuildError::NegativeFirst:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NegativeCount:
                    false
                SelfhostFunctionTypeArgRangeBuildError::NonCanonicalEmpty:
                    false
                SelfhostFunctionTypeArgRangeBuildError::EndOverflow:
                    false
                SelfhostFunctionTypeArgRangeBuildError::OutOfBounds:
                    true

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::Bool:
                Result::Ok alloc1:
                    let bool_id %SelfhostTypeId selfhost_type_arena_alloc_type_id &alloc1
                    let arena1 %SelfhostTypeArena selfhost_type_arena_alloc_into_arena alloc1
                    let invalid_args %SelfhostFunctionTypeArgRange selfhost_function_type_arg_range_new_unchecked 0 -1
                    let invalid_record %SelfhostTypeRecord selfhost_type_record_function invalid_args bool_id
                    let checks1 checks_push checks0 check is_negative_arg_count selfhost_function_type_arg_range_new_result 0 -1
                    let checks2 checks_push checks1 check is_noncanonical_empty selfhost_function_type_arg_range_new_result 2 0
                    let checks3 checks_push checks2 check is_arg_end_overflow selfhost_function_type_arg_range_new_result 2147483647 1
                    let checks4 checks_push checks3 check is_arg_out_of_bounds selfhost_function_type_arg_range_new_bounded_result 1 2 2
                    let checks5 checks_push checks4 check not selfhost_type_arena_records_equal &arena1 invalid_record invalid_record
                    selfhost_type_arena_free arena1
                    let shown checks_print_report checks5
                    checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result::Err "bool type allocation failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result::Err "type arena allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
