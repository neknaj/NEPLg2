# std/gui keymap

このファイルは platform-specific raw keyboard input が application へ直接渡らず、std layer の key code contract から `FocusRouteCommand` へ写像されることを固定します。

## keyboard_event_to_focus_route_command_maps_default_focus_keys

[目的/もくてき]:
- portable default map が Tab / Shift+Tab / Enter / Space を focus routing command へ変換することを確認します。
- `alloc/gui/routing/focus` は platform-specific raw key code や modifier bit を知らず、変換後の `FocusRouteCommand` だけを受け取る契約を固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui" as *
#import "core/option" as *
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

fn main %impure fn unit i32 \unit:
    let key_map %FocusKeyMap focus_key_map_default
    let tab %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 9 0
    let shift_tab %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 9 1
    let enter %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 13 0
    let space %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 32 0
    let check0 assert command_is_next keyboard_event_to_focus_route_command &key_map tab
    let check1 assert command_is_previous keyboard_event_to_focus_route_command &key_map shift_tab
    let check2 assert command_is_activate keyboard_event_to_focus_route_command &key_map enter
    let check3 assert command_is_activate keyboard_event_to_focus_route_command &key_map space
    let checks checks_push (checks_push (checks_push (checks_push checks_new check0) check1) check2) check3
    let shown checks_print_report checks
    checks_exit_code shown
```

## keyboard_event_to_focus_route_command_ignores_key_up_and_unknown_keys

[目的/もくてき]:
- `KeyUp` は focus command を発生させないことを確認します。
- default map にない key code は `Option::None` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui" as *
#import "core/option" as *
#import "std/gui/keymap" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let key_map %FocusKeyMap focus_key_map_default
    let key_up %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyUp 9 1
    let unknown %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 65 0
    let check0 assert is_none keyboard_event_to_focus_route_command &key_map key_up
    let check1 assert is_none keyboard_event_to_focus_route_command &key_map unknown
    let checks checks_push (checks_push checks_new check0) check1
    let shown checks_print_report checks
    checks_exit_code shown
```

## keyboard_event_to_focus_route_command_uses_custom_map

[目的/もくてき]:
- platform adapter が key code と Shift mask を差し替えても、std layer で同じ focus command data に変換できることを確認します。
- DOM / ANSI / OS API 名ではなく、明示的な数値 contract だけに依存することを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui" as *
#import "core/option" as *
#import "std/gui/keymap" as *
#import "std/test" as *

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

fn main %impure fn unit i32 \unit:
    let key_map %FocusKeyMap focus_key_map 100 101 102 4
    let shift_tab %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 100 4
    let space %KeyboardEvent keyboard_event_from_key_code KeyboardEventKind::KeyDown 102 0
    let check0 assert command_is_previous keyboard_event_to_focus_route_command &key_map shift_tab
    let check1 assert command_is_activate keyboard_event_to_focus_route_command &key_map space
    let checks checks_push (checks_push checks_new check0) check1
    let shown checks_print_report checks
    checks_exit_code shown
```
