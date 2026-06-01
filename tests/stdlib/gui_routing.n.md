# alloc/gui routing

このファイルは pointer 位置から `LayoutTree` の hit target を決め、`ViewTree` の widget data から `GuiEvent::Action` を導出する pure routing contract を固定します。

## route_pointer_action_hits_button_child

[目的/もくてき]:
- layout hit test が root ではなく child button を優先することを確認します。
- button activation は callback ではなく `GuiEvent::Action` として返ることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "alloc/gui/routing" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "root" (layout_hint_fixed 10 4)
    let button_node %WidgetDescriptor widget_button (button_config (widget_id 2) "Run" (action_id 7)) (layout_hint_fixed 4 2)
    let view_tree %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) button_node
    let root_layout %LayoutNode layout_node (widget_id 1) (gui_rect_new 0 0 10 4)
    let button_layout %LayoutNode layout_node (widget_id 2) (gui_rect_new 1 1 4 2)
    let layout_tree %LayoutTree unwrap_ok layout_tree_add_child (layout_tree_single root_layout) button_layout
    let hit_check match layout_tree_hit_test &layout_tree (gui_point_new 2 1):
        Option::Some id:
            assert_eq_i32 2 widget_id_value id
        Option::None:
            assert false
    let event_check match route_pointer_action &view_tree &layout_tree (gui_point_new 2 1):
        Option::Some event:
            match event:
                GuiEvent::Action action:
                    assert_eq_i32 7 action_id_value action
                _:
                    assert false
        Option::None:
            assert false
    let checks checks_push (checks_push checks_new hit_check) event_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## route_pointer_action_ignores_disabled_and_outside

[目的/もくてき]:
- disabled widget は hit しても action event を返さないことを確認します。
- layout bounds 外の pointer は `Option::None` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "alloc/gui/routing" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn is_none_event %fn Option GuiEvent bool \value:
    match value:
        Option::Some _event:
            false
        Option::None:
            true

fn main %impure fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "root" (layout_hint_fixed 10 4)
    let config %ButtonConfig button_config (widget_id 2) "Run" (action_id 7)
    let disabled %WidgetDescriptor widget_descriptor (widget_id 2) (button config) (layout_hint_fixed 4 2) true true "Run"
    let view_tree %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) disabled
    let root_layout %LayoutNode layout_node (widget_id 1) (gui_rect_new 0 0 10 4)
    let button_layout %LayoutNode layout_node (widget_id 2) (gui_rect_new 1 1 4 2)
    let layout_tree %LayoutTree unwrap_ok layout_tree_add_child (layout_tree_single root_layout) button_layout
    let disabled_check assert is_none_event route_pointer_action &view_tree &layout_tree (gui_point_new 2 1)
    let outside_check assert is_none_event route_pointer_action &view_tree &layout_tree (gui_point_new 20 20)
    let checks checks_push (checks_push checks_new disabled_check) outside_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## route_pointer_action_uses_second_child_as_topmost

[目的/もくてき]:
- child 同士が重なった場合、second child が first child より手前として hit されることを確認します。

neplg2:test
ret: 20
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "alloc/gui/routing" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "root" (layout_hint_fixed 10 4)
    let first %WidgetDescriptor widget_button (button_config (widget_id 2) "First" (action_id 10)) (layout_hint_fixed 4 2)
    let second %WidgetDescriptor widget_button (button_config (widget_id 3) "Second" (action_id 20)) (layout_hint_fixed 4 2)
    let view1 %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) first
    let view2 %ViewTree unwrap_ok view_tree_add_child view1 second
    let root_layout %LayoutNode layout_node (widget_id 1) (gui_rect_new 0 0 10 4)
    let first_layout %LayoutNode layout_node (widget_id 2) (gui_rect_new 1 1 4 2)
    let second_layout %LayoutNode layout_node (widget_id 3) (gui_rect_new 1 1 4 2)
    let layout1 %LayoutTree unwrap_ok layout_tree_add_child (layout_tree_single root_layout) first_layout
    let layout2 %LayoutTree unwrap_ok layout_tree_add_child layout1 second_layout
    match route_pointer_action &view2 &layout2 (gui_point_new 2 1):
        Option::Some event:
            match event:
                GuiEvent::Action action:
                    action_id_value action
                _:
                    0
        Option::None:
            0
```
