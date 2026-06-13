# std/gui host contract

このファイルは `std/gui` が platform 実装ではなく、`alloc/gui` の `GuiEffect` を runtime が解釈する host/effect contract であることを確認します。

## gui_runtime_interprets_effect_as_command

[目的/もくてき]:
- application は host を直接呼ばず、`GuiEffect` を返します。
- runtime helper は redraw request を `GuiRuntimeCommand` data に変換します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_runtime_interprets_effect_as_command\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"12\" actual=\"12\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/gui" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let win %WindowId unwrap_ok window_id_result 9
    let host %GuiHost gui_host_new_with_window gui_capabilities_text_grid win
    let command_result %Result GuiRuntimeCommand GuiError gui_runtime_interpret_effect &host request_redraw 12
    let checks match command_result:
        Result::Ok command:
            match command:
                GuiRuntimeCommand::RequestRedraw window:
                    test_report_push test_report_new "gui_runtime_interprets_effect_as_command" assert_eq_i32 12 window_id_raw &window
                GuiRuntimeCommand::Noop:
                    test_report_push test_report_new "gui_runtime_interprets_effect_as_command" assert false
                GuiRuntimeCommand::SetTitle _title:
                    test_report_push test_report_new "gui_runtime_interprets_effect_as_command" assert false
                GuiRuntimeCommand::PresentSurface _present:
                    test_report_push test_report_new "gui_runtime_interprets_effect_as_command" assert false
        Result::Err _error:
            test_report_push test_report_new "gui_runtime_interprets_effect_as_command" assert false
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_offscreen_snapshot_requires_offscreen_present_command

[目的/もくてき]:
- offscreen screenshot / snapshot は visible window や headless への fallback ではなく、`OffscreenPixel` host と `PresentSurface` command だけから作られることを固定します。
- pixel hash は 0 や -1 を sentinel として扱わず、backend presenter が返した opaque `i32` として保持します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_offscreen_snapshot_requires_offscreen_present_command\" count=7 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"snapshot width\" expected=\"16\" actual=\"16\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"pixel hash\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"dirty full\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"headless unsupported\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"window unsupported\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"device unsupported\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"noop unsupported\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/option" as *
#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn snapshot_error_is_unsupported %fn Result GuiOffscreenSnapshot GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    true
                _:
                    false
        Result::Ok _snapshot:
            false

fn main %impure fn void i32 \void:
    let offscreen_caps %GuiCapabilities gui_capabilities_offscreen_pixel 256 256
    let window_caps %GuiCapabilities gui_capabilities_window_pixel 256 256
    let device_caps %GuiCapabilities gui_capabilities_device_pixel ColorFormat::FormatRgba8888 false
    let no_window %Option WindowId none
    let offscreen_host %GuiHost gui_host_new offscreen_caps no_window
    let window_host %GuiHost gui_host_new window_caps no_window
    let device_host %GuiHost gui_host_new device_caps no_window
    let headless_host %GuiHost gui_host_headless
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let descriptor %GuiPixelBufferDescriptor unwrap_ok gui_pixel_buffer_descriptor surface 16 8 64 ColorFormat::FormatRgba8888
    let frame %FrameId unwrap_ok frame_id_result 4
    let surface_frame %GuiSurfaceFrame gui_surface_frame frame descriptor dirty_region_full
    let present %GuiSurfacePresentCommand gui_surface_present_pixel_frame surface_frame
    let command %GuiRuntimeCommand GuiRuntimeCommand::PresentSurface present
    let noop_command %GuiRuntimeCommand GuiRuntimeCommand::Noop
    let snapshot %GuiOffscreenSnapshot unwrap_ok gui_offscreen_snapshot_from_runtime_command &offscreen_host command -1
    let headless_result %Result GuiOffscreenSnapshot GuiError gui_offscreen_snapshot_from_runtime_command &headless_host command 1
    let window_result %Result GuiOffscreenSnapshot GuiError gui_offscreen_snapshot_from_runtime_command &window_host command 1
    let device_result %Result GuiOffscreenSnapshot GuiError gui_offscreen_snapshot_from_runtime_command &device_host command 1
    let noop_result %Result GuiOffscreenSnapshot GuiError gui_offscreen_snapshot_from_runtime_command &offscreen_host noop_command 1
    let dirty %DirtyRegion gui_offscreen_snapshot_dirty &snapshot
    let width_value %i32 gui_offscreen_snapshot_width &snapshot
    let hash_value %i32 gui_offscreen_snapshot_pixel_hash &snapshot
    let width_check assert_eq_i32 "snapshot width" 16 width_value
    let hash_check assert_eq_i32 "pixel hash" -1 hash_value
    let dirty_check assert "dirty full" dirty_region_is_full dirty
    let headless_check assert "headless unsupported" snapshot_error_is_unsupported headless_result
    let window_check assert "window unsupported" snapshot_error_is_unsupported window_result
    let device_check assert "device unsupported" snapshot_error_is_unsupported device_result
    let noop_check assert "noop unsupported" snapshot_error_is_unsupported noop_result
    let checks:
        test_report_new "gui_offscreen_snapshot_requires_offscreen_present_command"
        |> test_report_push width_check
        |> test_report_push hash_check
        |> test_report_push dirty_check
        |> test_report_push headless_check
        |> test_report_push window_check
        |> test_report_push device_check
        |> test_report_push noop_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_virtual_event_script_replays_typed_events_without_sentinel

