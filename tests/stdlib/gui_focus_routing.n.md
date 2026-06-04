# alloc/gui routing focus

このファイルは focus/key routing が platform raw key code や ANSI sequence に依存せず、解釈済み command data から focus movement と action emission を分けて返すことを固定します。

## route_focus_command_moves_next_and_previous

[目的/もくてき]:
- `Next` / `Previous` が既存 focus traversal を使い、`MoveFocus` として移動先だけを返すことを確認します。
- current が `none` の場合でも traversal の開始点として扱われることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_focus_command_moves_next_and_previous\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"9\" actual=\"9\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"7\" actual=\"7\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/app" as *
#import "alloc/gui/routing/focus" as *
#import "alloc/gui/tree" as *
#import "alloc/gui/widget" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let first_id %WidgetId widget_id 7
    let second_id %WidgetId widget_id 9
    let first_action %ActionId action_id 2
    let second_action %ActionId action_id 3
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let button_hint %LayoutHint layout_hint_fixed 6 1
    let first_config %ButtonConfig button_config first_id "Run" first_action
    let second_config %ButtonConfig button_config second_id "Save" second_action
    let root %WidgetDescriptor widget_label root_id "title" root_hint
    let first %WidgetDescriptor widget_button first_config button_hint
    let second %WidgetDescriptor widget_button second_config button_hint
    let tree0 %ViewTree view_tree_single root
    let tree1 %ViewTree unwrap_ok view_tree_add_child tree0 first
    let tree2 %ViewTree unwrap_ok view_tree_add_child tree1 second
    let start_next_check match route_focus_command &tree2 none FocusRouteCommand::Next:
        FocusRouteResult::MoveFocus id:
            assert_eq_i32 7 widget_id_value id
        _:
            assert false
    let first_current %Option WidgetId some first_id
    let second_current %Option WidgetId some second_id
    let step_next_check match route_focus_command &tree2 first_current FocusRouteCommand::Next:
        FocusRouteResult::MoveFocus id:
            assert_eq_i32 9 widget_id_value id
        _:
            assert false
    let step_previous_check match route_focus_command &tree2 second_current FocusRouteCommand::Previous:
        FocusRouteResult::MoveFocus id:
            assert_eq_i32 7 widget_id_value id
        _:
            assert false
    let checks1 test_report_push test_report_new "route_focus_command_moves_next_and_previous" start_next_check
    let checks2 test_report_push checks1 step_next_check
    let checks test_report_push checks2 step_previous_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## route_focus_command_emits_action_for_current_focus

[目的/もくてき]:
- `Activate` が current focus id から `GuiEvent::Action` を返すことを確認します。
- focus movement と action emission が別 variant で表されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_focus_command_emits_action_for_current_focus\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"42\" actual=\"42\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/app" as *
#import "alloc/gui/routing/focus" as *
#import "alloc/gui/tree" as *
#import "alloc/gui/widget" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let button_id %WidgetId widget_id 7
    let action %ActionId action_id 42
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let button_hint %LayoutHint layout_hint_fixed 6 1
    let button_config_value %ButtonConfig button_config button_id "Run" action
    let root %WidgetDescriptor widget_label root_id "title" root_hint
    let button_node %WidgetDescriptor widget_button button_config_value button_hint
    let tree0 %ViewTree view_tree_single root
    let tree %ViewTree unwrap_ok view_tree_add_child tree0 button_node
    let current %Option WidgetId some button_id
    let action_check match route_focus_command &tree current FocusRouteCommand::Activate:
        FocusRouteResult::Emit event:
            match event:
                GuiEvent::Action action:
                    assert_eq_i32 42 action_id_value action
                _:
                    assert false
        _:
            assert false
    let checks test_report_push test_report_new "route_focus_command_emits_action_for_current_focus" action_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## route_focus_command_ignores_invalid_activation

[目的/もくてき]:
- disabled widget、action を持たない label、古い current id は panic せず `Ignored` になることを確認します。
- traversal で移動先がない場合も `Ignored` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_focus_command_ignores_invalid_activation\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/app" as *
#import "alloc/gui/routing/focus" as *
#import "alloc/gui/tree" as *
#import "alloc/gui/widget" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn is_ignored %fn FocusRouteResult bool \result:
    match result:
        FocusRouteResult::Ignored:
            true
        _:
            false

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let button_id %WidgetId widget_id 7
    let stale_id %WidgetId widget_id 99
    let action %ActionId action_id 42
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let button_hint %LayoutHint layout_hint_fixed 6 1
    let root %WidgetDescriptor widget_label root_id "title" root_hint
    let config %ButtonConfig button_config button_id "Run" action
    let disabled_node %ViewNode button config
    let disabled %WidgetDescriptor widget_descriptor button_id disabled_node button_hint true true "Run"
    let tree0 %ViewTree view_tree_single root
    let tree %ViewTree unwrap_ok view_tree_add_child tree0 disabled
    let button_current %Option WidgetId some button_id
    let label_current %Option WidgetId some root_id
    let stale_current %Option WidgetId some stale_id
    let disabled_check assert is_ignored route_focus_command &tree button_current FocusRouteCommand::Activate
    let label_check assert is_ignored route_focus_command &tree label_current FocusRouteCommand::Activate
    let stale_check assert is_ignored route_focus_command &tree stale_current FocusRouteCommand::Activate
    let edge_check assert is_ignored route_focus_command &tree button_current FocusRouteCommand::Next
    let checks1 test_report_push test_report_new "route_focus_command_ignores_invalid_activation" disabled_check
    let checks2 test_report_push checks1 label_check
    let checks3 test_report_push checks2 stale_check
    let checks test_report_push checks3 edge_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```
