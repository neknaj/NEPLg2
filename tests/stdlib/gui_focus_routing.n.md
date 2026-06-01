# alloc/gui routing focus

このファイルは focus/key routing が platform raw key code や ANSI sequence に依存せず、解釈済み command data から focus movement と action emission を分けて返すことを固定します。

## route_focus_command_moves_next_and_previous

[目的/もくてき]:
- `Next` / `Previous` が既存 focus traversal を使い、`MoveFocus` として移動先だけを返すことを確認します。
- current が `none` の場合でも traversal の開始点として扱われることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n"
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

fn main %impure fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "title" (layout_hint_fixed 8 1)
    let first %WidgetDescriptor widget_button (button_config (widget_id 7) "Run" (action_id 2)) (layout_hint_fixed 6 1)
    let second %WidgetDescriptor widget_button (button_config (widget_id 9) "Save" (action_id 3)) (layout_hint_fixed 6 1)
    let tree1 %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) first
    let tree2 %ViewTree unwrap_ok view_tree_add_child tree1 second
    let start_next_check match route_focus_command &tree2 none FocusRouteCommand::Next:
        FocusRouteResult::MoveFocus id:
            assert_eq_i32 7 widget_id_value id
        _:
            assert false
    let step_next_check match route_focus_command &tree2 (some (widget_id 7)) FocusRouteCommand::Next:
        FocusRouteResult::MoveFocus id:
            assert_eq_i32 9 widget_id_value id
        _:
            assert false
    let step_previous_check match route_focus_command &tree2 (some (widget_id 9)) FocusRouteCommand::Previous:
        FocusRouteResult::MoveFocus id:
            assert_eq_i32 7 widget_id_value id
        _:
            assert false
    let checks checks_push (checks_push (checks_push checks_new start_next_check) step_next_check) step_previous_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## route_focus_command_emits_action_for_current_focus

[目的/もくてき]:
- `Activate` が current focus id から `GuiEvent::Action` を返すことを確認します。
- focus movement と action emission が別 variant で表されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
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

fn main %impure fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "title" (layout_hint_fixed 8 1)
    let button_node %WidgetDescriptor widget_button (button_config (widget_id 7) "Run" (action_id 42)) (layout_hint_fixed 6 1)
    let tree %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) button_node
    let action_check match route_focus_command &tree (some (widget_id 7)) FocusRouteCommand::Activate:
        FocusRouteResult::Emit event:
            match event:
                GuiEvent::Action action:
                    assert_eq_i32 42 action_id_value action
                _:
                    assert false
        _:
            assert false
    let checks checks_push checks_new action_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## route_focus_command_ignores_invalid_activation

[目的/もくてき]:
- disabled widget、action を持たない label、古い current id は panic せず `Ignored` になることを確認します。
- traversal で移動先がない場合も `Ignored` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n"
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

fn main %impure fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "title" (layout_hint_fixed 8 1)
    let config %ButtonConfig button_config (widget_id 7) "Run" (action_id 42)
    let disabled %WidgetDescriptor widget_descriptor (widget_id 7) (button config) (layout_hint_fixed 6 1) true true "Run"
    let tree %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) disabled
    let disabled_check assert is_ignored route_focus_command &tree (some (widget_id 7)) FocusRouteCommand::Activate
    let label_check assert is_ignored route_focus_command &tree (some (widget_id 1)) FocusRouteCommand::Activate
    let stale_check assert is_ignored route_focus_command &tree (some (widget_id 99)) FocusRouteCommand::Activate
    let edge_check assert is_ignored route_focus_command &tree (some (widget_id 7)) FocusRouteCommand::Next
    let checks checks_push (checks_push (checks_push (checks_push checks_new disabled_check) label_check) stale_check) edge_check
    let shown checks_print_report checks
    checks_exit_code shown
```
