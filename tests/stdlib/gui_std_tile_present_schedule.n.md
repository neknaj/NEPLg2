# GUI std row tile RLE present schedule doctests

このファイルは、F5ct の std layer RGBA8888 row tile RLE present scheduling boundary の public import surface と policy validation を固定する。

source policy labels:

- std_row_tile_rle_present_schedule_facade_ok
- std_row_tile_rle_present_schedule_policy_result_ok
- std_row_tile_rle_present_schedule_state_wraps_f5cs_ok
- std_row_tile_rle_present_schedule_yield_exact_budget_ok
- std_row_tile_rle_present_schedule_over_budget_error_ok
- std_row_tile_rle_present_schedule_resume_slice_ok
- std_row_tile_rle_present_schedule_no_f5cr_no_raw_no_platform_no_fallback

## policy validation smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_schedule\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"schedule policy accepts positive budgets\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"schedule policy rejects zero command budget\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "std/gui/tile_present_schedule" as *
#import "std/test" as test

// std_row_tile_rle_present_schedule_facade_ok
// std_row_tile_rle_present_schedule_policy_result_ok
// std_row_tile_rle_present_schedule_state_wraps_f5cs_ok
// std_row_tile_rle_present_schedule_yield_exact_budget_ok
// std_row_tile_rle_present_schedule_over_budget_error_ok
// std_row_tile_rle_present_schedule_resume_slice_ok
// std_row_tile_rle_present_schedule_no_f5cr_no_raw_no_platform_no_fallback

fn policy_ok %fn void i32 \void:
    match gui_rgba8888_row_tile_rle_present_schedule_policy 3 256:
        Result::Err _kind:
            1
        Result::Ok policy:
            let command_budget %i32 gui_rgba8888_row_tile_rle_present_schedule_policy_max_commands_per_slice &policy
            if ne command_budget 3:
                then 2
                else:
                    let pixel_budget %i32 gui_rgba8888_row_tile_rle_present_schedule_policy_max_pixels_per_slice &policy
                    if eq pixel_budget 256:
                        then 0
                        else 3

fn invalid_command_budget_ok %fn void i32 \void:
    match gui_rgba8888_row_tile_rle_present_schedule_policy 0 256:
        Result::Err kind:
            match kind:
                GuiRgba8888RowTileRlePresentSchedulePolicyErrorKind::MaxCommandsPerSliceInvalid:
                    0
                _:
                    2
        Result::Ok _policy:
            1

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_schedule"
        |> test::test_report_push test::assert_eq_i32 "schedule policy accepts positive budgets" 0 policy_ok
        |> test::test_report_push test::assert_eq_i32 "schedule policy rejects zero command budget" 0 invalid_command_budget_ok
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
