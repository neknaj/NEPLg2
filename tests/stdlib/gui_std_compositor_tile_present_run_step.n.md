# GUI std compositor tile RLE present run step bridge doctests

このファイルは、F5ms の std layer compositor tile RLE present run step bridge が F5mr run cursor owner から lower F5co run cursor step へ 1 回だけ進み、command cursor / packet record / host / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_run_step_facade_ok
- std_compositor_tile_rle_present_run_step_result_error_kind_runtime_ok
- std_compositor_tile_rle_present_run_step_calls_lower_f5co_step_ok
- std_compositor_tile_rle_present_run_step_metadata_before_owner_move_ok
- std_compositor_tile_rle_present_run_step_result_and_owner_recovery_ok
- std_compositor_tile_rle_present_run_step_free_delegates_run_cursor_ok
- std_compositor_tile_rle_present_run_step_no_command_record_host_platform_fallback

## facade and wrapped result smoke

neplg2:test
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_run_step" as *
#import "std/gui/tile_present_run_cursor" as *
#import "core/math" as *

// std_compositor_tile_rle_present_run_step_facade_ok
// std_compositor_tile_rle_present_run_step_result_error_kind_runtime_ok

fn step_result_code %fn GuiRgba8888RowTileRlePresentRunCursorStepResult i32 \result:
    match result:
        GuiRgba8888RowTileRlePresentRunCursorStepResult::RunReady _:
            1
        GuiRgba8888RowTileRlePresentRunCursorStepResult::Completed:
            2

fn step_error_kind_code %fn GuiRgba8888CompositorTileRlePresentRunStepErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRlePresentRunStepErrorKind::RunCursorStepFailed lower:
            match lower:
                GuiRgba8888RowTileRlePresentRunCursorStepErrorKind::RecordIndexAdvanceOverflow:
                    3
                _:
                    4

fn main %impure fn void i32 \void:
    let result %GuiRgba8888RowTileRlePresentRunCursorStepResult GuiRgba8888RowTileRlePresentRunCursorStepResult::Completed
    let lower %GuiRgba8888RowTileRlePresentRunCursorStepErrorKind GuiRgba8888RowTileRlePresentRunCursorStepErrorKind::RecordIndexAdvanceOverflow
    let kind %GuiRgba8888CompositorTileRlePresentRunStepErrorKind GuiRgba8888CompositorTileRlePresentRunStepErrorKind::RunCursorStepFailed lower
    if and eq step_result_code result 2 eq step_error_kind_code kind 3:
        then 0
        else 1
```

## source-policy fixtures

neplg2:test[skip]
```neplg2
#indent 4

#import "std/gui/compositor_tile_present_run_step" as *

// std_compositor_tile_rle_present_run_step_calls_lower_f5co_step_ok
// std_compositor_tile_rle_present_run_step_metadata_before_owner_move_ok
// std_compositor_tile_rle_present_run_step_result_and_owner_recovery_ok
// std_compositor_tile_rle_present_run_step_free_delegates_run_cursor_ok
// std_compositor_tile_rle_present_run_step_no_command_record_host_platform_fallback
```
