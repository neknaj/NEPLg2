# alloc/gui widget

このファイルは GUI / TUI 共通 widget snapshot が callback や platform handle を持たず、action identifier と semantic tree へ lowering できることを固定します。

## widget_button_activation_is_action_event

[目的/もくてき]:
- button activation が closure 呼び出しではなく `GuiEvent::Action` として返ることを確認します。
- disabled widget は action event を返さないことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let id %WidgetId widget_id 10
    let action %ActionId action_id 55
    let config %ButtonConfig button_config id "Run" action
    let hint %LayoutHint layout_hint_fixed 8 2
    let enabled %WidgetDescriptor widget_button config hint
    let button_node %ViewNode button config
    let disabled %WidgetDescriptor widget_descriptor id button_node hint true true "Run"
    let enabled_check match widget_action_event &enabled:
        Option::Some event:
            match event:
                GuiEvent::Action action:
                    assert_eq_i32 55 action_id_raw &action
                _:
                    assert false
        Option::None:
            assert false
    let disabled_check match widget_action_event &disabled:
        Option::Some _event:
            assert false
        Option::None:
            assert true
    let checks1 checks_push checks_new enabled_check
    let checks checks_push checks1 disabled_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## widget_semantics_are_generated_from_widget_data

[目的/もくてき]:
- semantic role が draw command から逆算されず、widget snapshot から生成されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let id %WidgetId widget_id 9
    let action %ActionId action_id 4
    let config %ButtonConfig button_config id "Save" action
    let hint %LayoutHint layout_hint_fixed 8 2
    let descriptor %WidgetDescriptor widget_button config hint
    let semantic %SemanticNode widget_semantic_node &descriptor
    let check assert semantic_node_is_button &semantic
    let checks checks_push checks_new check
    let shown checks_print_report checks
    checks_exit_code shown
```
