# GUI std compositor tile RLE present command cursor doctests

このファイルは、F5mt の std layer compositor tile RLE present command cursor boundary が F5mr/F5ms を authority として BeginFrame / Run / EndFrame を発行し、lower command cursor / host-command / packet record / host / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_command_cursor_facade_ok
- std_compositor_tile_rle_present_command_cursor_command_stream_ok
- std_compositor_tile_rle_present_command_cursor_owner_boundary_ok
- std_compositor_tile_rle_present_command_cursor_owner_recovery_ok
- std_compositor_tile_rle_present_command_cursor_one_output_step_ok
- std_compositor_tile_rle_present_command_cursor_uses_f5mr_f5ms_ok
- std_compositor_tile_rle_present_command_cursor_no_lower_command_host_record_platform_fallback

## facade and wrapped error smoke

neplg2:test
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_command_cursor" as *
#import "std/gui/compositor_tile_present_run_step" as *
#import "std/gui/tile_present_run_cursor" as *
#import "core/math" as *

// std_compositor_tile_rle_present_command_cursor_facade_ok
// std_compositor_tile_rle_present_command_cursor_command_stream_ok

fn phase_code %fn GuiRgba8888CompositorTileRlePresentCommandCursorPhase i32 \phase:
    match phase:
        GuiRgba8888CompositorTileRlePresentCommandCursorPhase::BeginPending:
            1
        GuiRgba8888CompositorTileRlePresentCommandCursorPhase::RunPending:
            2
        GuiRgba8888CompositorTileRlePresentCommandCursorPhase::Completed:
            3

fn step_error_kind_code %fn GuiRgba8888CompositorTileRlePresentCommandCursorStepErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRlePresentCommandCursorStepErrorKind::RunStepFailed lower:
            match lower:
                GuiRgba8888CompositorTileRlePresentRunStepErrorKind::RunCursorStepFailed inner:
                    match inner:
                        GuiRgba8888RowTileRlePresentRunCursorStepErrorKind::RecordIndexAdvanceOverflow:
                            7
                        _:
                            8

fn main %impure fn void i32 \void:
    let phase %GuiRgba8888CompositorTileRlePresentCommandCursorPhase GuiRgba8888CompositorTileRlePresentCommandCursorPhase::BeginPending
    let inner %GuiRgba8888RowTileRlePresentRunCursorStepErrorKind GuiRgba8888RowTileRlePresentRunCursorStepErrorKind::RecordIndexAdvanceOverflow
    let lower %GuiRgba8888CompositorTileRlePresentRunStepErrorKind GuiRgba8888CompositorTileRlePresentRunStepErrorKind::RunCursorStepFailed inner
    let kind %GuiRgba8888CompositorTileRlePresentCommandCursorStepErrorKind GuiRgba8888CompositorTileRlePresentCommandCursorStepErrorKind::RunStepFailed lower
    if and eq phase_code phase 1 eq step_error_kind_code kind 7:
        then 0
        else 1
```

## source-policy fixtures

neplg2:test[skip]
```neplg2
#indent 4

#import "std/gui/compositor_tile_present_command_cursor" as *

// std_compositor_tile_rle_present_command_cursor_owner_boundary_ok
// std_compositor_tile_rle_present_command_cursor_owner_recovery_ok
// std_compositor_tile_rle_present_command_cursor_one_output_step_ok
// std_compositor_tile_rle_present_command_cursor_uses_f5mr_f5ms_ok
// std_compositor_tile_rle_present_command_cursor_no_lower_command_host_record_platform_fallback
```