[目的/もくてき]:
- headless/offscreen test 用 event replay が raw string や `GuiEvent::None` sentinel ではなく、`Option GuiEvent` slot を用いることを確認します。
- virtual clock の負値と overflow、script overflow が typed error になることを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_virtual_event_script_replays_typed_events_without_sentinel\" count=12 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"clock now\" expected=\"125\" actual=\"125\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"clock tick\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"negative initial time rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"negative delta rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"clock overflow rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"script overflow rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"first action\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"second timer tick\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"empty poll none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"malformed empty rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"malformed one rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"cursor overflow rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/option" as *
#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn clock_error_is_invalid %fn Result GuiVirtualClock GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _clock:
            false

fn script_error_is_resource_exhausted %fn Result GuiVirtualEventScript GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::ResourceExhausted:
                    true
                _:
                    false
        Result::Ok _script:
            false

fn script_error_is_invalid %fn Result GuiVirtualEventScript GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _script:
            false

fn poll_error_is_invalid %fn Result GuiVirtualEventPoll GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _poll:
            false

fn option_action_value %fn Option GuiEvent i32 \event_option:
    match event_option:
        Option::Some event:
            match event:
                GuiEvent::Action action:
                    action_id_raw &action
                _:
                    -1
        Option::None:
            -2

fn option_timer_tick %fn Option GuiEvent i32 \event_option:
    match event_option:
        Option::Some event:
            match event:
                GuiEvent::Timer timer:
                    timer_event_tick &timer
                _:
                    -1
        Option::None:
            -2

