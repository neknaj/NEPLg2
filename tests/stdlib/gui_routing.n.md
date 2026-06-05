# alloc/gui routing

このファイルは pointer 位置から `LayoutTree` の hit target を決め、`ViewTree` の widget data から `GuiEvent::Action` を導出する pure routing contract を固定します。

## route_pointer_action_hits_button_child

[目的/もくてき]:
- layout hit test が root ではなく child button を優先することを確認します。
- button activation は callback ではなく `GuiEvent::Action` として返ることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_pointer_action_hits_button_child\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"7\" actual=\"7\" message=\"\"\n"
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

fn main %impure fn void i32 \void:
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
    let checks1 test_report_push test_report_new "route_pointer_action_hits_button_child" hit_check
    let checks test_report_push checks1 event_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## route_pointer_action_ignores_disabled_and_outside

[目的/もくてき]:
- disabled widget は hit しても action event を返さないことを確認します。
- layout bounds 外の pointer は `Option::None` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_pointer_action_ignores_disabled_and_outside\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"assert\" expected=\"true\" actual=\"true\" message=\"\"\n"
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

fn main %impure fn void i32 \void:
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
    let checks1 test_report_push test_report_new "route_pointer_action_ignores_disabled_and_outside" disabled_check
    let checks test_report_push checks1 outside_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## route_pointer_action_uses_second_child_as_topmost

[目的/もくてき]:
- child 同士が重なった場合、second child が first child より手前として hit されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_pointer_action_uses_second_child_as_topmost\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"20\" actual=\"20\" message=\"\"\n"
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

fn run_case %fn void i32 \void:
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

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "route_pointer_action_uses_second_child_as_topmost"
        |> test_report_push assert_eq_i32 "return value" 20 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## route_pointer_action_in_arena_hits_nested_front_node

[目的/もくてき]:
- allocator-backed `ViewTreeArena` / `LayoutTreeArena` で nested child の pointer routing が action event を返すことを確認します。
- arena hit test は後から追加された layout node を前面として扱い、root や parent button より nested button を優先することを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_pointer_action_in_arena_hits_nested_front_node\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"30\" actual=\"30\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "alloc/gui/routing" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn gui_event_action_value %fn Option GuiEvent i32 \event:
    match event:
        Option::Some value:
            match value:
                GuiEvent::Action action:
                    action_id_value action
                _:
                    0
        Option::None:
            0

fn run_case %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let parent_id %WidgetId widget_id 2
    let nested_id %WidgetId widget_id 3
    let parent_action %ActionId action_id 10
    let nested_action %ActionId action_id 30
    let root_hint %LayoutHint layout_hint_fixed 12 8
    let button_hint %LayoutHint layout_hint_fixed 6 4
    let nested_hint %LayoutHint layout_hint_fixed 3 2
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let parent_config %ButtonConfig button_config parent_id "Parent" parent_action
    let parent %WidgetDescriptor widget_button parent_config button_hint
    let nested_config %ButtonConfig button_config nested_id "Nested" nested_action
    let nested %WidgetDescriptor widget_button nested_config nested_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view1 %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 parent
    let view2 %ViewTreeArena unwrap_ok view_tree_arena_add_child view1 1 nested
    let root_rect %GuiRect gui_rect_new 0 0 12 8
    let parent_rect %GuiRect gui_rect_new 1 1 6 4
    let nested_rect %GuiRect gui_rect_new 2 2 3 2
    let root_layout %LayoutNode layout_node root_id root_rect
    let parent_layout %LayoutNode layout_node parent_id parent_rect
    let nested_layout %LayoutNode layout_node nested_id nested_rect
    let layout0 %LayoutTreeArena unwrap_ok layout_tree_arena_single root_layout
    let layout1 %LayoutTreeArena unwrap_ok layout_tree_arena_add_child layout0 0 parent_layout
    let layout2 %LayoutTreeArena unwrap_ok layout_tree_arena_add_child layout1 1 nested_layout
    let point %GuiPoint gui_point_new 2 2
    let hit_value %i32:
        match layout_tree_arena_hit_test &layout2 point:
            Option::Some id:
                widget_id_value id
            Option::None:
                0
    let action_value %i32 gui_event_action_value route_pointer_action_in_arena &view2 &layout2 point
    view_tree_arena_free view2
    layout_tree_arena_free layout2
    let hit_ok %bool eq hit_value 3
    let action_ok %bool eq action_value 30
    if and hit_ok action_ok:
        then 30
        else 0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "route_pointer_action_in_arena_hits_nested_front_node"
        |> test_report_push assert_eq_i32 "return value" 30 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## route_pointer_action_in_arena_ignores_disabled_missing_and_outside

[目的/もくてき]:
- arena routing でも disabled widget は action event を返さないことを確認します。
- layout hit があっても対応する `WidgetId` が `ViewTreeArena` に無い場合は `Option::None` になることを確認します。
- bounds 外の pointer は `Option::None` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"route_pointer_action_in_arena_ignores_disabled_missing_and_outside\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "alloc/gui/routing" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn is_none_event %fn Option GuiEvent bool \value:
    match value:
        Option::Some _event:
            false
        Option::None:
            true

fn run_case %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let disabled_id %WidgetId widget_id 2
    let missing_id %WidgetId widget_id 3
    let disabled_action %ActionId action_id 20
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let button_hint %LayoutHint layout_hint_fixed 4 3
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let disabled_config %ButtonConfig button_config disabled_id "Disabled" disabled_action
    let disabled_node %ViewNode button disabled_config
    let disabled %WidgetDescriptor widget_descriptor disabled_id disabled_node button_hint true true "Disabled"
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view1 %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 disabled
    let root_rect %GuiRect gui_rect_new 0 0 20 10
    let disabled_rect %GuiRect gui_rect_new 1 1 4 3
    let missing_rect %GuiRect gui_rect_new 10 1 4 3
    let root_layout %LayoutNode layout_node root_id root_rect
    let disabled_layout %LayoutNode layout_node disabled_id disabled_rect
    let missing_layout %LayoutNode layout_node missing_id missing_rect
    let layout0 %LayoutTreeArena unwrap_ok layout_tree_arena_single root_layout
    let layout1 %LayoutTreeArena unwrap_ok layout_tree_arena_add_child layout0 0 disabled_layout
    let layout2 %LayoutTreeArena unwrap_ok layout_tree_arena_add_child layout1 0 missing_layout
    let disabled_point %GuiPoint gui_point_new 2 2
    let missing_point %GuiPoint gui_point_new 11 2
    let outside_point %GuiPoint gui_point_new 30 30
    let disabled_none %bool is_none_event route_pointer_action_in_arena &view1 &layout2 disabled_point
    let missing_none %bool is_none_event route_pointer_action_in_arena &view1 &layout2 missing_point
    let outside_none %bool is_none_event route_pointer_action_in_arena &view1 &layout2 outside_point
    view_tree_arena_free view1
    layout_tree_arena_free layout2
    let all_none %bool and:
        disabled_none
        and:
            missing_none
            outside_none
    if all_none:
        then 0
        else 9

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "route_pointer_action_in_arena_ignores_disabled_missing_and_outside"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
