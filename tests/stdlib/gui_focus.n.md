# alloc/gui focus

このファイルは focus traversal が callback や platform API に依存せず、`ViewTree` の bounded order と current id だけで次/前の focus target を返すことを固定します。

## focus_order_moves_next_and_previous

[目的/もくてき]:
- label は focus target から除外し、button child だけを tree order で辿ることを確認します。
- current が `none` の場合、next は先頭、previous は末尾から開始できることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/app" as *
#import "alloc/gui/focus" as *
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
    let order %FocusOrder focus_order_from_view_tree &tree2
    let count_check assert_eq_i32 2 focus_order_count &order
    let start_next_check match focus_order_next &order none:
        Option::Some id:
            assert_eq_i32 7 widget_id_value id
        Option::None:
            assert false
    let first_current %Option WidgetId some first_id
    let step_next_check match focus_order_next &order first_current:
        Option::Some id:
            assert_eq_i32 9 widget_id_value id
        Option::None:
            assert false
    let start_previous_check match focus_order_previous &order none:
        Option::Some id:
            assert_eq_i32 9 widget_id_value id
        Option::None:
            assert false
    let checks1 checks_push checks_new count_check
    let checks2 checks_push checks1 start_next_check
    let checks3 checks_push checks2 step_next_check
    let checks checks_push checks3 start_previous_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## focus_tree_returns_none_for_edges_and_stale_current

[目的/もくてき]:
- disabled button は focus traversal から除外されることを確認します。
- 最後の target から next、先頭の target から previous、order に存在しない current id は `none` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/app" as *
#import "alloc/gui/focus" as *
#import "alloc/gui/tree" as *
#import "alloc/gui/widget" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn is_none_widget_id %fn Option WidgetId bool \value:
    match value:
        Option::Some _id:
            false
        Option::None:
            true

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 5
    let disabled_id %WidgetId widget_id 6
    let child_id %WidgetId widget_id 8
    let stale_id %WidgetId widget_id 99
    let root_action %ActionId action_id 1
    let disabled_action %ActionId action_id 2
    let child_action %ActionId action_id 3
    let hint %LayoutHint layout_hint_fixed 8 1
    let root_config %ButtonConfig button_config root_id "Root" root_action
    let root %WidgetDescriptor widget_button root_config hint
    let disabled_config %ButtonConfig button_config disabled_id "Disabled" disabled_action
    let disabled_node %ViewNode button disabled_config
    let disabled %WidgetDescriptor widget_descriptor disabled_id disabled_node hint true true "Disabled"
    let child_config %ButtonConfig button_config child_id "Child" child_action
    let child %WidgetDescriptor widget_button child_config hint
    let tree0 %ViewTree view_tree_single root
    let tree1 %ViewTree unwrap_ok view_tree_add_child tree0 disabled
    let tree2 %ViewTree unwrap_ok view_tree_add_child tree1 child
    let first_check match focus_next_in_tree &tree2 none:
        Option::Some id:
            assert_eq_i32 5 widget_id_value id
        Option::None:
            assert false
    let child_current %Option WidgetId some child_id
    let previous_check match focus_previous_in_tree &tree2 child_current:
        Option::Some id:
            assert_eq_i32 5 widget_id_value id
        Option::None:
            assert false
    let stale_current %Option WidgetId some stale_id
    let edge_next_check assert is_none_widget_id focus_next_in_tree &tree2 child_current
    let stale_check assert is_none_widget_id focus_previous_in_tree &tree2 stale_current
    let checks1 checks_push checks_new first_check
    let checks2 checks_push checks1 previous_check
    let checks3 checks_push checks2 edge_next_check
    let checks checks_push checks3 stale_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## focus_arena_moves_next_and_previous_across_nested_nodes

[目的/もくてき]:
- `ViewTreeArena` の nested node を、bounded `ViewTree` と同じ focus traversal contract へ接続することを確認します。
- `WidgetId` と arena index を混同せず、insertion order で next / previous を返すことを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/app" as *
#import "alloc/gui/focus" as *
#import "alloc/gui/tree" as *
#import "alloc/gui/widget" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn is_none_widget_id %fn Option WidgetId bool \value:
    match value:
        Option::Some _id:
            false
        Option::None:
            true

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let first_id %WidgetId widget_id 7
    let first_action %ActionId action_id 2
    let first_config %ButtonConfig button_config first_id "Open" first_action
    let first_hint %LayoutHint layout_hint_fixed 6 1
    let first %WidgetDescriptor widget_button first_config first_hint
    let nested_id %WidgetId widget_id 9
    let nested_action %ActionId action_id 3
    let nested_config %ButtonConfig button_config nested_id "Save" nested_action
    let nested_hint %LayoutHint layout_hint_fixed 6 1
    let nested %WidgetDescriptor widget_button nested_config nested_hint
    let sibling_id %WidgetId widget_id 11
    let sibling_action %ActionId action_id 4
    let sibling_config %ButtonConfig button_config sibling_id "Close" sibling_action
    let sibling_hint %LayoutHint layout_hint_fixed 6 1
    let sibling %WidgetDescriptor widget_button sibling_config sibling_hint
    let arena0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let arena1 %ViewTreeArena unwrap_ok view_tree_arena_add_child arena0 0 first
    let arena2 %ViewTreeArena unwrap_ok view_tree_arena_add_child arena1 1 nested
    let arena3 %ViewTreeArena unwrap_ok view_tree_arena_add_child arena2 0 sibling
    let no_current %Option WidgetId none
    let nested_current %Option WidgetId some nested_id
    let sibling_current %Option WidgetId some sibling_id
    let stale_current_id %WidgetId widget_id 99
    let stale_current %Option WidgetId some stale_current_id
    let start_next_check match focus_next_in_arena &arena3 no_current:
        Option::Some id:
            assert_eq_i32 7 widget_id_value id
        Option::None:
            assert false
    let nested_next_check match focus_next_in_arena &arena3 nested_current:
        Option::Some id:
            assert_eq_i32 11 widget_id_value id
        Option::None:
            assert false
    let sibling_previous_check match focus_previous_in_arena &arena3 sibling_current:
        Option::Some id:
            assert_eq_i32 9 widget_id_value id
        Option::None:
            assert false
    let start_previous_check match focus_previous_in_arena &arena3 no_current:
        Option::Some id:
            assert_eq_i32 11 widget_id_value id
        Option::None:
            assert false
    let edge_next_check assert is_none_widget_id focus_next_in_arena &arena3 sibling_current
    let stale_previous_check assert is_none_widget_id focus_previous_in_arena &arena3 stale_current
    view_tree_arena_free arena3
    let checks checks_push:
        checks_push:
            checks_push:
                checks_push:
                    checks_push:
                        checks_push checks_new start_next_check
                        nested_next_check
                    sibling_previous_check
                start_previous_check
            edge_next_check
        stale_previous_check
    let shown checks_print_report checks
    checks_exit_code shown
```
