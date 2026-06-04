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
#import "platforms/gui/web" as *
#import "std/gui/keymap" as *
#import "std/test" as *

fn pointer_wrapper_ok %fn &GuiWebEvent bool \web:
    match gui_web_event_pointer web:
        Option::Some event:
            match pointer_event_kind &event:
                PointerEventKind::Move:
                    eq 9 pointer_event_pointer_id &event
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
    let pointer %PointerEvent pointer_event_new PointerEventKind::Move 9 point PointerButton::None
    let pointer_web %GuiWebEvent GuiWebEvent 3 point gui_event_pointer pointer
    let pointer_ok %bool pointer_wrapper_ok &pointer_web
    let keyboard %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 9 1
    let keyboard_web %GuiWebEvent GuiWebEvent 3 point gui_event_keyboard keyboard
    let keyboard_ok %bool keyboard_wrapper_ok &keyboard_web
    let text %TextInputEvent text_input_event_new '\u{3042}'
    let text_web %GuiWebEvent GuiWebEvent 3 point gui_event_text_input text
    let text_ok %bool text_wrapper_ok &text_web
    let size %GuiSize gui_size_new 640 480
    let window %WindowEvent window_event_new WindowEventKind::Resized size
    let window_web %GuiWebEvent GuiWebEvent 3 point gui_event_window window
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
