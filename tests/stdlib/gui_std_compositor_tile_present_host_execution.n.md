# GUI std compositor tile RLE present host execution doctests

このファイルは、F5mz の std layer compositor tile RLE present host execution action boundary の public import surface と target x record mapping を固定する。

source policy labels:

- std_compositor_tile_rle_present_host_execution_facade_ok
- std_compositor_tile_rle_present_host_execution_action_enum_ok
- std_compositor_tile_rle_present_host_execution_f5mx_request_only_ok
- std_compositor_tile_rle_present_host_execution_metadata_descriptor_preserved_ok
- std_compositor_tile_rle_present_host_execution_run_record_preserved_ok
- std_compositor_tile_rle_present_host_execution_flat_target_record_mapping_ok
- std_compositor_tile_rle_present_host_execution_no_f5my_f5mw_f5mv_no_lower_no_platform_no_fallback

## host execution mapping smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_execution\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"window begin code\" expected=\"74\" actual=\"74\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"window run code\" expected=\"702\" actual=\"702\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"window end code\" expected=\"90\" actual=\"90\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"offscreen run code\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"device end code\" expected=\"7\" actual=\"7\" message=\"\"\n"
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
#import "core/math" as *
#import "core/result" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_import" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_execution_facade_ok
// std_compositor_tile_rle_present_host_execution_action_enum_ok
// std_compositor_tile_rle_present_host_execution_f5mx_request_only_ok
// std_compositor_tile_rle_present_host_execution_metadata_descriptor_preserved_ok
// std_compositor_tile_rle_present_host_execution_run_record_preserved_ok
// std_compositor_tile_rle_present_host_execution_flat_target_record_mapping_ok
// std_compositor_tile_rle_present_host_execution_no_f5my_f5mw_f5mv_no_lower_no_platform_no_fallback

fn sample_present_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 1 0 1 2 1 8 1 1 2 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_descriptor %fn void GuiRgba8888CompositorTileRlePresentFrameDescriptor \void:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 4 20 30 4 5 6 7
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_run_record %fn void GuiRgba8888CompositorTileRlePresentHostCommandRunRecord \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let r %u8 cast 11
    let g %u8 cast 12
    let b %u8 cast 13
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun 0 2 color
    gui_rgba8888_compositor_tile_rle_present_host_command_run_record descriptor run

fn window_begin_code %fn void i32 \void:
    let window %WindowId unwrap_ok window_id_result 7
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::BeginFrame descriptor
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Window window
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_action &request
    match action:
        GuiRgba8888CompositorTileRlePresentHostExecutionAction::WindowBegin payload:
            let payload_window %WindowId gui_rgba8888_compositor_tile_rle_present_host_execution_window_begin_window &payload
            let payload_descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor gui_rgba8888_compositor_tile_rle_present_host_execution_window_begin_descriptor &payload
            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_present_frame_descriptor_metadata &payload_descriptor
            add mul window_id_raw &payload_window 10 gui_rgba8888_compositor_frame_entry_metadata_frame_id &metadata
        _:
            -1

fn window_run_code %fn void i32 \void:
    let window %WindowId unwrap_ok window_id_result 7
    let run_record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord sample_run_record
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::RunRecord run_record
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Window window
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_action &request
    match action:
        GuiRgba8888CompositorTileRlePresentHostExecutionAction::WindowRun payload:
            let payload_window %WindowId gui_rgba8888_compositor_tile_rle_present_host_execution_window_run_window &payload
            let payload_record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord gui_rgba8888_compositor_tile_rle_present_host_execution_window_run_record &payload
            let payload_run %GuiRgba8888RowTileRleRun gui_rgba8888_compositor_tile_rle_present_host_command_run_record_run &payload_record
            add mul window_id_raw &payload_window 100 gui_rgba8888_row_tile_rle_run_pixel_count &payload_run
        _:
            -1

fn window_end_code %fn void i32 \void:
    let window %WindowId unwrap_ok window_id_result 7
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::EndFrame descriptor
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Window window
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_action &request
    match action:
        GuiRgba8888CompositorTileRlePresentHostExecutionAction::WindowEnd payload:
            let payload_window %WindowId gui_rgba8888_compositor_tile_rle_present_host_execution_window_end_window &payload
            let payload_descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor gui_rgba8888_compositor_tile_rle_present_host_execution_window_end_descriptor &payload
            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_present_frame_descriptor_metadata &payload_descriptor
            add mul window_id_raw &payload_window 10 gui_rgba8888_compositor_frame_entry_metadata_width &metadata
        _:
            -1

fn offscreen_run_code %fn void i32 \void:
    let run_record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord sample_run_record
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::RunRecord run_record
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Offscreen
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_action &request
    match action:
        GuiRgba8888CompositorTileRlePresentHostExecutionAction::OffscreenRun payload:
            let payload_run %GuiRgba8888RowTileRleRun gui_rgba8888_compositor_tile_rle_present_host_command_run_record_run &payload
            gui_rgba8888_row_tile_rle_run_pixel_count &payload_run
        _:
            -1

fn device_end_code %fn void i32 \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::EndFrame descriptor
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Device
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_action &request
    match action:
        GuiRgba8888CompositorTileRlePresentHostExecutionAction::DeviceEnd payload:
            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_present_frame_descriptor_metadata &payload
            gui_rgba8888_compositor_frame_entry_metadata_max_rows_per_batch &metadata
        _:
            -1

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_execution"
        |> test::test_report_push test::assert_eq_i32 "window begin code" 74 window_begin_code
        |> test::test_report_push test::assert_eq_i32 "window run code" 702 window_run_code
        |> test::test_report_push test::assert_eq_i32 "window end code" 90 window_end_code
        |> test::test_report_push test::assert_eq_i32 "offscreen run code" 2 offscreen_run_code
        |> test::test_report_push test::assert_eq_i32 "device end code" 7 device_end_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
