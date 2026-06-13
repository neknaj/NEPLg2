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
        Result::Err _error:
            test_report_push test_report_new "gui_runtime_interprets_effect_as_command" assert false
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
