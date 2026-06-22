# GUI platform Web compositor host executor doctests

このファイルは、F5ni の Web compositor tile RLE present host action executor backend bridge が、expected action を Web host import ABI へ渡し、host status を typed outcome として F5nh completion path へ戻すことを固定する。

source policy labels:

- platform_web_compositor_host_executor_facade_ok
- platform_web_compositor_host_executor_backend_boundary_ok
- platform_web_compositor_host_executor_host_import_status_ok
- platform_web_compositor_host_executor_borrowed_pending_action_ok
- platform_web_compositor_host_executor_reuses_f5ni_f5nh_ok
- platform_web_compositor_host_executor_fail_closed_default_stub_ok
- platform_web_compositor_host_executor_no_loop_queue_fallback

## Web compositor host executor fail-closed smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_web_compositor_host_executor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"default host import is unsupported\" expected=\"1\" actual=\"1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/gui" as *
#import "core/gui/error" as *
#import "core/result" as *
#import "platforms/gui/web/compositor_host_executor" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_import" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_web_compositor_host_executor_facade_ok
// platform_web_compositor_host_executor_backend_boundary_ok
// platform_web_compositor_host_executor_host_import_status_ok
// platform_web_compositor_host_executor_borrowed_pending_action_ok
// platform_web_compositor_host_executor_reuses_f5ni_f5nh_ok
// platform_web_compositor_host_executor_fail_closed_default_stub_ok
// platform_web_compositor_host_executor_no_loop_queue_fallback

fn sample_present_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 1 0 1 3 1 12 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_compositor_descriptor %fn void GuiRgba8888CompositorTileRlePresentFrameDescriptor \void:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 4 20 30 0 1 1 4
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_action %fn void GuiRgba8888CompositorTileRlePresentHostExecutionAction \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_compositor_descriptor
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::BeginFrame descriptor
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Offscreen
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    gui_rgba8888_compositor_tile_rle_present_host_execution_action &request

fn default_host_import_is_unsupported_code %impure fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_action
    match gui_web_compositor_host_executor_execute_action action:
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    1
                _:
                    -2
        Result::Ok _unit:
            -1

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_web_compositor_host_executor"
        |> test::test_report_push test::assert_eq_i32 "default host import is unsupported" 1 default_host_import_is_unsupported_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
