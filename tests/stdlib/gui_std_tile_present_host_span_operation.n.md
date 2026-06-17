# GUI std row tile RLE present host span operation doctests

このファイルは、F5dg の std layer RGBA8888 row tile RLE present host span operation boundary が、F5cw action を target-qualified operation stream に写し、Run action だけを F5df row span cursor で分解する contract を固定する。

source policy labels:

- std_row_tile_rle_present_host_span_operation_facade_ok
- std_row_tile_rle_present_host_span_operation_operation_enum_ok
- std_row_tile_rle_present_host_span_operation_single_pending_ok
- std_row_tile_rle_present_host_span_operation_run_span_cursor_ok
- std_row_tile_rle_present_host_span_operation_f5cw_f5df_only_ok
- std_row_tile_rle_present_host_span_operation_no_raw_no_platform_no_fallback

## begin action is one-shot

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_span_operation_begin\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"window begin single code\" expected=\"721\" actual=\"721\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/math" as *
#import "core/result" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_execution" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_row_tile_rle_present_host_span_operation_facade_ok
// std_row_tile_rle_present_host_span_operation_operation_enum_ok
// std_row_tile_rle_present_host_span_operation_single_pending_ok

fn sample_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 1 0 1 2 1 8 1 1 2 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn begin_action %fn void GuiRgba8888RowTileRlePresentHostExecutionAction \void:
    let window %WindowId unwrap_ok window_id_result 7
    let descriptor %GuiRgba8888RowTileRlePresentDescriptor sample_descriptor
    let payload %GuiRgba8888RowTileRlePresentHostExecutionWindowBegin GuiRgba8888RowTileRlePresentHostExecutionWindowBegin window descriptor
    GuiRgba8888RowTileRlePresentHostExecutionAction::WindowBegin payload

fn begin_operation_code %fn &GuiRgba8888RowTileRlePresentHostSpanOperation i32 \operation:
    match *operation:
        GuiRgba8888RowTileRlePresentHostSpanOperation::WindowBegin payload:
            let window %WindowId gui_rgba8888_row_tile_rle_present_host_span_operation_window_begin_window &payload
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor gui_rgba8888_row_tile_rle_present_host_span_operation_window_begin_descriptor &payload
            let surface %SurfaceId gui_rgba8888_row_tile_rle_present_descriptor_surface &descriptor
            add mul window_id_raw &window 10 surface_id_raw &surface
        _:
            -1

fn window_begin_single_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction begin_action
    match gui_rgba8888_row_tile_rle_present_host_span_operation_start action:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match gui_rgba8888_row_tile_rle_present_host_span_operation_step cursor:
                Result::Err _error2:
                    -2
                Result::Ok first:
                    match first:
                        GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::Completed:
                            -3
                        GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::OperationReady ready:
                            let operation %GuiRgba8888RowTileRlePresentHostSpanOperation gui_rgba8888_row_tile_rle_present_host_span_operation_ready_operation &ready
                            let code %i32 begin_operation_code &operation
                            let next_cursor %GuiRgba8888RowTileRlePresentHostSpanOperationCursor gui_rgba8888_row_tile_rle_present_host_span_operation_ready_cursor &ready
                            match gui_rgba8888_row_tile_rle_present_host_span_operation_step next_cursor:
                                Result::Err _error3:
                                    -4
                                Result::Ok second:
                                    match second:
                                        GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::Completed:
                                            add mul code 10 1
                                        GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::OperationReady _ready2:
                                            -5

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_span_operation_begin"
        |> test::test_report_push test::assert_eq_i32 "window begin single code" 721 window_begin_single_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## run action emits row span operations

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_span_operation_run\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"window run first span\" expected=\"7222\" actual=\"7222\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"window run second span\" expected=\"7033\" actual=\"7033\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"window run completed\" expected=\"1\" actual=\"1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/math" as *
#import "core/result" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_command" as *
#import "std/gui/tile_present_host_execution" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/tile_present_run_span" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_row_tile_rle_present_host_span_operation_run_span_cursor_ok
// std_row_tile_rle_present_host_span_operation_f5cw_f5df_only_ok
// std_row_tile_rle_present_host_span_operation_no_raw_no_platform_no_fallback

fn sample_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 2 2 2 4 8 16 2 1 8 2 24
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_color %fn void Rgba8888 \void:
    let r %u8 cast 11
    let g %u8 cast 12
    let b %u8 cast 13
    let a %u8 cast 255
    rgba8888_new r g b a

fn window_run_action %fn void GuiRgba8888RowTileRlePresentHostExecutionAction \void:
    let window %WindowId unwrap_ok window_id_result 7
    let descriptor %GuiRgba8888RowTileRlePresentDescriptor sample_descriptor
    let color %Rgba8888 sample_color
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun 2 5 color
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord gui_rgba8888_row_tile_rle_present_host_command_run_record descriptor run
    let payload %GuiRgba8888RowTileRlePresentHostExecutionWindowRun GuiRgba8888RowTileRlePresentHostExecutionWindowRun window record
    GuiRgba8888RowTileRlePresentHostExecutionAction::WindowRun payload

fn span_code %fn &GuiRgba8888RowTileRlePresentRunRowSpan i32 \span:
    add:
        add:
            mul gui_rgba8888_row_tile_rle_present_run_row_span_x span 100
            mul gui_rgba8888_row_tile_rle_present_run_row_span_y span 10
        gui_rgba8888_row_tile_rle_present_run_row_span_width span

