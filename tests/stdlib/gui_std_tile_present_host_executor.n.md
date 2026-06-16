# GUI std row tile RLE present host executor doctests

このファイルは、F5cy の std layer RGBA8888 row tile RLE present host executor boundary が、対応 target と report/action の結び付きを実行前に検査することを固定する。

source policy labels:

- std_row_tile_rle_present_host_executor_facade_ok
- std_row_tile_rle_present_host_executor_support_enum_ok
- std_row_tile_rle_present_host_executor_typed_error_ok
- std_row_tile_rle_present_host_executor_action_equality_ok
- std_row_tile_rle_present_host_executor_failed_report_preserved_ok
- std_row_tile_rle_present_host_executor_no_f5cv_no_platform_no_fallback

## host executor validation smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_executor\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"supported action\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"unsupported target\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"matching report\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"same variant mismatch\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"failed report preserved\" expected=\"5\" actual=\"5\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_command" as *
#import "std/gui/tile_present_host_execution" as *
#import "std/gui/tile_present_host_execution_report" as *
#import "std/gui/tile_present_host_executor" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_row_tile_rle_present_host_executor_facade_ok
// std_row_tile_rle_present_host_executor_support_enum_ok
// std_row_tile_rle_present_host_executor_typed_error_ok
// std_row_tile_rle_present_host_executor_action_equality_ok
// std_row_tile_rle_present_host_executor_failed_report_preserved_ok
// std_row_tile_rle_present_host_executor_no_f5cv_no_platform_no_fallback

fn sample_descriptor_for_frame %fn i32 GuiRgba8888RowTileRlePresentDescriptor \frame_id:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result frame_id
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor frame_id 0 0 0 1 0 1 3 1 12 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_run_record_offset %fn i32 GuiRgba8888RowTileRlePresentHostCommandRunRecord \offset:
    let descriptor %GuiRgba8888RowTileRlePresentDescriptor sample_descriptor_for_frame 4
    let r %u8 cast 21
    let g %u8 cast 22
    let b %u8 cast 23
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun offset 3 color
    gui_rgba8888_row_tile_rle_present_host_command_run_record descriptor run

fn sample_window_run_action %fn i32 GuiRgba8888RowTileRlePresentHostExecutionAction \offset:
    let window %WindowId unwrap_ok window_id_result 7
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_run_record_offset offset
    let payload %GuiRgba8888RowTileRlePresentHostExecutionWindowRun GuiRgba8888RowTileRlePresentHostExecutionWindowRun window record
    GuiRgba8888RowTileRlePresentHostExecutionAction::WindowRun payload

fn supported_action_code %fn void i32 \void:
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_run_record_offset 0
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction GuiRgba8888RowTileRlePresentHostExecutionAction::OffscreenRun record
    match gui_rgba8888_row_tile_rle_present_host_executor_require_supported GuiRgba8888RowTileRlePresentHostExecutorSupport::Offscreen action:
        Result::Ok _unit:
            match gui_rgba8888_row_tile_rle_present_host_executor_action_kind &action:
                GuiRgba8888RowTileRlePresentHostExecutorActionKind::OffscreenRun:
                    1
                _:
                    -2
        Result::Err _error:
            -1

fn unsupported_target_code %fn void i32 \void:
    let descriptor %GuiRgba8888RowTileRlePresentDescriptor sample_descriptor_for_frame 4
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction GuiRgba8888RowTileRlePresentHostExecutionAction::DeviceEnd descriptor
    match gui_rgba8888_row_tile_rle_present_host_executor_require_supported GuiRgba8888RowTileRlePresentHostExecutorSupport::Window action:
        Result::Ok _unit:
            -1
        Result::Err error:
            match gui_rgba8888_row_tile_rle_present_host_executor_error_kind &error:
                GuiRgba8888RowTileRlePresentHostExecutorErrorKind::UnsupportedAction:
                    match gui_rgba8888_row_tile_rle_present_host_executor_error_category_value &error:
                        Option::Some category:
                            match category:
                                GuiError::Unsupported:
                                    match gui_rgba8888_row_tile_rle_present_host_executor_error_reported &error:
                                        Option::None:
                                            2
                                        Option::Some _reported:
                                            -4
                                _:
                                    -3
                        Option::None:
                            -2
                _:
                    -5

fn matching_report_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction sample_window_run_action 0
    let report %GuiRgba8888RowTileRlePresentHostExecutionReport gui_rgba8888_row_tile_rle_present_host_execution_report action Result::Ok unit
    match gui_rgba8888_row_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888RowTileRlePresentHostExecutorSupport::Window action report:
        Result::Ok validated:
            match gui_rgba8888_row_tile_rle_present_host_execution_report_outcome &validated:
                Result::Ok _unit:
                    3
                Result::Err _error:
                    -2
        Result::Err _error:
            -1

fn same_variant_mismatch_code %fn void i32 \void:
    let expected %GuiRgba8888RowTileRlePresentHostExecutionAction sample_window_run_action 0
    let reported %GuiRgba8888RowTileRlePresentHostExecutionAction sample_window_run_action 1
    let report %GuiRgba8888RowTileRlePresentHostExecutionReport gui_rgba8888_row_tile_rle_present_host_execution_report reported Result::Ok unit
    match gui_rgba8888_row_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888RowTileRlePresentHostExecutorSupport::Window expected report:
        Result::Ok _validated:
            -1
        Result::Err error:
            match gui_rgba8888_row_tile_rle_present_host_executor_error_kind &error:
                GuiRgba8888RowTileRlePresentHostExecutorErrorKind::ReportActionMismatch:
                    match gui_rgba8888_row_tile_rle_present_host_executor_error_reported &error:
                        Option::Some reported_action:
                            if gui_rgba8888_row_tile_rle_present_host_executor_action_same &reported &reported_action:
                                then 4
                                else -3
                        Option::None:
                            -2
                _:
                    -4

fn failed_report_preserved_code %fn void i32 \void:
    let action %GuiRgba8888RowTileRlePresentHostExecutionAction sample_window_run_action 0
    let report %GuiRgba8888RowTileRlePresentHostExecutionReport gui_rgba8888_row_tile_rle_present_host_execution_report action Result::Err GuiError::BackendFailure
    match gui_rgba8888_row_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888RowTileRlePresentHostExecutorSupport::All action report:
        Result::Ok validated:
            match gui_rgba8888_row_tile_rle_present_host_execution_report_outcome &validated:
                Result::Ok _unit:
                    -1
                Result::Err error:
                    match error:
                        GuiError::BackendFailure:
                            5
                        _:
                            -3
        Result::Err _error:
            -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_executor"
        |> test::test_report_push test::assert_eq_i32 "supported action" 1 supported_action_code
        |> test::test_report_push test::assert_eq_i32 "unsupported target" 2 unsupported_target_code
        |> test::test_report_push test::assert_eq_i32 "matching report" 3 matching_report_code
        |> test::test_report_push test::assert_eq_i32 "same variant mismatch" 4 same_variant_mismatch_code
        |> test::test_report_push test::assert_eq_i32 "failed report preserved" 5 failed_report_preserved_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
