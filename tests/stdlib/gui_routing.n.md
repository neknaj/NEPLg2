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
    let root_id %WidgetId widget_id 1
    let button_id %WidgetId widget_id 2
    let action %ActionId action_id 7
    let root_hint %LayoutHint layout_hint_fixed 10 4
    let button_hint %LayoutHint layout_hint_fixed 4 2
    let button_config_value %ButtonConfig button_config button_id "Run" action
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let button_node %WidgetDescriptor widget_button button_config_value button_hint
    let view0 %ViewTree view_tree_single root
    let view_tree %ViewTree unwrap_ok view_tree_add_child view0 button_node
    let root_rect %GuiRect gui_rect_new 0 0 10 4
    let button_rect %GuiRect gui_rect_new 1 1 4 2
    let root_layout %LayoutNode layout_node root_id root_rect
    let button_layout %LayoutNode layout_node button_id button_rect
    let layout0 %LayoutTree layout_tree_single root_layout
    let layout_tree %LayoutTree unwrap_ok layout_tree_add_child layout0 button_layout
    let point %GuiPoint gui_point_new 2 1
    let hit_check match layout_tree_hit_test &layout_tree point:
        Option::Some id:
            assert_eq_i32 2 widget_id_value id
        Option::None:
            assert false
    let event_check match route_pointer_action &view_tree &layout_tree point:
        Option::Some event:
            match event:
                GuiEvent::Action action:
                    assert_eq_i32 7 action_id_value action
                _:
                    assert false
        Option::None:
            assert false
    let checks1 checks_push checks_new hit_check
    let checks checks_push checks1 event_check
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
    let root_id %WidgetId widget_id 1
    let button_id %WidgetId widget_id 2
    let action %ActionId action_id 7
    let root_hint %LayoutHint layout_hint_fixed 10 4
    let button_hint %LayoutHint layout_hint_fixed 4 2
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let config %ButtonConfig button_config button_id "Run" action
    let disabled_node %ViewNode button config
    let disabled %WidgetDescriptor widget_descriptor button_id disabled_node button_hint true true "Run"
    let view0 %ViewTree view_tree_single root
    let view_tree %ViewTree unwrap_ok view_tree_add_child view0 disabled
    let root_rect %GuiRect gui_rect_new 0 0 10 4
    let button_rect %GuiRect gui_rect_new 1 1 4 2
    let root_layout %LayoutNode layout_node root_id root_rect
    let button_layout %LayoutNode layout_node button_id button_rect
    let layout0 %LayoutTree layout_tree_single root_layout
    let layout_tree %LayoutTree unwrap_ok layout_tree_add_child layout0 button_layout
    let inside_point %GuiPoint gui_point_new 2 1
    let outside_point %GuiPoint gui_point_new 20 20
    let disabled_check assert is_none_event route_pointer_action &view_tree &layout_tree inside_point
    let outside_check assert is_none_event route_pointer_action &view_tree &layout_tree outside_point
    let checks1 checks_push checks_new disabled_check
    let checks checks_push checks1 outside_check
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
    let root_id %WidgetId widget_id 1
    let first_id %WidgetId widget_id 2
    let second_id %WidgetId widget_id 3
    let first_action %ActionId action_id 10
    let second_action %ActionId action_id 20
    let root_hint %LayoutHint layout_hint_fixed 10 4
    let button_hint %LayoutHint layout_hint_fixed 4 2
    let first_config %ButtonConfig button_config first_id "First" first_action
    let second_config %ButtonConfig button_config second_id "Second" second_action
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let first %WidgetDescriptor widget_button first_config button_hint
    let second %WidgetDescriptor widget_button second_config button_hint
    let view0 %ViewTree view_tree_single root
    let view1 %ViewTree unwrap_ok view_tree_add_child view0 first
    let view2 %ViewTree unwrap_ok view_tree_add_child view1 second
    let root_rect %GuiRect gui_rect_new 0 0 10 4
    let overlap_rect %GuiRect gui_rect_new 1 1 4 2
    let root_layout %LayoutNode layout_node root_id root_rect
    let first_layout %LayoutNode layout_node first_id overlap_rect
    let second_layout %LayoutNode layout_node second_id overlap_rect
    let layout0 %LayoutTree layout_tree_single root_layout
    let layout1 %LayoutTree unwrap_ok layout_tree_add_child layout0 first_layout
    let layout2 %LayoutTree unwrap_ok layout_tree_add_child layout1 second_layout
    let point %GuiPoint gui_point_new 2 1
    match route_pointer_action &view2 &layout2 point:
        Option::Some event:
            match event:
                GuiEvent::Action action:
                    action_id_value action
                _:
                    0
        Option::None:
            0
```
