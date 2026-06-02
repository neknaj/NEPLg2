# GUI terminal input normalization

このファイルは terminal raw byte が platform 層で typed GUI input event に正規化され、alloc/gui や application code へ raw terminal byte が露出しないことを固定します。

## gui_terminal_input_normalizes_keyboard_and_text_bytes

[目的/もくてき]:
- Tab / LF / CR / Space が std key code contract の `KeyboardEvent` になり、`std/gui/keymap` の focus command と同じ契約で解釈できることを確認します。
- Space は keyboard activation と text input の両方を返し、上位 state が focus activation と文字入力を分離できることを固定します。
- printable ASCII は text input のみ、範囲外 byte は `GuiError::InvalidCommand`、範囲内の未対応 control byte は event なしになることを確認します。
- `ESC [ Z` は Shift+Tab として key code contract に正規化され、`std/gui/keymap` 経由で `Previous` へ写像できることを確認します。
- `ESC [ A/B/C/D` は std navigation key code に正規化され、未対応の範囲内 3 byte sequence は event なし、範囲外 byte を含む 3 byte sequence は `GuiError::InvalidCommand` になることを確認します。
- `ESC [ 1 ; <modifier> A/B/C/D` は xterm style modifier 付き arrow key として正規化され、未知 final は event なし、arrow key に対する不正 modifier parameter は `GuiError::InvalidCommand` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n[6] ok\n[7] ok\n[8] ok\n[9] ok\n[10] ok\n[11] ok\n[12] ok\n[13] ok\n[14] ok\n[15] ok\n[16] ok\n[17] ok\n[18] ok\n[19] ok\n[20] ok\n[21] ok\n[22] ok\n[23] ok\n[24] ok\n[25] ok\n[26] ok\n[27] ok\n"
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

fn command_is_previous %fn Option FocusRouteCommand bool \command:
    match command:
        Option::Some value:
            match value:
                FocusRouteCommand::Previous:
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

fn keyboard_key_code_is %fn TerminalInputEvents fn i32 bool \events\expected:
    match terminal_input_events_keyboard &events:
        Option::Some keyboard:
            eq expected keyboard_event_key_code &keyboard
        Option::None:
            false

fn keyboard_modifier_has %fn TerminalInputEvents fn i32 bool \events\mask:
    match terminal_input_events_keyboard &events:
        Option::Some keyboard:
            let modifier_bits %i32 keyboard_event_modifier_bits &keyboard
            let masked_bits %i32 and modifier_bits mask
            eq mask masked_bits
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

fn main %impure fn void i32 \void:
    let key_map %FocusKeyMap focus_key_map_default
    let tab %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 9
    let lf %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 10
    let cr %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 13
    let space %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 32
    let printable %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 65
    let control %TerminalInputEvents unwrap_ok terminal_input_events_from_byte 1
    let invalid %Result TerminalInputEvents GuiError terminal_input_events_from_byte 256
    let shift_tab %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 90
    let arrow_up %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 65
    let arrow_down %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 66
    let arrow_right %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 67
    let arrow_left %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 68
    let unknown_escape %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 81
    let invalid_escape %Result TerminalInputEvents GuiError terminal_input_events_from_escape3 27 91 300
    let modified_arrow_up %TerminalInputEvents unwrap_ok terminal_input_events_from_csi6 27 91 49 59 50 65
    let modified_arrow_left %TerminalInputEvents unwrap_ok terminal_input_events_from_csi6 27 91 49 59 53 68
    let unknown_csi %TerminalInputEvents unwrap_ok terminal_input_events_from_csi6 27 91 49 59 50 90
    let invalid_modifier %Result TerminalInputEvents GuiError terminal_input_events_from_csi6 27 91 49 59 57 65
    let invalid_csi %Result TerminalInputEvents GuiError terminal_input_events_from_csi6 27 91 49 59 50 300
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
    let control_has_keyboard %bool terminal_input_events_has_keyboard &control
    let control_has_text %bool terminal_input_events_has_text_input &control
    let control_no_keyboard %bool not control_has_keyboard
    let control_no_text %bool not control_has_text
    let control_has_no_events %bool and control_no_keyboard control_no_text
    let shift_tab_has_keyboard %bool terminal_input_events_has_keyboard &shift_tab
    let shift_tab_focus_previous %bool command_is_previous events_focus_command &key_map shift_tab
    let shift_tab_has_no_text %bool is_none terminal_input_events_text_input &shift_tab
    let arrow_up_key_code %bool keyboard_key_code_is arrow_up key_code_arrow_up
    let arrow_down_key_code %bool keyboard_key_code_is arrow_down key_code_arrow_down
    let arrow_right_key_code %bool keyboard_key_code_is arrow_right key_code_arrow_right
    let arrow_left_key_code %bool keyboard_key_code_is arrow_left key_code_arrow_left
    let unknown_escape_has_keyboard %bool terminal_input_events_has_keyboard &unknown_escape
    let unknown_escape_has_text %bool terminal_input_events_has_text_input &unknown_escape
    let unknown_escape_no_keyboard %bool not unknown_escape_has_keyboard
    let unknown_escape_no_text %bool not unknown_escape_has_text
    let unknown_escape_has_no_events %bool and unknown_escape_no_keyboard unknown_escape_no_text
    let invalid_escape_rejected %bool result_error_is_invalid_command invalid_escape
    let modified_arrow_up_key_code %bool keyboard_key_code_is modified_arrow_up key_code_arrow_up
    let modified_arrow_up_has_shift %bool keyboard_modifier_has modified_arrow_up key_modifier_shift_bit
    let modified_arrow_left_key_code %bool keyboard_key_code_is modified_arrow_left key_code_arrow_left
    let modified_arrow_left_has_control %bool keyboard_modifier_has modified_arrow_left key_modifier_control_bit
    let unknown_csi_has_keyboard %bool terminal_input_events_has_keyboard &unknown_csi
    let unknown_csi_has_text %bool terminal_input_events_has_text_input &unknown_csi
    let unknown_csi_no_keyboard %bool not unknown_csi_has_keyboard
    let unknown_csi_no_text %bool not unknown_csi_has_text
    let unknown_csi_has_no_events %bool and unknown_csi_no_keyboard unknown_csi_no_text
    let invalid_modifier_rejected %bool result_error_is_invalid_command invalid_modifier
    let invalid_csi_rejected %bool result_error_is_invalid_command invalid_csi
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
        |> checks_push assert "shift tab has keyboard" shift_tab_has_keyboard
        |> checks_push assert "shift tab focus previous" shift_tab_focus_previous
        |> checks_push assert "shift tab has no text" shift_tab_has_no_text
        |> checks_push assert "arrow up key code" arrow_up_key_code
        |> checks_push assert "arrow down key code" arrow_down_key_code
        |> checks_push assert "arrow right key code" arrow_right_key_code
        |> checks_push assert "arrow left key code" arrow_left_key_code
        |> checks_push assert "unknown escape has no events" unknown_escape_has_no_events
        |> checks_push assert "invalid escape rejected" invalid_escape_rejected
        |> checks_push assert "modified arrow up key code" modified_arrow_up_key_code
        |> checks_push assert "modified arrow up has shift" modified_arrow_up_has_shift
        |> checks_push assert "modified arrow left key code" modified_arrow_left_key_code
        |> checks_push assert "modified arrow left has control" modified_arrow_left_has_control
        |> checks_push assert "unknown csi has no events" unknown_csi_has_no_events
        |> checks_push assert "invalid modifier rejected" invalid_modifier_rejected
        |> checks_push assert "invalid csi rejected" invalid_csi_rejected
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_terminal_input_normalizes_home_end_delete_csi

