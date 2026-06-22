# GUI std compositor tile RLE present host executor doctests

このファイルは、F5nb の std layer compositor tile RLE present host executor validation boundary が、対応 target と metadata 付き report/action の結び付きを実行前に検査することを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_executor_facade_ok
- std_compositor_tile_rle_present_host_executor_support_enum_ok
- std_compositor_tile_rle_present_host_executor_typed_error_ok
- std_compositor_tile_rle_present_host_executor_action_equality_ok
- std_compositor_tile_rle_present_host_executor_metadata_equality_ok
- std_compositor_tile_rle_present_host_executor_failed_report_preserved_ok
- std_compositor_tile_rle_present_host_executor_no_f5my_f5mw_f5mv_no_lower_host_no_platform_no_fallback

## host executor validation smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_executor\" count=6 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"supported action\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"unsupported target\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"matching report\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"run mismatch\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"metadata mismatch\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"failed report preserved\" expected=\"6\" actual=\"6\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_execution_report" as *
#import "std/gui/compositor_tile_present_host_executor" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_executor_facade_ok
// std_compositor_tile_rle_present_host_executor_support_enum_ok
// std_compositor_tile_rle_present_host_executor_typed_error_ok
// std_compositor_tile_rle_present_host_executor_action_equality_ok
// std_compositor_tile_rle_present_host_executor_metadata_equality_ok
// std_compositor_tile_rle_present_host_executor_failed_report_preserved_ok
// std_compositor_tile_rle_present_host_executor_no_f5my_f5mw_f5mv_no_lower_host_no_platform_no_fallback

fn sample_present_descriptor_for_frame %fn i32 GuiRgba8888RowTileRlePresentDescriptor \frame_id:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result frame_id
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor frame_id 0 0 0 1 0 1 3 1 12 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_descriptor_for_frame_width %fn i32 fn i32 GuiRgba8888CompositorTileRlePresentFrameDescriptor \frame_id\width:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor_for_frame frame_id
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata frame_id width 30 4 5 6 7
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_run_record_offset_width %fn i32 fn i32 GuiRgba8888CompositorTileRlePresentHostCommandRunRecord \offset\width:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor_for_frame_width 4 width
    let r %u8 cast 21
    let g %u8 cast 22
    let b %u8 cast 23
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun offset 3 color
    gui_rgba8888_compositor_tile_rle_present_host_command_run_record descriptor run

fn sample_window_run_action %fn i32 fn i32 GuiRgba8888CompositorTileRlePresentHostExecutionAction \offset\width:
    let window %WindowId unwrap_ok window_id_result 7
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord sample_run_record_offset_width offset width
    let payload %GuiRgba8888CompositorTileRlePresentHostExecutionWindowRun GuiRgba8888CompositorTileRlePresentHostExecutionWindowRun window record
    GuiRgba8888CompositorTileRlePresentHostExecutionAction::WindowRun payload

fn supported_action_code %fn void i32 \void:
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord sample_run_record_offset_width 0 20
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction GuiRgba8888CompositorTileRlePresentHostExecutionAction::OffscreenRun record
    match gui_rgba8888_compositor_tile_rle_present_host_executor_require_supported GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen action:
        Result::Ok _unit:
            match gui_rgba8888_compositor_tile_rle_present_host_executor_action_kind &action:
                GuiRgba8888CompositorTileRlePresentHostExecutorActionKind::OffscreenRun:
                    1
                _:
                    -2
        Result::Err _error:
            -1

fn unsupported_target_code %fn void i32 \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor_for_frame_width 4 20
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction GuiRgba8888CompositorTileRlePresentHostExecutionAction::DeviceEnd descriptor
    match gui_rgba8888_compositor_tile_rle_present_host_executor_require_supported GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window action:
        Result::Ok _unit:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_executor_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostExecutorErrorKind::UnsupportedAction:
                    match gui_rgba8888_compositor_tile_rle_present_host_executor_error_category_value &error:
                        Option::Some category:
                            match category:
                                GuiError::Unsupported:
                                    match gui_rgba8888_compositor_tile_rle_present_host_executor_error_reported &error:
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
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action 0 20
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report action Result::Ok unit
    match gui_rgba8888_compositor_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window action report:
        Result::Ok validated:
            match gui_rgba8888_compositor_tile_rle_present_host_execution_report_outcome &validated:
                Result::Ok _unit:
                    3
                Result::Err _error:
                    -2
        Result::Err _error:
            -1

fn run_mismatch_code %fn void i32 \void:
    let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action 0 20
    let reported %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action 1 20
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report reported Result::Ok unit
    match gui_rgba8888_compositor_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window expected report:
        Result::Ok _validated:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_executor_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostExecutorErrorKind::ReportActionMismatch:
                    match gui_rgba8888_compositor_tile_rle_present_host_executor_error_reported &error:
                        Option::Some reported_action:
                            if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &reported &reported_action:
                                then 4
                                else -3
                        Option::None:
                            -2
                _:
                    -4

fn metadata_mismatch_code %fn void i32 \void:
    let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action 0 20
    let reported %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action 0 21
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report reported Result::Ok unit
    match gui_rgba8888_compositor_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window expected report:
        Result::Ok _validated:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_executor_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostExecutorErrorKind::ReportActionMismatch:
                    5
                _:
                    -2

fn failed_report_preserved_code %fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action 0 20
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report action Result::Err GuiError::BackendFailure
    match gui_rgba8888_compositor_tile_rle_present_host_executor_validate_report_for_action GuiRgba8888CompositorTileRlePresentHostExecutorSupport::All action report:
        Result::Ok validated:
            match gui_rgba8888_compositor_tile_rle_present_host_execution_report_outcome &validated:
                Result::Ok _unit:
                    -1
                Result::Err error:
                    match error:
                        GuiError::BackendFailure:
                            6
                        _:
                            -3
        Result::Err _error:
            -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_executor"
        |> test::test_report_push test::assert_eq_i32 "supported action" 1 supported_action_code
        |> test::test_report_push test::assert_eq_i32 "unsupported target" 2 unsupported_target_code
        |> test::test_report_push test::assert_eq_i32 "matching report" 3 matching_report_code
        |> test::test_report_push test::assert_eq_i32 "run mismatch" 4 run_mismatch_code
        |> test::test_report_push test::assert_eq_i32 "metadata mismatch" 5 metadata_mismatch_code
        |> test::test_report_push test::assert_eq_i32 "failed report preserved" 6 failed_report_preserved_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
