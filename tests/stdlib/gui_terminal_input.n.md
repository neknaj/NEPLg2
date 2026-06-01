# GUI terminal input normalization

このファイルは terminal raw byte が platform 層で typed GUI input event に正規化され、alloc/gui や application code へ raw terminal byte が露出しないことを固定します。

## gui_terminal_input_normalizes_keyboard_and_text_bytes

[目的/もくてき]:
- Tab / LF / CR / Space が std key code contract の `KeyboardEvent` になり、`std/gui/keymap` の focus command と同じ契約で解釈できることを確認します。
- Space は keyboard activation と text input の両方を返し、上位 state が focus activation と文字入力を分離できることを固定します。
- printable ASCII は text input のみ、範囲外 byte は `GuiError::InvalidCommand`、範囲内の未対応 control byte は event なしになることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n[6] ok\n[7] ok\n[8] ok\n[9] ok\n[10] ok\n[11] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "platforms/gui/terminal/input" as *
#import "core/char" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/keymap" as *
#import "std/test" as *

fn command_is_next %fn Option FocusRouteCommand bool \command:
    match command:
        Option::Some value:
            match value:
                FocusRouteCommand::Next:
                    true
                _:
                    false
        Option::None:
            false

fn command_is_activate %fn Option FocusRouteCommand bool \command:
    match command:
        Option::Some value:
            match value:
                FocusRouteCommand::Activate:
                    true
                _:
                    false
        Option::None:
            false

fn events_focus_command %fn &FocusKeyMap fn TerminalInputEvents Option FocusRouteCommand \key_map\events:
    match terminal_input_events_keyboard &events:
        Option::Some keyboard:
            keyboard_event_to_focus_route_command key_map keyboard
        Option::None:
            none

fn text_value_is %fn TerminalInputEvents fn i32 bool \events\expected:
    match terminal_input_events_text_input &events:
        Option::Some text:
            let value %char terminal_text_input_event_value &text
            eq expected char_to_i32 value
        Option::None:
            false

fn result_error_is_invalid_command %fn Result TerminalInputEvents GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _events:
            false

fn main %impure fn unit i32 \unit:
    let key_map %FocusKeyMap focus_key_map_default
    let tab %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 9
    let lf %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 10
    let cr %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 13
    let space %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 32
    let printable %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 65
    let control %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 1
    let invalid %Result TerminalInputEvents GuiError terminal_input_events_from_byte 256
    let tab_has_keyboard %bool terminal_input_events_has_keyboard &tab
    let tab_focus_next %bool command_is_next events_focus_command &key_map tab
    let tab_has_no_text %bool is_none terminal_input_events_text_input &tab
    let lf_focus_activate %bool command_is_activate events_focus_command &key_map lf
    let cr_focus_activate %bool command_is_activate events_focus_command &key_map cr
    let space_has_keyboard %bool terminal_input_events_has_keyboard &space
    let space_focus_activate %bool command_is_activate events_focus_command &key_map space
    let space_text_value %bool text_value_is space 32
    let printable_has_no_keyboard %bool is_none terminal_input_events_keyboard &printable
    let printable_text_value %bool text_value_is printable 65
    let invalid_rejected %bool result_error_is_invalid_command invalid
    let control_has_no_events %bool and (not terminal_input_events_has_keyboard &control) (not terminal_input_events_has_text_input &control)
    let checks:
        checks_new
        |> checks_push assert "tab has keyboard" tab_has_keyboard
        |> checks_push assert "tab focus next" tab_focus_next
        |> checks_push assert "tab has no text" tab_has_no_text
        |> checks_push assert "lf focus activate" lf_focus_activate
        |> checks_push assert "cr focus activate" cr_focus_activate
        |> checks_push assert "space has keyboard" space_has_keyboard
        |> checks_push assert "space focus activate" space_focus_activate
        |> checks_push assert "space text value" space_text_value
        |> checks_push assert "printable has no keyboard" printable_has_no_keyboard
        |> checks_push assert "printable text value" printable_text_value
        |> checks_push assert "invalid byte rejected" invalid_rejected
        |> checks_push assert "control byte has no events" control_has_no_events
    let shown checks_print_report checks
    checks_exit_code shown
```
