# alloc/gui tree

このファイルは retained `ViewTree` / `LayoutTree` が platform handle を持たず、`Result` と `Option` による bounded contract で扱えることを固定します。

## view_tree_tracks_focusable_widget_order

[目的/もくてき]:
- label は focus target ではなく、button だけが focus traversal に入ることを確認します。
- focus target は callback ではなく `WidgetId` として返ることを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"view_tree_tracks_focusable_widget_order\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"assert_eq_i32\" expected=\"7\" actual=\"7\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "title" root_hint
    let button_id %WidgetId widget_id 7
    let button_action %ActionId action_id 2
    let button_config0 %ButtonConfig button_config button_id "Run" button_action
    let button_hint %LayoutHint layout_hint_fixed 6 1
    let button_node %WidgetDescriptor widget_button button_config0 button_hint
    let tree0 %ViewTree view_tree_single root
    let tree %ViewTree unwrap_ok view_tree_add_child tree0 button_node
    let count_check assert_eq_i32 1 view_tree_focusable_count &tree
    let first_check match view_tree_first_focusable_id &tree:
        Option::Some id:
            assert_eq_i32 7 widget_id_value id
        Option::None:
            assert false
    let checks0 test_report_push test_report_new "view_tree_tracks_focusable_widget_order" count_check
    let checks test_report_push checks0 first_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## view_tree_capacity_overflow_is_result_error

[目的/もくてき]:
- bounded tree の capacity overflow が panic や silent no-op ではなく `GuiError::ResourceExhausted` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"view_tree_capacity_overflow_is_result_error\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *
#import "std/test" as *

fn run_case %fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let c1_id %WidgetId widget_id 2
    let c1_action %ActionId action_id 2
    let c1_config %ButtonConfig button_config c1_id "A" c1_action
    let c1_hint %LayoutHint layout_hint_fixed 4 1
    let c1 %WidgetDescriptor widget_button c1_config c1_hint
    let c2_id %WidgetId widget_id 3
    let c2_action %ActionId action_id 3
    let c2_config %ButtonConfig button_config c2_id "B" c2_action
    let c2_hint %LayoutHint layout_hint_fixed 4 1
    let c2 %WidgetDescriptor widget_button c2_config c2_hint
    let c3_id %WidgetId widget_id 4
    let c3_action %ActionId action_id 4
    let c3_config %ButtonConfig button_config c3_id "C" c3_action
    let c3_hint %LayoutHint layout_hint_fixed 4 1
    let c3 %WidgetDescriptor widget_button c3_config c3_hint
    let tree0 %ViewTree view_tree_single root
    let tree1 %ViewTree unwrap_ok view_tree_add_child tree0 c1
    let tree2 %ViewTree unwrap_ok view_tree_add_child tree1 c2
    match view_tree_add_child tree2 c3:
        Result::Ok _tree:
            1
        Result::Err error:
            match error:
                GuiError::ResourceExhausted:
                    0
                _:
                    2

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "view_tree_capacity_overflow_is_result_error"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## layout_tree_tracks_layout_node_children

[目的/もくてき]:
- `LayoutTree` が placement result を `Option` child として保持できることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"layout_tree_tracks_layout_node_children\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"1\" actual=\"1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *
#import "std/test" as *

fn run_case %fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_bounds %GuiRect gui_rect_new 0 0 10 2
    let root %LayoutNode layout_node root_id root_bounds
    let child_id %WidgetId widget_id 2
    let child_bounds %GuiRect gui_rect_new 0 2 10 2
    let child %LayoutNode layout_node child_id child_bounds
    let tree0 %LayoutTree layout_tree_single root
    match layout_tree_add_child tree0 child:
        Result::Ok tree:
            layout_tree_child_count &tree
        Result::Err _error:
            9

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "layout_tree_tracks_layout_node_children"
        |> test_report_push assert_eq_i32 "return value" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## view_tree_arena_allows_nested_focus_order

[目的/もくてき]:
- allocator-backed `ViewTreeArena` が root + child + grandchild の depth を保持し、focus target を tree insertion order で数えられることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"view_tree_arena_allows_nested_focus_order\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"2\" actual=\"2\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let first_id0 %WidgetId widget_id 7
    let first_action %ActionId action_id 2
    let first_config %ButtonConfig button_config first_id0 "Open" first_action
    let first_hint %LayoutHint layout_hint_fixed 6 1
    let first %WidgetDescriptor widget_button first_config first_hint
    let nested_id0 %WidgetId widget_id 9
    let nested_action %ActionId action_id 3
    let nested_config %ButtonConfig button_config nested_id0 "Save" nested_action
    let nested_hint %LayoutHint layout_hint_fixed 6 1
    let nested %WidgetDescriptor widget_button nested_config nested_hint
    let tree0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let tree1 %ViewTreeArena unwrap_ok view_tree_arena_add_child tree0 0 first
    let tree2 %ViewTreeArena unwrap_ok view_tree_arena_add_child tree1 1 nested
    let focus_total %i32 view_tree_arena_focusable_count &tree2
    let nested_depth %i32:
        match view_tree_arena_get_node &tree2 2:
            Option::Some node:
                view_tree_arena_node_depth &node
            Option::None:
                99
    let first_id %i32:
        match view_tree_arena_first_focusable_id &tree2:
            Option::Some id:
                widget_id_value id
            Option::None:
                0
    view_tree_arena_free tree2
    let arena_ok %bool and:
        eq focus_total 2
        and:
            eq nested_depth 2
            eq first_id 7
    if arena_ok:
        then 2
        else 9

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "view_tree_arena_allows_nested_focus_order"
        |> test_report_push assert_eq_i32 "return value" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## view_tree_arena_invalid_parent_is_result_error