fn run_operation_code %fn &GuiRgba8888RowTileRlePresentHostSpanOperation i32 \operation:
    match *operation:
        GuiRgba8888RowTileRlePresentHostSpanOperation::WindowRunSpan payload:
            let window %WindowId gui_rgba8888_row_tile_rle_present_host_span_operation_window_run_span_window &payload
            let span %GuiRgba8888RowTileRlePresentRunRowSpan gui_rgba8888_row_tile_rle_present_host_span_operation_window_run_span_span &payload
            add mul window_id_raw &window 1000 span_code &span
        _:
            -1

fn first_ready_cursor %fn GuiRgba8888RowTileRlePresentHostSpanOperationCursor Result GuiRgba8888RowTileRlePresentHostSpanOperationReady i32 \cursor:
    match gui_rgba8888_row_tile_rle_present_host_span_operation_step cursor:
        Result::Err _error:
            Result::Err -2
        Result::Ok result:
            match result:
                GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::Completed:
                    Result::Err -3
                GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::OperationReady ready:
                    Result::Ok ready

fn window_run_first_span_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction window_run_action
    match gui_rgba8888_row_tile_rle_present_host_span_operation_start action:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match first_ready_cursor cursor:
                Result::Err code:
                    code
                Result::Ok ready:
                    let operation %GuiRgba8888RowTileRlePresentHostSpanOperation gui_rgba8888_row_tile_rle_present_host_span_operation_ready_operation &ready
                    run_operation_code &operation

fn window_run_second_span_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction window_run_action
    match gui_rgba8888_row_tile_rle_present_host_span_operation_start action:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match first_ready_cursor cursor:
                Result::Err code:
                    code
                Result::Ok ready:
                    let second_cursor %GuiRgba8888RowTileRlePresentHostSpanOperationCursor gui_rgba8888_row_tile_rle_present_host_span_operation_ready_cursor &ready
                    match first_ready_cursor second_cursor:
                        Result::Err code2:
                            code2
                        Result::Ok ready2:
                            let operation %GuiRgba8888RowTileRlePresentHostSpanOperation gui_rgba8888_row_tile_rle_present_host_span_operation_ready_operation &ready2
                            run_operation_code &operation

fn window_run_completed_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction window_run_action
    match gui_rgba8888_row_tile_rle_present_host_span_operation_start action:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match first_ready_cursor cursor:
                Result::Err code:
                    code
                Result::Ok ready:
                    let second_cursor %GuiRgba8888RowTileRlePresentHostSpanOperationCursor gui_rgba8888_row_tile_rle_present_host_span_operation_ready_cursor &ready
                    match first_ready_cursor second_cursor:
                        Result::Err code2:
                            code2
                        Result::Ok ready2:
                            let third_cursor %GuiRgba8888RowTileRlePresentHostSpanOperationCursor gui_rgba8888_row_tile_rle_present_host_span_operation_ready_cursor &ready2
                            match gui_rgba8888_row_tile_rle_present_host_span_operation_step third_cursor:
                                Result::Err _error3:
                                    -6
                                Result::Ok result:
                                    match result:
                                        GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::Completed:
                                            1
                                        GuiRgba8888RowTileRlePresentHostSpanOperationStepResult::OperationReady _ready3:
                                            -7

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_span_operation_run"
        |> test::test_report_push test::assert_eq_i32 "window run first span" 7222 window_run_first_span_code
        |> test::test_report_push test::assert_eq_i32 "window run second span" 7033 window_run_second_span_code
        |> test::test_report_push test::assert_eq_i32 "window run completed" 1 window_run_completed_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## run start error keeps lower kind

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_span_operation_error\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"run start wrapped error\" expected=\"77\" actual=\"77\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/result" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_command" as *
#import "std/gui/tile_present_host_execution" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/tile_present_run_span" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_row_tile_rle_present_host_span_operation_f5cw_f5df_only_ok
// std_row_tile_rle_present_host_span_operation_no_raw_no_platform_no_fallback

fn invalid_action %fn void GuiRgba8888RowTileRlePresentHostExecutionAction \void:
    let window %WindowId unwrap_ok window_id_result 7
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 2 2 2 4 8 16 2 1 8 2 24
    let descriptor %GuiRgba8888RowTileRlePresentDescriptor GuiRgba8888RowTileRlePresentDescriptor surface frame packet
    let r %u8 cast 11
    let g %u8 cast 12
    let b %u8 cast 13
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun 7 2 color
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord gui_rgba8888_row_tile_rle_present_host_command_run_record descriptor run
    let payload %GuiRgba8888RowTileRlePresentHostExecutionWindowRun GuiRgba8888RowTileRlePresentHostExecutionWindowRun window record
    GuiRgba8888RowTileRlePresentHostExecutionAction::WindowRun payload

fn run_start_wrapped_error_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction invalid_action
    match gui_rgba8888_row_tile_rle_present_host_span_operation_start action:
        Result::Ok _cursor:
            -1
        Result::Err error:
            let kind %GuiRgba8888RowTileRlePresentHostSpanOperationStartErrorKind gui_rgba8888_row_tile_rle_present_host_span_operation_start_error_kind &error
            match kind:
                GuiRgba8888RowTileRlePresentHostSpanOperationStartErrorKind::RunSpanStartFailed lower_kind:
                    match lower_kind:
                        GuiRgba8888RowTileRlePresentRunSpanStartErrorKind::RunEndOutOfBounds:
                            77
                        _:
                            -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_span_operation_error"
        |> test::test_report_push test::assert_eq_i32 "run start wrapped error" 77 run_start_wrapped_error_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