fn main %impure fn void i32 \void:
    let clock0 %GuiVirtualClock unwrap_ok gui_virtual_clock_result 100
    let clock1 %GuiVirtualClock unwrap_ok gui_virtual_clock_advance clock0 25
    let negative_initial %Result GuiVirtualClock GuiError gui_virtual_clock_result -1
    let negative_delta %Result GuiVirtualClock GuiError gui_virtual_clock_advance clock0 -1
    let near_max %GuiVirtualClock unwrap_ok gui_virtual_clock_result 2147483647
    let clock_overflow %Result GuiVirtualClock GuiError gui_virtual_clock_advance near_max 1
    let empty_script %GuiVirtualEventScript gui_virtual_event_script_empty
    let action %ActionId action_id_new 7
    let action_event %GuiEvent gui_event_action action
    let timer %TimerEvent timer_event_new 11 3
    let timer_event %GuiEvent gui_event_timer timer
    let script1 %GuiVirtualEventScript unwrap_ok gui_virtual_event_script_push empty_script action_event
    let script2 %GuiVirtualEventScript unwrap_ok gui_virtual_event_script_push script1 timer_event
    let overflow_push %Result GuiVirtualEventScript GuiError gui_virtual_event_script_push script2 action_event
    let poll1 %GuiVirtualEventPoll unwrap_ok gui_virtual_event_script_poll script2
    let event1 %Option GuiEvent gui_virtual_event_poll_event &poll1
    let after1 %GuiVirtualEventScript gui_virtual_event_poll_script &poll1
    let poll2 %GuiVirtualEventPoll unwrap_ok gui_virtual_event_script_poll after1
    let event2 %Option GuiEvent gui_virtual_event_poll_event &poll2
    let after2 %GuiVirtualEventScript gui_virtual_event_poll_script &poll2
    let poll3 %GuiVirtualEventPoll unwrap_ok gui_virtual_event_script_poll after2
    let event3 %Option GuiEvent gui_virtual_event_poll_event &poll3
    let malformed_empty %GuiVirtualEventScript GuiVirtualEventScript some action_event none 0 0
    let malformed_one %GuiVirtualEventScript GuiVirtualEventScript some action_event some timer_event 1 0
    let cursor_max %GuiVirtualEventScript GuiVirtualEventScript some action_event none 1 2147483647
    let malformed_empty_poll %Result GuiVirtualEventPoll GuiError gui_virtual_event_script_poll malformed_empty
    let malformed_one_push %Result GuiVirtualEventScript GuiError gui_virtual_event_script_push malformed_one action_event
    let cursor_overflow_poll %Result GuiVirtualEventPoll GuiError gui_virtual_event_script_poll cursor_max
    let clock_now %i32 gui_virtual_clock_now_ms &clock1
    let clock_tick %i32 gui_virtual_clock_tick &clock1
    let action_value %i32 option_action_value event1
    let timer_tick %i32 option_timer_tick event2
    let clock_now_check assert_eq_i32 "clock now" 125 clock_now
    let clock_tick_check assert_eq_i32 "clock tick" 1 clock_tick
    let negative_initial_check assert "negative initial time rejected" clock_error_is_invalid negative_initial
    let negative_delta_check assert "negative delta rejected" clock_error_is_invalid negative_delta
    let clock_overflow_check assert "clock overflow rejected" clock_error_is_invalid clock_overflow
    let script_overflow_check assert "script overflow rejected" script_error_is_resource_exhausted overflow_push
    let action_check assert_eq_i32 "first action" 7 action_value
    let timer_check assert_eq_i32 "second timer tick" 3 timer_tick
    let empty_poll_check assert "empty poll none" is_none event3
    let malformed_empty_check assert "malformed empty rejected" poll_error_is_invalid malformed_empty_poll
    let malformed_one_check assert "malformed one rejected" script_error_is_invalid malformed_one_push
    let cursor_overflow_check assert "cursor overflow rejected" poll_error_is_invalid cursor_overflow_poll
    let checks:
        test_report_new "gui_virtual_event_script_replays_typed_events_without_sentinel"
        |> test_report_push clock_now_check
        |> test_report_push clock_tick_check
        |> test_report_push negative_initial_check
        |> test_report_push negative_delta_check
        |> test_report_push clock_overflow_check
        |> test_report_push script_overflow_check
        |> test_report_push action_check
        |> test_report_push timer_check
        |> test_report_push empty_poll_check
        |> test_report_push malformed_empty_check
        |> test_report_push malformed_one_check
        |> test_report_push cursor_overflow_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_std_contract_values_have_no_platform_handle

