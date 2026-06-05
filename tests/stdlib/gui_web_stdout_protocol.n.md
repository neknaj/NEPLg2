# platforms/gui/web stdout protocol

Web Playground backend の stdout fallback が、drawing と input hit target を typed helper から安定した順序で出力することを確認する。

## button helper emits fill label and action target

[目的/もくてき]:
- `GuiWebButtonConfig` が button の geometry、action、label、色、text layout offset を 1 つの struct として保持することを確認します。
- `gui_web_stdout_button` が `fill_rect -> text_run -> action_rect` の順序を example 側へ重複させずに出力することを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "NEPLG2_GUI_FILL_RECT 1 2 30 12 10 20 30 255\nNEPLG2_GUI_TEXT_RUN 16 5 11 center 200 210 220 255 Run\nNEPLG2_GUI_ACTION_RECT 1 2 30 12 7\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/cast" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/result" as *
#import "platforms/gui/web" as *

fn rgba %fn i32 fn i32 fn i32 Rgba8888 \r\g\b:
    let red %u8 cast r
    let green %u8 cast g
    let blue %u8 cast b
    let alpha %u8 cast 255
    rgba8888_new red green blue alpha

fn main %impure fn void i32 \void:
    let rect %GuiRect gui_rect_new 1 2 30 12
    let action %ActionId action_id_new 7
    let fill %Rgba8888 rgba 10 20 30
    let text %Rgba8888 rgba 200 210 220
    let config %GuiWebButtonConfig gui_web_button_config rect action "Run" fill text 11 3
    match gui_web_stdout_button config:
        Result::Ok _:
            0
        Result::Err _:
            1
```

## button helper rejects invalid config before stdout output

[目的/もくてき]:
- invalid rect、invalid action id、invalid text size が `GuiError::InvalidGeometry` になることを確認します。
- error 時に button の一部だけが stdout protocol へ出ないことを、空 stdout で確認します。

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/cast" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/result" as *
#import "platforms/gui/web" as *

fn rgba %fn i32 fn i32 fn i32 Rgba8888 \r\g\b:
    let red %u8 cast r
    let green %u8 cast g
    let blue %u8 cast b
    let alpha %u8 cast 255
    rgba8888_new red green blue alpha

fn is_invalid_geometry %fn Result unit GuiError bool \result:
    match result:
        Result::Ok _:
            false
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    true
                _:
                    false

fn main %impure fn void i32 \void:
    let rect %GuiRect gui_rect_new 1 2 30 12
    let invalid_rect %GuiRect gui_rect_new 1 2 -1 12
    let action %ActionId action_id_new 7
    let invalid_action %ActionId action_id_new 0
    let fill %Rgba8888 rgba 10 20 30
    let text %Rgba8888 rgba 200 210 220
    let invalid_rect_config %GuiWebButtonConfig gui_web_button_config invalid_rect action "Run" fill text 11 3
    let invalid_action_config %GuiWebButtonConfig gui_web_button_config rect invalid_action "Run" fill text 11 3
    let invalid_text_config %GuiWebButtonConfig gui_web_button_config rect action "Run" fill text 0 3
    let invalid_rect_result %Result unit GuiError gui_web_stdout_button invalid_rect_config
    let invalid_action_result %Result unit GuiError gui_web_stdout_button invalid_action_config
    let invalid_text_result %Result unit GuiError gui_web_stdout_button invalid_text_config
    let rect_ok %bool is_invalid_geometry invalid_rect_result
    let action_ok %bool is_invalid_geometry invalid_action_result
    let text_ok %bool is_invalid_geometry invalid_text_result
    let action_and_text_ok %bool and action_ok text_ok
    if:
        and rect_ok action_and_text_ok
        then:
            0
        else:
            1
```
