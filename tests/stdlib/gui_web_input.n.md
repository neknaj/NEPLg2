# platforms/gui/web input boundary

Web Playground backend の input host import を、raw sentinel ではなく `Result` / `Option` で扱えることを確認する。

## empty event queue returns Ok None

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"empty event queue returns Ok None\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/option" as *
#import "core/result" as *
#import "platforms/gui/web" as *
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    match gui_web_poll_event_result:
        Result::Ok event:
            if is_none event then 0 else 2
        Result::Err _:
            1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "empty event queue returns Ok None"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## web event wrapper exposes pointer keyboard text input and window variants

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"web event wrapper exposes pointer keyboard text input and window variants\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/char" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "platforms/gui/web" as *
#import "std/gui/keymap" as *
#import "std/gui/window" as *
#import "std/test" as *

fn pointer_wrapper_ok %fn &GuiWebEvent bool \web:
    let window %WindowId gui_web_event_window_id web
    let window_ok %bool eq 3 window_id_raw &window
    match gui_web_event_pointer web:
        Option::Some event:
            match pointer_event_kind &event:
                PointerEventKind::Move:
                    and window_ok eq 9 pointer_event_pointer_id &event
                _:
                    false
        Option::None:
            false

fn keyboard_wrapper_ok %fn &GuiWebEvent bool \web:
    match gui_web_event_keyboard web:
        Option::Some event:
            let key_map %FocusKeyMap focus_key_map_default
            let command %Option FocusRouteCommand keyboard_event_to_focus_route_command &key_map event
            is_some command
        Option::None:
            false

fn text_wrapper_ok %fn &GuiWebEvent bool \web:
    match gui_web_event_text_input web:
        Option::Some event:
            eq 0x3042 char_to_i32 text_input_event_value &event
        Option::None:
            false

fn window_wrapper_ok %fn &GuiWebEvent bool \web:
    match gui_web_event_window web:
        Option::Some event:
            match window_event_kind &event:
                WindowEventKind::Resized:
                    let got_size %GuiSize window_event_size &event
                    let width_ok %bool eq 640 gui_size_width &got_size
                    let height_ok %bool eq 480 gui_size_height &got_size
                    and width_ok height_ok
                _:
                    false
        Option::None:
            false

fn run_case %impure fn void i32 \void:
    let point %GuiPoint gui_point_new 0 0
    let host_window %WindowId unwrap_ok window_id_result 3
    let pointer %PointerEvent pointer_event_new PointerEventKind::Move 9 point PointerButton::None
    let pointer_web %GuiWebEvent GuiWebEvent host_window point gui_event_pointer pointer
    let pointer_ok %bool pointer_wrapper_ok &pointer_web
    let keyboard %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 9 1
    let keyboard_web %GuiWebEvent GuiWebEvent host_window point gui_event_keyboard keyboard
    let keyboard_ok %bool keyboard_wrapper_ok &keyboard_web
    let text %TextInputEvent text_input_event_new '\u{3042}'
    let text_web %GuiWebEvent GuiWebEvent host_window point gui_event_text_input text
    let text_ok %bool text_wrapper_ok &text_web
    let size %GuiSize gui_size_new 640 480
    let window %WindowEvent window_event_new WindowEventKind::Resized size
    let window_web %GuiWebEvent GuiWebEvent host_window point gui_event_window window
    let window_ok %bool window_wrapper_ok &window_web
    let no_window %Option WindowEvent gui_web_event_window &keyboard_web
    let no_window_ok %bool is_none no_window
    let primary_ok %bool and:
        and pointer_ok keyboard_ok
        and text_ok window_ok
    if and primary_ok no_window_ok then 0 else 1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "web event wrapper exposes pointer keyboard text input and window variants"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## web video memory surface produces standard present command

[目的/もくてき]:
- Web backend の正式 GUI presentation path が legacy stdout helper ではなく、標準 `GuiSurfacePresentCommand` を作ることを確認します。
- single-slot surface は tearing 防止のため `GuiError::InvalidCommand` として拒否されます。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"web video memory surface produces standard present command\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"slot count\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"frame id\" expected=\"8\" actual=\"8\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"single slot rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"descriptor width\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "platforms/gui/web" as *
#import "std/gui" as *
#import "std/test" as *

fn single_slot_rejected %fn SurfaceId bool \surface:
    match gui_web_video_memory_surface surface 16 16 64 1 0:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _surface:
            false

fn command_frame_id %fn GuiSurfacePresentCommand i32 \command:
    match command:
        GuiSurfacePresentCommand::PresentPixelFrame payload:
            let frame %FrameId gui_surface_frame_id &payload
            frame_id_raw &frame

fn main %impure fn void i32 \void:
    let surface_id_value %SurfaceId unwrap_ok surface_id_result 6
    let web_surface %GuiWebVideoMemorySurface unwrap_ok gui_web_video_memory_surface surface_id_value 16 16 64 2 3
    let descriptor %GuiPixelBufferDescriptor gui_web_video_memory_surface_descriptor &web_surface
    let frame %FrameId unwrap_ok frame_id_result 8
    let command %GuiSurfacePresentCommand gui_web_video_memory_present_command &web_surface frame dirty_region_full
    let width_ok %bool eq 16 gui_pixel_buffer_width &descriptor
    let slot_count %i32 gui_web_video_memory_surface_slot_count &web_surface
    let slot_count_check assert_eq_i32 "slot count" 2 slot_count
    let frame_id_check assert_eq_i32 "frame id" 8 command_frame_id command
    let single_slot_check assert "single slot rejected" single_slot_rejected surface_id_value
    let width_check assert "descriptor width" width_ok
    let checks0 test_report_new "web video memory surface produces standard present command"
    let checks1 test_report_push checks0 slot_count_check
    let checks2 test_report_push checks1 frame_id_check
    let checks3 test_report_push checks2 single_slot_check
    let checks test_report_push checks3 width_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```