[目的/もくてき]:
- window、timer、IME、text measure、accessibility が raw platform handle ではなく typed value で表されることを固定します。
- text metrics と accessibility snapshot は host update の data contract として扱います。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_contract_values_have_no_platform_handle\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"1000\" actual=\"1000\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"80\" actual=\"80\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/gui" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let win %WindowId unwrap_ok window_id_result 4
    let timer %TimerRequest timer_request win timer_id 2 1000 true
    let measurer %HostTextMeasurer host_text_measurer_fixed gui_capabilities_text_grid 8 16 12
    let run_id %TextRunId text_run_id_new 1
    let font_id %FontId font_id_new 1
    let request %TextMeasureRequest text_measure_request_new run_id font_id 80 10
    let metrics %TextMeasureResult unwrap_ok measure_text &measurer request
    let ime %ImeStateRequest ime_state_request win ImeState::Enabled
    let root %AccessibilityNodeSnapshot accessibility_node_snapshot accessibility_node_id 1 AccessibilityRole::Button "Run" true
    let check0 assert_eq_i32 1000 timer_request_interval_ms &timer
    let check1 assert_eq_i32 80 text_measure_result_width &metrics
    let check2 assert accessibility_node_is_focused &root
    let check3 match ime_state_request_state &ime:
        ImeState::Enabled:
            assert true
        ImeState::Disabled:
            assert false
    let checks0 test_report_new "gui_std_contract_values_have_no_platform_handle"
    let checks1 test_report_push checks0 check0
    let checks2 test_report_push checks1 check1
    let checks3 test_report_push checks2 check2
    let checks test_report_push checks3 check3
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_runtime_interprets_present_surface_effect

[目的/もくてき]:
- app-facing `PresentSurfaceEffect` を runtime が checked `GuiSurfacePresentCommand` に変換することを確認します。
- `TextGrid` / `Headless` は pixel frame の代替先ではなく、`GuiError::Unsupported` を返すことを固定します。
- 不正 id と不正 geometry は typed error として分岐します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_runtime_interprets_present_surface_effect\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"frame\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"width\" expected=\"16\" actual=\"16\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"text grid unsupported\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"invalid id\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"invalid geometry\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn is_unsupported %fn Result GuiRuntimeCommand GuiError bool \result:
    match result:
        Result::Ok _command:
            false
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    true
                _:
                    false

fn is_invalid_command %fn Result GuiRuntimeCommand GuiError bool \result:
    match result:
        Result::Ok _command:
            false
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false

fn is_invalid_geometry %fn Result GuiRuntimeCommand GuiError bool \result:
    match result:
        Result::Ok _command:
            false
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    true
                _:
                    false

fn main %impure fn void i32 \void:
    let win %WindowId unwrap_ok window_id_result 9
    let window_host %GuiHost gui_host_new_with_window gui_capabilities_window_pixel 1024 1024 win
    let text_host %GuiHost gui_host_new_with_window gui_capabilities_text_grid win
    let good_effect %GuiEffect present_surface 2 4 16 8 64 ColorFormat::FormatRgba8888 dirty_region_full
    let good_result %Result GuiRuntimeCommand GuiError gui_runtime_interpret_effect &window_host good_effect
    let checks0 match good_result:
        Result::Ok command:
            match command:
                GuiRuntimeCommand::PresentSurface present:
                    match present:
                        GuiSurfacePresentCommand::PresentPixelFrame surface_frame:
                            let frame %FrameId gui_surface_frame_id &surface_frame
                            let descriptor %GuiPixelBufferDescriptor gui_surface_frame_descriptor &surface_frame
                            let frame_raw %i32 frame_id_raw &frame
                            let width %i32 gui_pixel_buffer_width &descriptor
                            let frame_check assert_eq_i32 "frame" 4 frame_raw
                            let width_check assert_eq_i32 "width" 16 width
                            let report0 test_report_new "gui_runtime_interprets_present_surface_effect"
                            let report1 test_report_push report0 frame_check
                            test_report_push report1 width_check
                _:
                    test_report_push test_report_new "gui_runtime_interprets_present_surface_effect" assert false
        Result::Err _error:
            test_report_push test_report_new "gui_runtime_interprets_present_surface_effect" assert false
    let unsupported_effect %GuiEffect present_surface -1 -1 0 0 1 ColorFormat::FormatRgb888 dirty_region_full
    let unsupported_result %Result GuiRuntimeCommand GuiError gui_runtime_interpret_effect &text_host unsupported_effect
    let invalid_id_effect %GuiEffect present_surface 0 4 16 8 64 ColorFormat::FormatRgba8888 dirty_region_full
    let invalid_id_result %Result GuiRuntimeCommand GuiError gui_runtime_interpret_effect &window_host invalid_id_effect
    let invalid_geometry_effect %GuiEffect present_surface 2 4 0 8 64 ColorFormat::FormatRgba8888 dirty_region_full
    let invalid_geometry_result %Result GuiRuntimeCommand GuiError gui_runtime_interpret_effect &window_host invalid_geometry_effect
    let unsupported_check assert "text grid unsupported" is_unsupported unsupported_result
    let invalid_id_check assert "invalid id" is_invalid_command invalid_id_result
    let invalid_geometry_check assert "invalid geometry" is_invalid_geometry invalid_geometry_result
    let checks1 test_report_push checks0 unsupported_check
    let checks2 test_report_push checks1 invalid_id_check
    let checks test_report_push checks2 invalid_geometry_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_runtime_interprets_update_effect_batch

