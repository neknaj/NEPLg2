# alloc/gui app model

このファイルは GUI / TUI 共通 UI substrate の alloc layer 初期 contract を固定します。

## gui_app_action_identifier_button_has_no_callback

[目的/もくてき]:
- button が closure ではなく `ActionId` を持つことを固定します。
- application は raw platform event ではなく action identifier を `update` で扱う前提にします。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_app_action_identifier_button_has_no_callback\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"42\" actual=\"42\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let id %WidgetId widget_id 7
    let action %ActionId action_id 42
    let config %ButtonConfig button_config id "Save" action
    let node %ViewNode button config
    let checks match node:
        ViewNode::Button cfg:
            let got_id %WidgetId get cfg "id"
            let got_action %ActionId get cfg "action"
            test_report_new "gui_app_action_identifier_button_has_no_callback"
            |> test_report_push assert_eq_i32 7 widget_id_value got_id
            |> test_report_push assert_eq_i32 42 action_id_value got_action
        ViewNode::Label _text:
            test_report_push test_report_new "gui_app_action_identifier_button_has_no_callback" assert false
        ViewNode::Empty:
            test_report_push test_report_new "gui_app_action_identifier_button_has_no_callback" assert false
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_app_update_can_batch_multiple_effects

[目的/もくてき]:
- `update` が redraw と title 変更のような複数 effect を data として返せることを固定します。
- 現 checkpoint の bounded batch が capacity overflow を `GuiError::ResourceExhausted` として返すことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_app_update_can_batch_multiple_effects\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/field" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let batch0 %GuiEffectBatch gui_effect_batch_empty
    let batch1 %GuiEffectBatch unwrap_ok gui_effect_batch_push batch0 request_redraw 1
    let batch2 %GuiEffectBatch unwrap_ok gui_effect_batch_push batch1 set_title 1 "Main"
    let upd %Update i32 update_result_batch 44 batch2
    let effects %GuiEffectBatch update_effects upd
    let first_check match gui_effect_batch_first &effects:
        Option::Some effect:
            match effect:
                GuiEffect::RequestRedraw payload:
                    assert_eq_i32 1 get payload "target"
                _:
                    assert false
        Option::None:
            assert false
    let second_check match gui_effect_batch_second &effects:
        Option::Some effect:
            match effect:
                GuiEffect::SetTitle payload:
                    assert_eq_i32 1 get payload "target"
                _:
                    assert false
        Option::None:
            assert false
    let overflow_check match gui_effect_batch_push effects request_redraw 2:
        Result::Ok _next:
            assert false
        Result::Err error:
            match error:
                GuiError::ResourceExhausted:
                    assert true
                _:
                    assert false
    let checks1 test_report_push test_report_new "gui_app_update_can_batch_multiple_effects" first_check
    let checks2 test_report_push checks1 second_check
    let checks test_report_push checks2 overflow_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_app_update_returns_model_and_effect

[目的/もくてき]:
- `update` が host API を直接呼ばず、model と `GuiEffect` を返す形を固定します。
- redraw request が data として保持されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_app_update_returns_model_and_effect\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"3\" actual=\"3\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let upd %Update i32 update_result 11 request_redraw 3
    let effect %GuiEffect update_effect upd
    let checks match effect:
        GuiEffect::RequestRedraw payload:
            test_report_push test_report_new "gui_app_update_returns_model_and_effect" assert_eq_i32 3 get payload "target"
        GuiEffect::None:
            test_report_push test_report_new "gui_app_update_returns_model_and_effect" assert false
        GuiEffect::SetTitle _payload:
            test_report_push test_report_new "gui_app_update_returns_model_and_effect" assert false
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```