[目的/もくてき]:
- allocator-backed `ViewTreeArena` の不正 parent index が panic や silent no-op ではなく `GuiError::InvalidCommand` になることを確認します。
- Err path では API が消費した tree owner を error payload に戻すため、呼び出し側が cleanup できます。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"view_tree_arena_invalid_parent_is_result_error\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let child_id0 %WidgetId widget_id 2
    let child_action %ActionId action_id 3
    let child_config %ButtonConfig button_config child_id0 "Run" child_action
    let child_hint %LayoutHint layout_hint_fixed 4 1
    let child %WidgetDescriptor widget_button child_config child_hint
    let tree %ViewTreeArena unwrap_ok view_tree_arena_single root
    match view_tree_arena_add_child tree 99 child:
        Result::Ok out:
            view_tree_arena_free out
            1
        Result::Err error:
            let kind %GuiError view_tree_arena_add_child_error_kind &error
            let rejected %WidgetDescriptor view_tree_arena_add_child_error_child &error
            let rejected_id %i32 widget_id_value widget_descriptor_id &rejected
            let recovered %ViewTreeArena view_tree_arena_add_child_error_tree error
            let recovered_len %i32 view_tree_arena_len &recovered
            view_tree_arena_free recovered
            match kind:
                GuiError::InvalidCommand:
                    let length_ok %bool eq recovered_len 1
                    let rejected_ok %bool eq rejected_id 2
                    if and length_ok rejected_ok:
                        then 0
                        else 3
                _:
                    2

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "view_tree_arena_invalid_parent_is_result_error"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## layout_tree_arena_tracks_nested_layout_depth

[目的/もくてき]:
- allocator-backed `LayoutTreeArena` が nested layout node を parent index と depth で保持できることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"layout_tree_arena_tracks_nested_layout_depth\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"3\" actual=\"3\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_bounds %GuiRect gui_rect_new 0 0 10 4
    let root %LayoutNode layout_node root_id root_bounds
    let child_id %WidgetId widget_id 2
    let child_bounds %GuiRect gui_rect_new 1 1 8 2
    let child %LayoutNode layout_node child_id child_bounds
    let nested_id %WidgetId widget_id 3
    let nested_bounds %GuiRect gui_rect_new 2 2 4 1
    let nested %LayoutNode layout_node nested_id nested_bounds
    let tree0 %LayoutTreeArena unwrap_ok layout_tree_arena_single root
    let tree1 %LayoutTreeArena unwrap_ok layout_tree_arena_add_child tree0 0 child
    let tree2 %LayoutTreeArena unwrap_ok layout_tree_arena_add_child tree1 1 nested
    let node_total %i32 layout_tree_arena_len &tree2
    let depth %i32:
        match layout_tree_arena_get_node &tree2 2:
            Option::Some node:
                layout_tree_arena_node_depth &node
            Option::None:
                99
    layout_tree_arena_free tree2
    let arena_ok %bool and:
        eq node_total 3
        eq depth 2
    if arena_ok:
        then 3
        else 9

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "layout_tree_arena_tracks_nested_layout_depth"
        |> test_report_push assert_eq_i32 "return value" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## layout_tree_arena_invalid_parent_returns_owner

[目的/もくてき]:
- allocator-backed `LayoutTreeArena` の不正 parent index が owner-recovery error を返すことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"layout_tree_arena_invalid_parent_returns_owner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_bounds %GuiRect gui_rect_new 0 0 10 4
    let root %LayoutNode layout_node root_id root_bounds
    let child_id %WidgetId widget_id 2
    let child_bounds %GuiRect gui_rect_new 1 1 8 2
    let child %LayoutNode layout_node child_id child_bounds
    let tree %LayoutTreeArena unwrap_ok layout_tree_arena_single root
    match layout_tree_arena_add_child tree 8 child:
        Result::Ok out:
            layout_tree_arena_free out
            1
        Result::Err error:
            let kind %GuiError layout_tree_arena_add_child_error_kind &error
            let rejected %LayoutNode layout_tree_arena_add_child_error_child &error
            let rejected_bounds %GuiRect layout_node_bounds &rejected
            let rejected_x %i32 gui_rect_x &rejected_bounds
            let recovered %LayoutTreeArena layout_tree_arena_add_child_error_tree error
            let recovered_len %i32 layout_tree_arena_len &recovered
            layout_tree_arena_free recovered
            match kind:
                GuiError::InvalidCommand:
                    let length_ok %bool eq recovered_len 1
                    let rejected_ok %bool eq rejected_x 1
                    if and length_ok rejected_ok:
                        then 0
                        else 3
                _:
                    2

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "layout_tree_arena_invalid_parent_returns_owner"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