[目的/もくてき]:
- `Update.effects` の 2 個目以降を runtime が落とさず、command batch として保持することを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_runtime_interprets_update_effect_batch\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"2\" actual=\"2\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let win %WindowId unwrap_ok window_id_result 9
    let host %GuiHost gui_host_new_with_window gui_capabilities_text_grid win
    let batch0 %GuiEffectBatch gui_effect_batch_empty
    let batch1 %GuiEffectBatch unwrap_ok gui_effect_batch_push batch0 request_redraw 1
    let batch2 %GuiEffectBatch unwrap_ok gui_effect_batch_push batch1 request_redraw 2
    let update %Update i32 update_result_batch 0 batch2
    let commands %GuiRuntimeCommandBatch unwrap_ok gui_runtime_interpret_update &host update
    let count_check assert_eq_i32 2 gui_runtime_command_batch_count &commands
    let second_check match gui_runtime_command_batch_second &commands:
        Option::Some command:
            match command:
                GuiRuntimeCommand::RequestRedraw window:
                    assert_eq_i32 2 window_id_raw &window
                _:
                    assert false
        Option::None:
            assert false
    let checks1 test_report_push test_report_new "gui_runtime_interprets_update_effect_batch" count_check
    let checks test_report_push checks1 second_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_runtime_rejects_unsupported_capability

[目的/もくてき]:
- surface を持たない headless host に対する redraw request が silent no-op ではなく `GuiError::Unsupported` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_runtime_rejects_unsupported_capability\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn run_case %fn void i32 \void:
    let host %GuiHost gui_host_headless
    match gui_runtime_interpret_effect &host request_redraw 1:
        Result::Ok _command:
            1
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    0
                _:
                    2

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "gui_runtime_rejects_unsupported_capability"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## gui_opaque_ids_require_checked_constructors

[目的/もくてき]:
- window / surface / frame id は raw `i32` だけで作らず、checked constructor で 0 以下を拒否します。
- headless host は `WindowId 0` ではなく `Option::None` で既定 window 不在を表します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_opaque_ids_require_checked_constructors\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/option" as *
#import "core/math" as *
#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn is_invalid_window_id %fn i32 bool \raw:
    match window_id_result raw:
        Result::Ok _:
            false
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false

fn is_invalid_surface_id %fn i32 bool \raw:
    match surface_id_result raw:
        Result::Ok _:
            false
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false

fn frame_id_roundtrip_ok %fn i32 bool \raw:
    match frame_id_result raw:
        Result::Ok id:
            eq raw frame_id_raw &id
        Result::Err _:
            false

