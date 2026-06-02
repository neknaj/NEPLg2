# platforms/gui/web input boundary

Web Playground backend の input host import を、raw sentinel ではなく `Result` / `Option` で扱えることを確認する。

## empty event queue returns Ok None

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/option" as *
#import "core/result" as *
#import "core/test" as *
#import "platforms/gui/web" as *

fn main %impure fn unit i32 \unit:
    match gui_web_poll_event_result:
        Result::Ok event:
            assert is_none event
            0
        Result::Err _:
            1
```

## web event wrapper exposes pointer keyboard text input and window variants

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/char" as *
#import "core/gui" as *
#import "core/option" as *
#import "core/test" as *
#import "platforms/gui/web" as *
#import "std/gui/keymap" as *

fn main %impure fn unit i32 \unit:
    let point %GuiPoint gui_point_new 0 0
    let pointer %PointerEvent pointer_event_new PointerEventKind::Move 9 point PointerButton::None
    let pointer_web %GuiWebEvent GuiWebEvent 3 point gui_event_pointer pointer
    match gui_web_event_pointer &pointer_web:
        Option::Some event:
            match pointer_event_kind &event:
                PointerEventKind::Move:
                    assert_eq_i32 9 pointer_event_pointer_id &event
                _:
                    test_fail "pointer move kind mismatch"
        Option::None:
            test_fail "pointer event missing"
    let keyboard %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 9 1
    let keyboard_web %GuiWebEvent GuiWebEvent 3 point gui_event_keyboard keyboard
    match gui_web_event_keyboard &keyboard_web:
        Option::Some event:
            let key_map %FocusKeyMap focus_key_map_default
            let command %Option FocusRouteCommand keyboard_event_to_focus_route_command &key_map event
            assert is_some command
        Option::None:
            test_fail "keyboard event missing"
    let text %TextInputEvent text_input_event_new '\u{3042}'
    let text_web %GuiWebEvent GuiWebEvent 3 point gui_event_text_input text
    match gui_web_event_text_input &text_web:
        Option::Some event:
            assert_eq_i32 0x3042 char_to_i32 text_input_event_value &event
        Option::None:
            test_fail "text input event missing"
    let size %GuiSize gui_size_new 640 480
    let window %WindowEvent window_event_new WindowEventKind::Resized size
    let window_web %GuiWebEvent GuiWebEvent 3 point gui_event_window window
    match gui_web_event_window &window_web:
        Option::Some event:
            match window_event_kind &event:
                WindowEventKind::Resized:
                    let got_size %GuiSize window_event_size &event
                    assert_eq_i32 640 gui_size_width &got_size
                    assert_eq_i32 480 gui_size_height &got_size
                _:
                    test_fail "window kind mismatch"
        Option::None:
            test_fail "window event missing"
    let no_window %Option WindowEvent gui_web_event_window &keyboard_web
    assert is_none no_window
    0
```