[目的/もくてき]:
- `ESC [ H/F` と `ESC [ 1/3/4 ~` が raw CSI ではなく typed `KeyboardEvent` として返ることを確認します。
- terminal backend は `TerminalInputEvents` だけを返し、`FocusRouteCommand` や `ActionId` を作らない責務境界を維持します。
- `ESC [ <digit> ~` の未知 numeric parameter は event なし、numeric parameter として不正な byte は `GuiError::InvalidCommand` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n[6] ok\n[7] ok\n[8] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "platforms/gui/terminal/input" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/keymap" as *
#import "std/test" as *

fn keyboard_key_code_is %fn TerminalInputEvents fn i32 bool \events\expected:
    match terminal_input_events_keyboard &events:
        Option::Some keyboard:
            eq expected keyboard_event_key_code &keyboard
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

fn no_events %fn TerminalInputEvents bool \events:
    let has_keyboard %bool terminal_input_events_has_keyboard &events
    let has_text %bool terminal_input_events_has_text_input &events
    let no_keyboard %bool not has_keyboard
    let no_text %bool not has_text
    and no_keyboard no_text

fn main %impure fn void i32 \void:
    let home_h %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 72
    let end_f %TerminalInputEvents unwrap_ok terminal_input_events_from_escape3 27 91 70
    let home_tilde %TerminalInputEvents unwrap_ok terminal_input_events_from_csi4 27 91 49 126
    let delete_tilde %TerminalInputEvents unwrap_ok terminal_input_events_from_csi4 27 91 51 126
    let end_tilde %TerminalInputEvents unwrap_ok terminal_input_events_from_csi4 27 91 52 126
    let insert_tilde %TerminalInputEvents unwrap_ok terminal_input_events_from_csi4 27 91 50 126
    let invalid_param %Result TerminalInputEvents GuiError terminal_input_events_from_csi4 27 91 58 126
    let unknown_final %TerminalInputEvents unwrap_ok terminal_input_events_from_csi4 27 91 49 88
    let invalid_byte %Result TerminalInputEvents GuiError terminal_input_events_from_csi4 27 91 49 300
    let check0 assert "home ESC[H key code" keyboard_key_code_is home_h terminal_key_code_home
    let check1 assert "end ESC[F key code" keyboard_key_code_is end_f terminal_key_code_end
    let check2 assert "home tilde key code" keyboard_key_code_is home_tilde terminal_key_code_home
    let check3 assert "delete tilde key code" keyboard_key_code_is delete_tilde terminal_key_code_delete
    let check4 assert "end tilde key code" keyboard_key_code_is end_tilde terminal_key_code_end
    let check5 assert "unknown numeric tilde has no events" no_events insert_tilde
    let check6 assert "invalid numeric param rejected" result_error_is_invalid_command invalid_param
    let check7 assert "unknown final has no events" no_events unknown_final
    let check8 assert "invalid csi4 byte rejected" result_error_is_invalid_command invalid_byte
    let checks:
        checks_new
        |> checks_push check0
        |> checks_push check1
        |> checks_push check2
        |> checks_push check3
        |> checks_push check4
        |> checks_push check5
        |> checks_push check6
        |> checks_push check7
        |> checks_push check8
    let shown checks_print_report checks
    checks_exit_code shown
```