fn main %impure fn void i32 \void:
    let headless %GuiHost gui_host_headless
    let default_is_none %bool is_none gui_host_default_window &headless
    let checks0 test_report_new "gui_opaque_ids_require_checked_constructors"
    let checks1 test_report_push checks0 assert is_invalid_window_id 0
    let checks2 test_report_push checks1 assert is_invalid_surface_id -1
    let checks3 test_report_push checks2 assert frame_id_roundtrip_ok 7
    let checks test_report_push checks3 assert default_is_none
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_window_id_raw_constructor_is_not_public_contract

neplg2:test[compile_fail]
diag_code: type.stack.extra_values
```neplg2
#entry main
#indent 4
#target std

#import "std/gui" as *

fn main %fn void i32 \void:
    let _id %WindowId WindowId 0
    0
```

## gui_error_display_keeps_typed_error

[目的/もくてき]:
- unsupported を silent no-op にせず `GuiError` value として扱い、display helper は表示 label だけを担当します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_error_display_keeps_typed_error\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"unsupported\" actual=\"unsupported\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/gui" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let check0 assert gui_error_is_unsupported GuiError::Unsupported
    let check1 assert_str_eq "unsupported" gui_error_label GuiError::Unsupported
    let checks0 test_report_new "gui_error_display_keeps_typed_error"
    let checks1 test_report_push checks0 check0
    let checks test_report_push checks1 check1
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_surface_present_command_uses_typed_pixel_frame

[目的/もくてき]:
- `std/gui` の surface contract が Web 専用 stdout transport ではなく、typed pixel frame present command を使うことを確認します。
- 不正な stride と未対応 color format が silent fallback ではなく `GuiError` になることを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_surface_present_command_uses_typed_pixel_frame\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"frame id\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"width\" expected=\"640\" actual=\"640\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"short stride rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"unaligned stride rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"unsupported format rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn short_stride_rejected %fn SurfaceId bool \surface:
    match gui_pixel_buffer_descriptor surface 2 2 4 ColorFormat::FormatRgba8888:
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    true
                _:
                    false
        Result::Ok _descriptor:
            false

fn unaligned_stride_rejected %fn SurfaceId bool \surface:
    match gui_pixel_buffer_descriptor surface 2 2 9 ColorFormat::FormatRgba8888:
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    true
                _:
                    false
        Result::Ok _descriptor:
            false

fn unsupported_format_rejected %fn SurfaceId bool \surface:
    match gui_pixel_buffer_descriptor surface 2 2 8 ColorFormat::FormatRgb888:
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    true
                _:
                    false
        Result::Ok _descriptor:
            false

fn main %impure fn void i32 \void:
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let descriptor %GuiPixelBufferDescriptor unwrap_ok gui_pixel_buffer_descriptor surface 640 480 2560 ColorFormat::FormatRgba8888
    let frame %FrameId unwrap_ok frame_id_result 4
    let surface_frame %GuiSurfaceFrame gui_surface_frame frame descriptor dirty_region_full
    let command %GuiSurfacePresentCommand gui_surface_present_pixel_frame surface_frame
    let frame_check match command:
        GuiSurfacePresentCommand::PresentPixelFrame payload:
            let payload_frame %FrameId gui_surface_frame_id &payload
            assert_eq_i32 "frame id" 4 frame_id_raw &payload_frame
    let descriptor_check assert_eq_i32 "width" 640 gui_pixel_buffer_width &descriptor
    let short_stride_check assert "short stride rejected" short_stride_rejected surface
    let unaligned_stride_check assert "unaligned stride rejected" unaligned_stride_rejected surface
    let format_check assert "unsupported format rejected" unsupported_format_rejected surface
    let checks:
        test_report_new "gui_surface_present_command_uses_typed_pixel_frame"
        |> test_report_push frame_check
        |> test_report_push descriptor_check
        |> test_report_push short_stride_check
        |> test_report_push unaligned_stride_check
        |> test_report_push format_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```
