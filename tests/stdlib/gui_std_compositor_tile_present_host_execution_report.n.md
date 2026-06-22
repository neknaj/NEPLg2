# GUI std compositor tile RLE present host execution report doctests

このファイルは、F5na の std layer compositor tile RLE present host execution report boundary が metadata 付き action context と executor outcome を失わずに保持することを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_execution_report_facade_ok
- std_compositor_tile_rle_present_host_execution_report_kind_enum_ok
- std_compositor_tile_rle_present_host_execution_report_f5mz_action_only_ok
- std_compositor_tile_rle_present_host_execution_report_f5mx_request_bridge_ok
- std_compositor_tile_rle_present_host_execution_report_metadata_action_preserved_ok
- std_compositor_tile_rle_present_host_execution_report_outcome_roundtrip_ok
- std_compositor_tile_rle_present_host_execution_report_no_f5my_f5mw_f5mv_no_lower_no_platform_no_fallback

## host execution report smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_execution_report\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"success report\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"failure report\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"request action metadata report\" expected=\"707\" actual=\"707\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"outcome roundtrip\" expected=\"4\" actual=\"4\" message=\"\"\n"
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
#import "core/result" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_execution_report" as *
#import "std/gui/compositor_tile_present_host_import" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_execution_report_facade_ok
// std_compositor_tile_rle_present_host_execution_report_kind_enum_ok
// std_compositor_tile_rle_present_host_execution_report_f5mz_action_only_ok
// std_compositor_tile_rle_present_host_execution_report_f5mx_request_bridge_ok
// std_compositor_tile_rle_present_host_execution_report_metadata_action_preserved_ok
// std_compositor_tile_rle_present_host_execution_report_outcome_roundtrip_ok
// std_compositor_tile_rle_present_host_execution_report_no_f5my_f5mw_f5mv_no_lower_no_platform_no_fallback

fn sample_present_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 1 0 1 3 1 8 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_descriptor %fn void GuiRgba8888CompositorTileRlePresentFrameDescriptor \void:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 4 20 30 4 5 6 7
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_run_record %fn void GuiRgba8888CompositorTileRlePresentHostCommandRunRecord \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let r %u8 cast 21
    let g %u8 cast 22
    let b %u8 cast 23
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun 0 3 color
    gui_rgba8888_compositor_tile_rle_present_host_command_run_record descriptor run

fn success_report_code %fn void i32 \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction GuiRgba8888CompositorTileRlePresentHostExecutionAction::OffscreenEnd descriptor
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report action Result::Ok unit
    match gui_rgba8888_compositor_tile_rle_present_host_execution_report_kind &report:
        GuiRgba8888CompositorTileRlePresentHostExecutionReportKind::Succeeded:
            match gui_rgba8888_compositor_tile_rle_present_host_execution_report_outcome &report:
                Result::Ok _unit:
                    1
                Result::Err _error:
                    -2
        GuiRgba8888CompositorTileRlePresentHostExecutionReportKind::Failed _error:
            -1

fn failure_report_code %fn void i32 \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction GuiRgba8888CompositorTileRlePresentHostExecutionAction::DeviceEnd descriptor
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report action Result::Err GuiError::Unsupported
    match gui_rgba8888_compositor_tile_rle_present_host_execution_report_error &report:
        Result::Ok error:
            match error:
                GuiError::Unsupported:
                    2
                _:
                    -2
        Result::Err _error:
            -1

fn request_action_metadata_report_code %fn void i32 \void:
    let window %WindowId unwrap_ok window_id_result 7
    let run_record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord sample_run_record
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::RunRecord run_record
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Window window
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report_for_request &request Result::Ok unit
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_report_action &report
    match action:
        GuiRgba8888CompositorTileRlePresentHostExecutionAction::WindowRun payload:
            let payload_window %WindowId gui_rgba8888_compositor_tile_rle_present_host_execution_window_run_window &payload
            let payload_record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord gui_rgba8888_compositor_tile_rle_present_host_execution_window_run_record &payload
            let payload_descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor gui_rgba8888_compositor_tile_rle_present_host_command_run_record_descriptor &payload_record
            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_present_frame_descriptor_metadata &payload_descriptor
            let payload_run %GuiRgba8888RowTileRleRun gui_rgba8888_compositor_tile_rle_present_host_command_run_record_run &payload_record
            add add mul window_id_raw &payload_window 100 gui_rgba8888_row_tile_rle_run_pixel_count &payload_run gui_rgba8888_compositor_frame_entry_metadata_frame_id &metadata
        _:
            -1

fn outcome_roundtrip_code %fn void i32 \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction GuiRgba8888CompositorTileRlePresentHostExecutionAction::DeviceBegin descriptor
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport gui_rgba8888_compositor_tile_rle_present_host_execution_report action Result::Err GuiError::InvalidCommand
    match gui_rgba8888_compositor_tile_rle_present_host_execution_report_outcome &report:
        Result::Ok _unit:
            -1
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    4
                _:
                    -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_execution_report"
        |> test::test_report_push test::assert_eq_i32 "success report" 1 success_report_code
        |> test::test_report_push test::assert_eq_i32 "failure report" 2 failure_report_code
        |> test::test_report_push test::assert_eq_i32 "request action metadata report" 707 request_action_metadata_report_code
        |> test::test_report_push test::assert_eq_i32 "outcome roundtrip" 4 outcome_roundtrip_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
