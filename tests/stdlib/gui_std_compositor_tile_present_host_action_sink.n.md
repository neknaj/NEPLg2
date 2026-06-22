# GUI std compositor tile RLE present host action sink doctests

このファイルは、F5nf の std layer compositor tile RLE present host action sink boundary が、executor-supplied outcome を作らずに support preflight 後の typed step へ包むことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_action_sink_facade_ok
- std_compositor_tile_rle_present_host_action_sink_executor_outcome_ok
- std_compositor_tile_rle_present_host_action_sink_support_preflight_ok
- std_compositor_tile_rle_present_host_action_sink_no_manufactured_success_ok
- std_compositor_tile_rle_present_host_action_sink_no_driver_no_platform_no_fallback

## host action sink smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_action_sink\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"success outcome preserved\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"failure outcome preserved\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"unsupported target rejected\" expected=\"3\" actual=\"3\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/gui" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_host_action_sink" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_executor" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_action_sink_facade_ok
// std_compositor_tile_rle_present_host_action_sink_executor_outcome_ok
// std_compositor_tile_rle_present_host_action_sink_support_preflight_ok
// std_compositor_tile_rle_present_host_action_sink_no_manufactured_success_ok
// std_compositor_tile_rle_present_host_action_sink_no_driver_no_platform_no_fallback

fn sample_present_descriptor_for_frame %fn i32 GuiRgba8888RowTileRlePresentDescriptor \frame_id:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result frame_id
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor frame_id 0 0 0 1 0 1 3 1 12 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_descriptor %fn void GuiRgba8888CompositorTileRlePresentFrameDescriptor \void:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor_for_frame 4
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 4 20 30 0 1 1 4
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_offscreen_begin_action %fn void GuiRgba8888CompositorTileRlePresentHostExecutionAction \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    GuiRgba8888CompositorTileRlePresentHostExecutionAction::OffscreenBegin descriptor

fn sample_device_end_action %fn void GuiRgba8888CompositorTileRlePresentHostExecutionAction \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_descriptor
    GuiRgba8888CompositorTileRlePresentHostExecutionAction::DeviceEnd descriptor

fn success_outcome_preserved_code %fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_offscreen_begin_action
    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen action Result::Ok unit:
        Result::Ok step:
            let stored %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_action &step
            if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &action &stored:
                then:
                    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_outcome &step:
                        Result::Ok _unit:
                            1
                        Result::Err _error:
                            -3
                else:
                    -2
        Result::Err _error:
            -1

fn failure_outcome_preserved_code %fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_offscreen_begin_action
    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen action Result::Err GuiError::BackendFailure:
        Result::Ok step:
            let stored %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_action &step
            if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &action &stored:
                then:
                    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_outcome &step:
                        Result::Err error:
                            match error:
                                GuiError::BackendFailure:
                                    2
                                _:
                                    -4
                        Result::Ok _unit:
                            -3
                else:
                    -2
        Result::Err _error:
            -1

fn unsupported_target_rejected_code %fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_device_end_action
    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window action Result::Ok unit:
        Result::Ok _step:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_action_sink_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostActionSinkErrorKind::UnsupportedAction _lower:
                    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_error_category_value &error:
                        Option::Some category:
                            match category:
                                GuiError::Unsupported:
                                    let stored %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_sink_error_action &error
                                    if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &action &stored:
                                        then 3
                                        else -5
                                _:
                                    -4
                        Option::None:
                            -3

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_action_sink"
        |> test::test_report_push test::assert_eq_i32 "success outcome preserved" 1 success_outcome_preserved_code
        |> test::test_report_push test::assert_eq_i32 "failure outcome preserved" 2 failure_outcome_preserved_code
        |> test::test_report_push test::assert_eq_i32 "unsupported target rejected" 3 unsupported_target_rejected_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
